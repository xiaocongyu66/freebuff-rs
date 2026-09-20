//! 网关执行器: 账号重试 × session/run 生命周期 × 流式透传 / 非流式聚合。
use crate::models::ModelEntry;
use crate::pool::Pool;
use crate::protocol;
use crate::upstream;
use axum::response::Response;
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Instant;
use std::time::Duration;

pub struct ExecResult {
    pub response: Response,
}

fn error_response(status: u16, msg: &str) -> Response {
    let body = json!({"error": {"message": msg, "type": "api_error"}});
    Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(axum::body::Body::from(body.to_string()))
        .unwrap()
}

/// 会话保活: 5 分钟一轮, 只刷新临期 session (expires_at < 20min) — 盲目全量轮询
/// 会触发上游 session 端点 IP 级限流(409), 反而打死正常请求 (v0.3.5 实测教训)。
pub async fn session_heartbeat(pool: Arc<Pool>) {
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(300)).await;
        let items = pool.expiring_sessions(20 * 60 * 1000);
        if items.is_empty() {
            continue;
        }
        let base = upstream::base_for("");
        for (token, model, inst) in items {
            match upstream::get_session(&base, &token, Some(&inst)).await {
                Ok(r) if r.status == 200 => {
                    if let Some(d) = r.json() {
                        if let Some(s) = upstream::parse_session(&d, &model) {
                            pool.store_session(&token, &model, s);
                        }
                    }
                }
                // 404/401 = session 真失效; 409/429 = 限流 — 保留缓存别慌删
                Ok(r) if r.status == 404 || r.status == 401 => {
                    pool.drop_session(&token, &model);
                }
                _ => {}
            }
            // 间隔 2s 串行, 摊平请求
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    }
}

fn parse_cooldown(text: &str, status: u16) -> i64 {
    if status == 429 {
        return 60_000;
    }
    // 上游 retryAfterMs / retry_after
    if let Ok(d) = serde_json::from_str::<Value>(text) {
        if let Some(ms) = d["retryAfterMs"].as_i64() {
            return ms.max(1000);
        }
    }
    if status == 403 || text.contains("banned") {
        // free_mode_unavailable = 当日免费模式关闭, 额度重置(美西07:00)即恢复 — 不按 banned 冷却 24h
        if text.contains("free_mode_unavailable") {
            return 3 * 3600 * 1000;
        }
        return 24 * 3600 * 1000;
    }
    30_000
}

fn is_stale_session(status: u16, text: &str) -> bool {
    status == 428 || status == 409 || text.contains("waiting_room_required")
        || text.contains("session_superseded")
}

