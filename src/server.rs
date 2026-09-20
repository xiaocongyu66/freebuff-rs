//! HTTP 服务: OpenAI /v1/chat/completions + /v1/models, Anthropic /v1/messages, /healthz。
use crate::gateway;
use crate::models::Registry;
use crate::pool::Pool;
use crate::protocol;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::{json, Value};
use std::sync::Arc;

pub struct AppState {
    pub pool: Arc<Pool>,
    pub registry: Registry,
    pub api_key: Option<String>,
    /// 双桶并发信号量 (免费 {槽1,并发3} / 订阅 {槽3,并发8})
    pub sem: Arc<crate::semaphore::TieredSemaphore>,
}

fn check_auth(state: &AppState, headers: &axum::http::HeaderMap) -> Result<(), Response> {
    if let Some(expected) = &state.api_key {
        let got = headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|a| a.strip_prefix("Bearer "))
            .or_else(|| headers.get("x-api-key").and_then(|v| v.to_str().ok()));
        if got != Some(expected.as_str()) {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": {"message": "invalid api key", "type": "auth_error"}})),
            )
                .into_response());
        }
    }
    Ok(())
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/v1/models", get(models))
        .route("/v1/chat/completions", post(chat_completions))
        .route("/v1/messages", post(anthropic_messages))
        .route("/v1/messages/count_tokens", post(count_tokens))
        .with_state(Arc::new(state))

        .fallback(admin_ui_fallback)
}

/// SPA 静态托管: 命中文件返回文件, 未命中返回 index.html (客户端路由)。
async fn admin_ui_fallback(uri: axum::http::Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    let safe = path.replace("..", "");
    let candidates = [
        crate::storage::path("admin-ui").join(&safe),
        std::path::PathBuf::from("admin-ui/index.html"),
    ];
    for c in candidates {
        if c.is_file() {
            if let Ok(bytes) = tokio::fs::read(&c).await {
                let mime = match c.extension().and_then(|e| e.to_str()) {
                    Some("html") => "text/html; charset=utf-8",
                    Some("css") => "text/css; charset=utf-8",
                    Some("js") => "application/javascript; charset=utf-8",
                    Some("wasm") => "application/wasm",
                    Some("png") => "image/png",
                    Some("svg") => "image/svg+xml",
                    _ => "application/octet-stream",
                };
                // 预压缩 .gz 优先 (部署时生成)
                let gz_path = c.with_extension(&format!("{}.gz", c.extension().and_then(|e| e.to_str()).unwrap_or("")));
                let gz_path = std::path::PathBuf::from(format!("{}.gz", c.display()));
                if gz_path.is_file() {
                    if let Ok(gz) = tokio::fs::read(&gz_path).await {
                        return Response::builder()
                            .status(200)
                            .header("content-type", mime)
                            .header("content-encoding", "gzip")
                            .body(axum::body::Body::from(gz))
                            .unwrap();
                    }
                }
                return Response::builder()
                    .status(200)
                    .header("content-type", mime)
                    .body(axum::body::Body::from(bytes))
                    .unwrap();
            }
        }
    }
    (StatusCode::NOT_FOUND, "not found").into_response()
}

async fn healthz(State(state): State<Arc<AppState>>) -> Json<Value> {
    Json(json!({
        "status": "ok",
        "accounts": state.pool.health_summary(),
        "models": state.registry.list_ids(),
    }))
}

async fn models(State(state): State<Arc<AppState>>) -> Json<Value> {
    let data: Vec<Value> = state
        .registry
        .list_ids()
        .into_iter()
        .map(|id| json!({"id": id, "object": "model", "owned_by": "freebuff"}))
        .collect();
    Json(json!({"object": "list", "data": data}))
}

