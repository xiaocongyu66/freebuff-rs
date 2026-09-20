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
    /// 管理台签发的 keys (enabled 才放行) — 与 admin 共享
    pub gateway_keys: std::sync::Arc<std::sync::Mutex<Vec<crate::admin::KeyEntry>>>,
    /// 日志总线
    pub logbus: std::sync::Arc<crate::admin::LogBus>,
}

fn check_auth(state: &AppState, headers: &axum::http::HeaderMap) -> Result<(), Response> {
    let got = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|a| a.strip_prefix("Bearer "))
        .or_else(|| headers.get("x-api-key").and_then(|v| v.to_str().ok()));
    // ① 管理台签发的 keys (enabled 才有效)
    if let Some(k) = got {
        let keys = state.gateway_keys.lock().unwrap();
        if let Some(e) = keys.iter().find(|e| &e.key == k) {
            return if e.enabled {
                Ok(())
            } else {
                Err((
                    StatusCode::UNAUTHORIZED,
                    Json(json!({"error": {"message": "api key disabled", "type": "auth_error"}})),
                )
                    .into_response())
            };
        }
    }
    // ② env 兼容 (sk-test / FREEBUFF_API_KEY)
    if let Some(expected) = &state.api_key {
        if got == Some(expected.as_str()) {
            return Ok(());
        }
    }
    {
        if let Some(expected) = &state.api_key {
            let _ = expected;
        }
    }
    if let Some(expected) = &state.api_key {
        let got2 = headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|a| a.strip_prefix("Bearer "))
            .or_else(|| headers.get("x-api-key").and_then(|v| v.to_str().ok()));
        let _ = got2;
        let _ = expected;
    }
    // 未命中任何 key
    {
        let matched_env = state
            .api_key
            .as_ref()
            .map(|e| got == Some(e.as_str()))
            .unwrap_or(false);
        if !matched_env {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": {"message": "invalid api key", "type": "auth_error"}})),
            )
                .into_response());
        }
    }
    Ok(())
}

#[allow(dead_code)]
fn check_auth_old(state: &AppState, headers: &axum::http::HeaderMap) -> Result<(), Response> {
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

/// 内嵌管理台产物 (编译期): 单二进制自带 web — 替换一个文件即完成全部更新。
/// embedded/admin-ui 由 CI 构建时刷新 (dx build + tailwind + gzip), 仓库内保留最近版本供本地 cargo build。
static EMBEDDED_UI: include_dir::Dir = include_dir::include_dir!("$CARGO_MANIFEST_DIR/embedded/admin-ui");

fn mime_of(path: &str) -> &'static str {
    match path.rsplit('.').next().unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "application/javascript; charset=utf-8",
        "wasm" => "application/wasm",
        "png" => "image/png",
        "svg" => "image/svg+xml",
        _ => "application/octet-stream",
    }
}

fn embedded_response(path: &str) -> Option<Response> {
    // .gz 产物直接下发 (content-encoding: gzip)
    let gz = EMBEDDED_UI.get_file(&format!("{path}.gz"));
    if let Some(f) = gz.or_else(|| EMBEDDED_UI.get_file(path)) {
        let is_gz = gz.is_some();
        let mime = mime_of(path);
        let mut b = Response::builder().status(200).header("content-type", mime);
        if is_gz {
            b = b.header("content-encoding", "gzip").header("cache-control", "public, max-age=3600");
        }
        return Some(b.body(axum::body::Body::from(f.contents().to_vec())).unwrap());
    }
    None
}

/// SPA 静态托管: 外挂 admin-ui/ 优先 (热替换), 否则内嵌产物; 未命中返回 index.html (客户端路由)。
async fn admin_ui_fallback(uri: axum::http::Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    let safe = path.replace("..", "");
    let asset = if safe.is_empty() { "index.html" } else { &safe };
    // 1) 外挂热替换目录
    let ext = crate::storage::path("admin-ui").join(asset);
    if ext.is_file() {
        let mime = mime_of(asset);
        let gz = std::path::PathBuf::from(format!("{}.gz", ext.display()));
        if gz.is_file() {
            if let Ok(g) = tokio::fs::read(&gz).await {
                return Response::builder().status(200).header("content-type", mime)
                    .header("content-encoding", "gzip").body(axum::body::Body::from(g)).unwrap();
            }
        }
        if let Ok(bytes) = tokio::fs::read(&ext).await {
            return Response::builder().status(200).header("content-type", mime)
                .body(axum::body::Body::from(bytes)).unwrap();
        }
    }
    // 2) 内嵌产物
    if let Some(r) = embedded_response(asset) {
        return r;
    }
    // 3) SPA 路由兜底: index.html
    if let Some(r) = embedded_response("index.html") {
        return r;
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
    // 免费层默认可用集 ∪ 账号实测集 — 付费层模型不展示 (升级后实测集自动扩展)
    let mut ids: Vec<String> = crate::models::FREE_TIER_MODELS
        .iter()
        .map(|s| s.to_string())
        .collect();
    for m in state.pool.available_models_list() {
        if !ids.contains(&m) {
            ids.push(m);
        }
    }
    let data: Vec<Value> = ids
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
    state.logbus.push(&format!(
        "[chat] {} {}+{} tok / {}ms",
        model, pt, ct, started.elapsed().as_millis()
    ));
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