/// 执行一次 chat: 多号重试; 返回流式 Response (客户端请求是否流式由调用方聚合决定)。
/// 客户端协议转换 (OpenAI/Anthropic 响应格式) 由 handler 层基于本函数的 SSE 输出完成。
pub async fn execute_chat(
    pool: &Arc<Pool>,
    registry: &crate::models::Registry,
    params: &Value,
    model_id: &str,
) -> Result<ExecResult, Response> {
    // 模型可用性路由: 请求了实测不可用的模型 → 自动降级到能用的 (同家族优先)
    let mut model_id = model_id.to_string();
    if let Some(fallback) = pool.fallback_model(&model_id) {
        eprintln!("[chat] model fallback: {model_id} -> {fallback}");
        model_id = fallback;
    }
    let mc: ModelEntry = registry
        .find(&model_id)
        .cloned()
        .ok_or_else(|| {
            let avail = pool.available_models_list();
            if avail.is_empty() {
                error_response(400, &format!("model not found: {model_id}"))
            } else {
                error_response(400, &format!("model not found: {model_id} (实测可用: {})", avail.join(", ")))
            }
        })?;
    if pool.is_empty() {
        return Err(error_response(503, "FREEBUFF_TOKEN not configured (run `freebuff-rs login`)"));
    }

    let session_model = mc.session.as_str();
    let mut last_err = String::new();
    let pool_len = pool.accounts.lock().unwrap().len();

    for acct_try in 0..pool_len {
        let Some(acct) = pool.pick(Some(session_model)) else { break };
        let token = acct.token.clone();
        let base = upstream::base_for(pool.source_of(&token));
        let fp = upstream::stable_fingerprint(&token);

        // session (含缓存钉住)
        let cached = pool.cached_session(&token, session_model);
        let mut sess = match upstream::ensure_session(base, &token, session_model, &cached, false).await {
            Ok(s) => {
                // 撞额度: (token, model) 冷却至重置 — pick 自动切下一账号
                for (m, until) in &s.exhausted_models {
                    if m == session_model {
                        pool.note_model_exhausted(&token, m, *until);
                    }
                }
                s
            }
            Ok(s) => s,
            Err(e) => {
                last_err = e;
                pool.cooldown(&token, 60_000);
                continue;
            }
        };

        // run 链 (10min 复用缓存) — invalidate 后必须更新此变量 (否则重试永远带死 runId)
        let mut run = match run_chain(pool, &token, &mc.agent).await {
            Ok(r) => r,
            Err(e) => {
                last_err = e;
                pool.cooldown(&token, 60_000);
                continue;
            }
        };

        // chat: 失效重试一次, 空流删 session 重建重试一次
        let mut attempt = 0;
        loop {
            attempt += 1;
            let payload = protocol::build_payload(params, &mc, &sess, &run, &upstream::sdk_client_id());
            let resp = upstream::chat_completions(base, &token, &sess.instance_id, &payload).await;
            match resp {
                Ok(upstream::ChatUpResp { status: 200, body: Some(stream), .. }) => {
                    pool.observe(&token, 200, "{}");
                    return Ok(ExecResult {
                        response: to_stream_response(stream, pool.clone(), token.clone(), run.clone()),
                    });
                }
                Ok(upstream::ChatUpResp { status, text, .. }) => {
                    pool.observe(&token, status, &text);
                    // 耗尽/限流(429) → 删 session 重建 → 重试一次
                    // glm 例外: reward 池对会话重置不敏感, 固定会话即可 (重建反而打乱 rhythm)
                    if status == 429 {
                        // 账号级耗尽: pick 跳过 30min + 广告波熔断 (上游已拒, 空转只会加剧)
                        pool.note_account_exhausted(&token);
                        let _ = upstream::mark_exhausted(&token);
                    }
                    if status == 429 && attempt <= 2 && !session_model.contains("glm") {
                        eprintln!("[chat] 429 -> drop session + recreate + retry");
                        pool.drop_session(&token, &mc.session.as_str().to_string().as_str());
                        if invalidate_run(&pool, &token, &mc.agent).await.is_ok() {
                            continue;
                        }
                        continue;
                    }
                    if status == 400 && attempt <= 2 {
                        // ① runId 类: 重开 run 并更新 payload 用的 run 变量 (旧变量是死的)
                        if text.contains("runId Not Running") {
                            match invalidate_run(&pool, &token, &mc.agent).await {
                                Ok(new_run) => { run = new_run; continue; }
                                Err(_) => {}
                            }
                        }
                        // ② 其他 400: session 侧问题 (假页/坏流) — 删会话重建 (glm 除外, 固定会话)
                        if session_model.contains("glm") {
                            // glm: 重开 run (新 run 同步), 不动会话
                            if let Ok(new_run) = invalidate_run(&pool, &token, &mc.agent).await {
                                run = new_run;
                            }
                            continue;
                        }
                        eprintln!("[chat] 400 -> drop session + recreate + retry");
                        pool.drop_session(&token, session_model);
                        match upstream::ensure_session(base, &token, session_model, &None, true).await {
                            Ok(s2) => { sess = s2; continue; }
                            Err(e) => { last_err = e; break; }
                        }
                    }
                    if is_stale_session(status, &text) && attempt == 1 {
                        pool.drop_session(&token, session_model);
                        match upstream::ensure_session(base, &token, session_model, &None, true).await {
                            Ok(s) => { sess = s; continue; }
                            Err(e) => { last_err = e; break; }
                        }
                    }
                    pool.cooldown(&token, parse_cooldown(&text, status));
                    last_err = format!("upstream error ({status}): {}", &text[..text.len().min(300)]);
                    break;
                }
                Err(e) => {
                    last_err = e;
                    pool.cooldown(&token, 60_000);
                    break;
                }
            }
        }
    }
    Err(error_response(502, &format!("all accounts failed: {last_err}")))
}