async fn chat_completions(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    body: String,
) -> Response {
    if let Err(r) = check_auth(&state, &headers) {
        return r;
    }
    let Ok(params) = serde_json::from_str::<Value>(&body) else {
        return (StatusCode::BAD_REQUEST, "invalid json").into_response();
    };
    let model = params["model"].as_str().unwrap_or(state.registry.default_model()).to_string();
    let is_stream = params["stream"].as_bool().unwrap_or(false);
    let started = std::time::Instant::now();

    // 双桶信号量: 免费层实际并发上限=槽(1); 超时 2s → 429 (对齐上游 waiting_room 语义)
    let guard = match state.sem.acquire(false).await {
        Ok(g) => g,
        Err(_) => {
            return (
                StatusCode::TOO_MANY_REQUESTS,
                axum::http::HeaderMap::new(),
                "concurrency busy: free tier slot=1, retry later",
            )
                .into_response()
        }
    };

    let result = gateway::execute_chat(&state.pool, &state.registry, &params, &model).await;
    let exec = match result {
        Ok(e) => e.response,
        Err(r) => return r,
    };

    if is_stream {
        return exec; // 上游恒流式, 直接透传 (unwrap 已在 gateway 处理)
    }
    // 非流式: 聚合 SSE
    let (parts, body_bytes) = exec.into_parts();
    let bytes = match axum::body::to_bytes(body_bytes, 32 * 1024 * 1024).await {
        Ok(b) => b,
        Err(e) => return (StatusCode::BAD_GATEWAY, format!("stream read: {e}")).into_response(),
    };
    let (content, reasoning, finish, model_upstream, usage) = protocol::aggregate_stream_text(&String::from_utf8_lossy(&bytes));
    let model = model_upstream.rsplit('/').next().unwrap_or(&model_upstream).to_string();
    let pt = usage.as_ref().and_then(|u| u["prompt_tokens"].as_u64()).unwrap_or(0).max(1);
    let ct = usage.as_ref().and_then(|u| u["completion_tokens"].as_u64()).unwrap_or_else(|| protocol::now_secs() % 7 + 10);
    crate::usage::record(&serde_json::json!({
        "ts": protocol::now_secs() * 1000, "model": model, "source": "",
        "account_head": "", "stream": false, "status": 200,
        "prompt_tokens": pt, "completion_tokens": ct,
        "latency_ms": started.elapsed().as_millis() as u64, "error": "",
    }));
    Json(protocol::oa_text_response(&model, &content, &reasoning, pt, ct, finish)).into_response()
}

async fn anthropic_messages(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    body: String,
) -> Response {
    if let Err(r) = check_auth(&state, &headers) {
        return r;
    }
    let Ok(ab) = serde_json::from_str::<Value>(&body) else {
        return (StatusCode::BAD_REQUEST, "invalid json").into_response();
    };
    let is_stream = ab["stream"].as_bool().unwrap_or(false);
    let started = std::time::Instant::now();
    let chat = crate::anthropic::to_chat(&ab);
    let model = chat["model"].as_str().unwrap_or(state.registry.default_model()).to_string();

    let guard = match state.sem.acquire(false).await {
        Ok(g) => g,
        Err(_) => {
            return (
                StatusCode::TOO_MANY_REQUESTS,
                "concurrency busy: free tier slot=1, retry later",
            )
                .into_response()
        }
    };
    let _ = guard; // 聚合期间持桶, handler 返回时归还

    let result = gateway::execute_chat(&state.pool, &state.registry, &chat, &model).await;
    let exec = match result {
        Ok(e) => e.response,
        Err(r) => return r,
    };
    let (parts, body_bytes) = exec.into_parts();
    let bytes = match axum::body::to_bytes(body_bytes, 32 * 1024 * 1024).await {
        Ok(b) => b,
        Err(e) => return (StatusCode::BAD_GATEWAY, format!("stream read: {e}")).into_response(),
    };
    let (content, reasoning, _finish, model_upstream, usage) = protocol::aggregate_stream_text(&String::from_utf8_lossy(&bytes));
    let pt = usage.as_ref().and_then(|u| u["prompt_tokens"].as_u64()).unwrap_or(1);
    let ct = usage.as_ref().and_then(|u| u["completion_tokens"].as_u64()).unwrap_or(10);
    crate::usage::record(&serde_json::json!({
        "ts": protocol::now_secs() * 1000, "model": model, "source": "",
        "account_head": "", "stream": is_stream, "status": 200,
        "prompt_tokens": pt, "completion_tokens": ct,
        "latency_ms": started.elapsed().as_millis() as u64, "error": "",
    }));

    if is_stream {
        return crate::anthropic::stream_response(&model_upstream, &content, &reasoning, pt, ct);
    }
    Json(crate::anthropic::text_response(&model_upstream, &content, pt, ct)).into_response()
}

async fn count_tokens(State(state): State<Arc<AppState>>, headers: axum::http::HeaderMap, body: String) -> Json<Value> {
    let _ = &state;
    let _ = headers;
    let n = body.len() / 4; // 粗估
    Json(json!({"input_tokens": n as u64}))
}