/// run 链缓存 (10min): root run + context-pruner 子 run。
async fn run_chain(pool: &Arc<Pool>, token: &str, agent_id: &str) -> Result<String, String> {
    const TTL: Duration = Duration::from_secs(10 * 60);
    static CACHE: std::sync::OnceLock<std::sync::Mutex<Vec<(String, String, Instant)>>> =
        std::sync::OnceLock::new();
    let cache = CACHE.get_or_init(|| std::sync::Mutex::new(Vec::new()));
    let key = format!("{token}:{agent_id}");
    {
        let mut c = cache.lock().unwrap();
        c.retain(|(_, _, ts)| ts.elapsed() < TTL);
        if let Some((k, run_id, _)) = c.iter().find(|(k, _, _)| *k == key) {
            let _ = pool;
            return Ok(run_id.clone());
        }
    }
    let root = upstream::start_run(base_for_pool(pool, token), token, agent_id, &[]).await?;
    let _child = upstream::start_run(base_for_pool(pool, token), token, upstream::CONTEXT_PRUNER_AGENT, &[root.clone()]).await;
    cache.lock().unwrap().push((key, root.clone(), Instant::now()));
    Ok(root)
}

fn base_for_pool(pool: &Arc<Pool>, token: &str) -> &'static str {
    upstream::base_for(pool.source_of(token))
}

/// run 失效时从缓存剔除并重开一个 (chat 400 runId Not Running 时调用)。
pub async fn invalidate_run(pool: &Arc<Pool>, token: &str, agent_id: &str) -> Result<String, String> {
    const TTL: Duration = Duration::from_secs(10 * 60);
    static CACHE: std::sync::OnceLock<std::sync::Mutex<Vec<(String, String, Instant)>>> =
        std::sync::OnceLock::new();
    let cache = CACHE.get_or_init(|| std::sync::Mutex::new(Vec::new()));
    let key = format!("{token}:{agent_id}");
    cache.lock().unwrap().retain(|(k, _, _)| *k != key);
    let root = upstream::start_run(base_for_pool(pool, token), token, agent_id, &[]).await?;
    let _child = upstream::start_run(base_for_pool(pool, token), token, upstream::CONTEXT_PRUNER_AGENT, &[root.clone()]).await;
    cache.lock().unwrap().push((key, root.clone(), Instant::now()));
    Ok(root)
}

/// 上游 SSE → 客户端 SSE 透传 (剥 data 包装; 完成后 finishRun)。
fn to_stream_response(
    upstream: crate::upstream::UnifStream,
    pool: Arc<Pool>,
    token: String,
    run_id: String,
) -> Response {
    let stream = async_stream::stream! {
        use futures::StreamExt;
        let mut bytes = upstream;
        let mut buf = String::new();
        while let Some(chunk) = bytes.next().await {
            match chunk {
                Ok(b) => {
                    buf.push_str(&String::from_utf8_lossy(&b));
                    let mut lines: Vec<String> = Vec::new();
                    while let Some(i) = buf.find('\n') {
                        let line = buf[..i].to_string();
                        buf = buf[i + 1..].to_string();
                        lines.push(line);
                    }
                    for line in lines {
                        if let Some(payload) = line.strip_prefix("data:") {
                            let payload = payload.trim();
                            if payload.is_empty() || payload == "[DONE]" {
                                yield Ok::<_, std::convert::Infallible>(axum::body::Bytes::from(format!("{line}\n\n")));
                                continue;
                            }
                            match serde_json::from_str::<Value>(payload) {
                                Ok(obj) => {
                                    let normalized = protocol::unwrap_data(obj);
                                    yield Ok(axum::body::Bytes::from(format!("data: {}\n\n", normalized)));
                                }
                                Err(_) => yield Ok(axum::body::Bytes::from(format!("{line}\n"))),
                            }
                        } else {
                            yield Ok(axum::body::Bytes::from(format!("{line}\n")));
                        }
                    }
                }
                Err(_) => break,
            }
        }
        upstream::finish_run(upstream::base_for(pool.source_of(&token)), &token, &run_id, 1).await;
    };
    Response::builder()
        .status(200)
        .header("content-type", "text/event-stream")
        .header("cache-control", "no-cache")
        .body(axum::body::Body::from_stream(stream))
        .unwrap()
}
