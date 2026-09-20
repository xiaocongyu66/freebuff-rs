//! codebuff 上游客户端: session / agent-runs / chat / 广告与 usage 触碰 / 健康观测。
//! 协议逆向自 worker.js (freebuff2api 1.8.9)。
use http_body::Body as _;
use hyper::body::Incoming;
use std::pin::Pin;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub const CODEBUFF_API: &str = "https://www.codebuff.com";
pub const FREEBUFF_API: &str = "https://freebuff.com";
pub const UPSTREAM_HOST: &str = "www.codebuff.com";

/// 业务 API 基址 — 实测 (2026-09-20): freebuff.com 无 /api/v1/* (404 HTML),
/// 而 freebuff.com 授权的 token 在 codebuff.com API 上有效 (200)。
/// 即: 授权入口分源 (auth_flow 按域发起), 业务 API 统一 codebuff.com。
pub fn base_for(_source: &str) -> &'static str {
    CODEBUFF_API
}

pub fn host_of(base: &str) -> &str {
    base.trim_start_matches("https://").trim_start_matches("http://").trim_end_matches('/')
}
pub const BUFFY: &str = "You are Buffy, the strategic coding assistant.";
pub const CONTEXT_PRUNER_AGENT: &str = "context-pruner";

fn http() -> &'static reqwest::Client {
    static C: std::sync::OnceLock<reqwest::Client> = std::sync::OnceLock::new();
    C.get_or_init(|| {
        let px = std::env::var("FREEBUFF_PROXY").ok().filter(|s| !s.is_empty());
        build_client(px.as_deref())
    })
}

/// 构建带可选出站代理的客户端 (http/https/socks5 URL; reqwest socks 需开启 feature)。
pub fn build_client(proxy_url: Option<&str>) -> reqwest::Client {
    let mut b = reqwest::Client::builder().timeout(Duration::from_secs(60));
    if let Some(p) = proxy_url {
        match reqwest::Proxy::all(p) {
            Ok(px) => b = b.proxy(px),
            Err(e) => eprintln!("[freebuff-rs] invalid FREEBUFF_PROXY {p}: {e}"),
        }
    }
    b.build().expect("freebuff http client")
}

/// 每账号代理 client 缓存 (FREEBUFF_ACCOUNTS.proxyUrl 语义)。
pub fn client_for(proxy_url: &str) -> reqwest::Client {
    static MAP: std::sync::OnceLock<Mutex<HashMap<String, reqwest::Client>>> = std::sync::OnceLock::new();
    let map = MAP.get_or_init(|| Mutex::new(HashMap::new()));
    map.lock().unwrap().entry(proxy_url.to_string()).or_insert_with(|| build_client(Some(proxy_url))).clone()
}

/// 上游一次调用的结果
pub struct UpResp {
    pub status: u16,
    pub text: String,
}

impl UpResp {
    pub fn json(&self) -> Option<Value> {
        serde_json::from_str(&self.text).ok()
    }
}

/// 单账号上游调用（不带账号池语义）。extra_headers 由调用方拼桌面版签名头。
pub async fn up(
    method: &str,
    path: &str,
    token: &str,
    body: Option<&Value>,
    extra_headers: &[(&str, String)],
    timeout: Duration,
) -> Result<UpResp, String> {
    up_at(CODEBUFF_API, method, path, token, body, extra_headers, timeout).await
}

pub async fn up_at(
    base: &str,
    method: &str,
    path: &str,
    token: &str,
    body: Option<&Value>,
    extra_headers: &[(&str, String)],
    timeout: Duration,
) -> Result<UpResp, String> {
    up_base(base, None, method, path, token, body, extra_headers, timeout).await
}

/// 指定出站代理 (None=全局/直连) 的上游调用。
pub async fn up_via(
    proxy_url: Option<&str>,
    method: &str,
    path: &str,
    token: &str,
    body: Option<&Value>,
    extra_headers: &[(&str, String)],
    timeout: Duration,
) -> Result<UpResp, String> {
    up_base(CODEBUFF_API, proxy_url, method, path, token, body, extra_headers, timeout).await
}

/// 指定上游基址的调用 (freebuff.com 授权必须同源发起与轮询)。
#[allow(clippy::too_many_arguments)]
pub async fn up_base(
    base: &str,
    proxy_url: Option<&str>,
    method: &str,
    path: &str,
    token: &str,
    body: Option<&Value>,
    extra_headers: &[(&str, String)],
    timeout: Duration,
) -> Result<UpResp, String> {
    let base = base.to_string();
    let method = reqwest::Method::from_bytes(method.as_bytes()).map_err(|e| format!("bad method: {e}"))?;
    let client = match proxy_url {
        Some(p) if !p.is_empty() => client_for(p),
        _ => http().clone(),
    };
    let mut req = client
        .request(method.clone(), format!("{base}{path}"))
        .bearer_auth(token)
        .timeout(timeout);
    if body.is_some() {
        req = req.header("content-type", "application/json");
    }
    // 默认伪装官方 SDK 特征 (调用方可覆写)
    let has_ua = extra_headers.iter().any(|(k, _)| k.eq_ignore_ascii_case("user-agent"));
    if !has_ua {
        req = req.header("user-agent", SDK_USER_AGENT);
        req = req.header("accept", SDK_ACCEPT);
    }
    for (k, v) in extra_headers {
        req = req.header(*k, v);
    }
    if let Some(b) = body {
        req = req.body(b.to_string());
    }
    // 隧道直连优先 (免回环跳); 主机从 base 提取 — freebuff.com 授权不可拨错源
    if let Some((ob, port)) = crate::tunnel_client::tunnel_outbound().await {
        let host = base
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .trim_end_matches('/');
        let mut hdrs: Vec<(&str, String)> = extra_headers.iter().map(|(k, v)| (*k, v.clone())).collect();
        if body.is_some() {
            hdrs.push(("content-type", "application/json".into()));
        }
        if !hdrs.iter().any(|(k, _)| k.eq_ignore_ascii_case("user-agent")) {
            hdrs.push(("user-agent", SDK_USER_AGENT.into()));
            hdrs.push(("accept", SDK_ACCEPT.into()));
        }
        match crate::tunnel_client::request(
            ob, host, method.as_str(), path, token, &hdrs, body,
            timeout.as_secs().max(1),
        ).await {
            Ok((st, body_inc)) => {
                let text = http_body_to_string(body_inc).await;
                return Ok(UpResp { status: st, text });
            }
            Err(e) => {
                // 隧道失败回落直连并降权该节点
                eprintln!("[upstream] tunnel failed, fallback direct: {e}");
                crate::tunnel_client::mark_node_fail(port).await;
            }
        }
    }
    let resp = req.send().await.map_err(|e| format!("upstream: {e}"))?;
    let status = resp.status().as_u16();
    let text = resp.text().await.unwrap_or_default();
    Ok(UpResp { status, text })
}

pub async fn body_to_string(inc: Incoming) -> String {
    http_body_to_string(inc).await
}

async fn http_body_to_string(inc: Incoming) -> String {
    use http_body_util::BodyExt;
    match BodyExt::collect(inc).await {
        Ok(c) => String::from_utf8_lossy(&c.to_bytes()).to_string(),
        Err(e) => format!("body read: {e}"),
    }
}


/// 稳定设备指纹: token 派生, 同一账号永远一致 (官方 enhanced- 前缀)。
/// 每请求 client_id: 13 位 base-36 — 对齐官方 SDK `Math.random().toString(36).substring(2,15)` 形状
/// (同步 Quorinex/Freebuff2API 的 Stealth Request Handling; 真实官方 SDK 每请求随机)
pub fn sdk_client_id() -> String {
    const ALPHABET: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    let mut out = String::with_capacity(13);
    // 轻熵: 时间纳秒 + 地址熵 (无 getrandom 依赖)
    let mut seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0x9E3779B97F4A7C15)
        ^ (&out as *const _ as u64);
    for _ in 0..13 {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        out.push(ALPHABET[((seed >> 33) % 36) as usize] as char);
    }
    out
}

/// 官方 SDK 形态 UA (客户端版本对真实世界是稳定的 — 不每请求变)
pub const SDK_USER_AGENT: &str = "ai-sdk/openai-compatible/1.0.25/codebuff";
pub const SDK_ACCEPT: &str = "application/json, text/event-stream";

pub fn stable_fingerprint(token: &str) -> String {
    let s = format!("freebuff-fp-v2:{token}");
    let mut h1: u32 = 0x811c_9dc5;
    let mut h2: u32 = 0x0100_0193;
    for c in s.encode_utf16() {
        h1 = (h1 ^ c as u32).wrapping_mul(0x0100_0193);
        h2 = (h2 ^ c as u32).wrapping_mul(0x85eb_ca6b);
    }
    format!("enhanced-{h1:08x}{h2:08x}")
}

// ---------------------------------------------------------------------------
// 广告链 + usage 触碰 (30 分钟节流, 失败静默)
// ---------------------------------------------------------------------------
fn behavior_cache() -> &'static Mutex<HashMap<String, Instant>> {
    static C: std::sync::OnceLock<Mutex<HashMap<String, Instant>>> = std::sync::OnceLock::new();
    C.get_or_init(|| Mutex::new(HashMap::new()))
}

fn behavior_due(key: &str) -> bool {
    behavior_due_within(key, 30 * 60)
}

fn behavior_due_within(key: &str, secs: u64) -> bool {
    let mut m = behavior_cache().lock().unwrap();
    let now = Instant::now();
    match m.get(key) {
        Some(t) if now.duration_since(*t) < Duration::from_secs(secs) => false,
        _ => {
            m.insert(key.to_string(), now);
            true
        }
    }
}

pub async fn run_normal_client_behavior(base: &str, token: &str) {
    // glm 的 Reward 池 base=0 全靠广告计数 — 高频看广告; 其余模型维持 30 分钟节流
    let need_reward = true; // 由调用方按 session_model 决定
    run_normal_client_behavior_freq(base, token, need_reward).await
}

pub async fn run_normal_client_behavior_freq(base: &str, token: &str, reward_model: bool) {
    if reward_model {
        run_ads_round(base, token, 3 * 60).await;
    } else if behavior_due(&format!("ads:{token}")) {
        run_ads_round(base, token, 30 * 60).await;
    }
    // 签到: entitlementBreakdown.streak 加每日额度 (worker.js 同款; 失败静默)
    if behavior_due_within(&format!("streak:{token}"), 12 * 3600) {
        let _ = up_at(base, "GET", "/api/v1/freebuff/streak", token, None, &[],
            Duration::from_secs(6)).await;
    }
    if behavior_due(&format!("usage:{token}")) {
        let body = json!({"fingerprintId": stable_fingerprint(token)});
        let _ = up_at(base, "POST", "/api/v1/usage", token, Some(&body), &[], Duration::from_secs(6)).await;
    }
}

async fn run_ads_round(base: &str, token: &str, throttle_secs: u64) {
    if behavior_due_within(&format!("ads:{token}"), throttle_secs) {
        let body = json!({
            "provider": "gravity",
            "sessionId": uuid::Uuid::new_v4().to_string(),
            "surface": "waiting_room",
            "device": {"os": "macos", "timezone": "Asia/Shanghai", "locale": "zh-CN"},
            "userAgent": "Freebuff-CLI/0.0.138",
        });
        if let Ok(ad) = up_at(base, "POST", "/api/v1/ads", token, Some(&body),
            &[("User-Agent", "Freebuff-CLI/0.0.138".into())], Duration::from_secs(6),
        ).await {
            let imp_url = ad.json()
                .and_then(|d| d["ads"][0]["impUrl"].as_str().map(String::from));
            if ad.status == 200 {
                if let Some(imp) = imp_url {
                    // 真实拉取广告资源 (服务端可能校验 impUrl 被请求过)
                    let _ = up_at(base, "GET", &imp_url_path(&imp), token, None, &[],
                        Duration::from_secs(8)).await;
                    let ib = json!({"impUrl": imp, "mode": "free"});
                    let _ = up_at(base, "POST", "/api/v1/ads/impression", token, Some(&ib),
                        &[("User-Agent", "Freebuff-CLI/0.0.138".into())], Duration::from_secs(6)).await;
                }
            }
        }
    }
}

/// impUrl 可能是完整 URL — 提取 path; 纯 path 则原样
fn imp_url_path(url: &str) -> String {
    if let Some(i) = url.find("://") {
        let rest = &url[i + 3..];
        match rest.find('/') {
            Some(j) => rest[j..].to_string(),
            None => "/".into(),
        }
    } else {
        url.to_string()
    }
}

// ---------------------------------------------------------------------------
// session 生命周期
// ---------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct Session {
    pub model: String,
    pub instance_id: String,
    pub expires_at_ms: i64,
    /// 本次响应里额度耗尽的 (model, resetAt_ms) — 供池按模型冷却跳过
    pub exhausted_models: Vec<(String, i64)>,
}

pub fn is_usable_session(s: &Option<Session>) -> bool {
    s.as_ref()
        .map(|s| !s.instance_id.is_empty() && s.expires_at_ms > now_ms() + 60_000)
        .unwrap_or(false)
}

pub fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

pub fn parse_session(data: &Value, requested_model: &str) -> Option<Session> {
    let status = data["status"].as_str()?;
    let instance_id = data["instanceId"].as_str()?.to_string();
    if status != "active" {
        return None;
    }
    // 额度耗尽追踪: remaining == 0 的模型记 (model, resetAt_ms)
    let mut exhausted_models: Vec<(String, i64)> = Vec::new();
    if let Some(rl) = data["rateLimitsByModel"].as_object() {
        for (m, v) in rl {
            let remaining = v["remaining"].as_i64()
                .or_else(|| v["limit"].as_i64().map(|l| l - v["recentCount"].as_i64().unwrap_or(0)));
            if remaining == Some(0) {
                let reset = v["resetAt"].as_str()
                    .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
                    .map(|d| d.timestamp_millis())
                    .unwrap_or_else(|| now_ms() + 24 * 3600 * 1000);
                exhausted_models.push((m.clone(), reset));
            }
        }
    }
    let expires_at_ms = if let Some(t) = data["expiresAt"].as_str() {
        chrono::DateTime::parse_from_rfc3339(t)
            .ok()
            .map(|d| d.timestamp_millis())
    } else if let Some(rem) = data["remainingMs"].as_i64() {
        Some(now_ms() + rem.max(0))
    } else {
        None
    };
    Some(Session {
        model: data["model"].as_str().unwrap_or(requested_model).to_string(),
        instance_id,
        expires_at_ms: expires_at_ms.unwrap_or(0),
        exhausted_models,
    })
}

pub async fn delete_upstream_session(base: &str, token: &str, instance_id: &str) {
    let _ = up_at(base, "DELETE",
        &format!("/api/v1/freebuff/session/{instance_id}"),
        token,
        None,
        &[("x-freebuff-instance-id", instance_id.to_string())],
        Duration::from_secs(10),
    )
    .await;
}

/// GET 当前 session; 返回 (session|None, UpResp)
pub async fn get_session(base: &str, token: &str, instance_hint: Option<&str>) -> Result<UpResp, String> {
    let mut headers: Vec<(&str, String)> =
        vec![("x-freebuff-include-unused-rate-limits", "1".to_string())];
    if let Some(h) = instance_hint {
        headers.push(("x-freebuff-instance-id", h.to_string()));
    }
    up_at(base, "GET", "/api/v1/freebuff/session", token, None, &headers, Duration::from_secs(10)).await
}

/// 创建/复用 session (含 queued 轮询), 全程按 worker.js 语义。
pub async fn ensure_session(
    base: &str,
    token: &str,
    session_model: &str,
    cached: &Option<Session>,
    force_create: bool,
) -> Result<Session, String> {
    // 官方 worker 语义: 广告/签到在 session 创建前发起, 但失败静默跳过不阻塞聊天 → 异步 fire-and-forget
    let base_s = base.to_string();
    let token_s = token.to_string();
    let is_reward_model = session_model.contains("glm");
    tokio::spawn(async move {
        run_normal_client_behavior_freq(&base_s, &token_s, is_reward_model).await;
    });
    if !force_create && is_usable_session(cached) {
        return Ok(cached.clone().unwrap());
    }
    if !force_create {
        let cur = get_session(base, token, None).await?;
        if cur.status == 200 {
            if let Some(s) = cur.json().as_ref().and_then(|d| parse_session(d, session_model)) {
                if s.model == session_model {
                    return Ok(s);
                }
                delete_upstream_session(base, token, &s.instance_id).await;
            }
        }
    }
    // create: 单会话 + 预生成 instance-id (桌面版签名; multi-session 实例会被 chat gate 拒)
    let inst_id = uuid::Uuid::new_v4().to_string();
    let r = up_at(base, "POST", "/api/v1/freebuff/session", token, None,
        &[
            ("x-freebuff-model", session_model.to_string()),
            ("x-freebuff-instance-id", inst_id.clone()),
        ],
        Duration::from_secs(10),
    )
    .await?;
    let data = r.json();
    if r.status == 200 {
        if let Some(s) = data.as_ref().and_then(|d| parse_session(d, session_model)) {
            return Ok(s);
        }
        if data.as_ref().and_then(|d| d["status"].as_str()) == Some("queued") {
            for _ in 0..8 {
                tokio::time::sleep(Duration::from_millis(1500)).await;
                let q = get_session(base, token, Some(&inst_id)).await?;
                if q.status == 200 {
                    if let Some(mut d) = q.json() {
                        if d["instanceId"].is_null() {
                            d["instanceId"] = json!(inst_id);
                        }
                        if let Some(s) = parse_session(&d, session_model) {
                            return Ok(s);
                        }
                    }
                }
            }
            return Err("session stayed queued (retry later)".into());
        }
    }
    if r.status == 409 {
        return Err(format!(
            "session_model_mismatch: {}",
            data.and_then(|d| d["message"].as_str().map(String::from)).unwrap_or_default()
        ));
    }
    Err(format!("create session failed: {} {}", r.status, &r.text[..r.text.len().min(300)]))
}

// ---------------------------------------------------------------------------
// agent-runs 生命周期 (精简: 只 START; chat 只校验 run_id 存在性)
// ---------------------------------------------------------------------------
pub async fn start_run(base: &str, token: &str, agent_id: &str, ancestors: &[String]) -> Result<String, String> {
    let body = json!({"action": "START", "agentId": agent_id, "ancestorRunIds": ancestors});
    let r = up_at(base, "POST", "/api/v1/agent-runs", token, Some(&body), &[], Duration::from_secs(10)).await?;
    if r.status == 200 {
        if let Some(run_id) = r.json().and_then(|d| d["runId"].as_str().map(String::from)) {
            return Ok(run_id);
        }
    }
    Err(format!("start_run failed: {} {}", r.status, &r.text[..r.text.len().min(200)]))
}

pub async fn finish_run(base: &str, token: &str, run_id: &str, total_steps: u64) {
    let body = json!({
        "action": "FINISH", "runId": run_id, "status": "completed",
        "totalSteps": total_steps, "directCredits": 0, "totalCredits": 0
    });
    let _ = up_at(base, "POST", "/api/v1/agent-runs", token, Some(&body), &[], Duration::from_secs(10)).await;
}

// ---------------------------------------------------------------------------
// chat 主调用
// ---------------------------------------------------------------------------
/// 统一上游分块流 (reqwest / 隧道直连两种传输共用)
pub type UnifStream = Pin<Box<dyn futures::Stream<Item = Result<bytes::Bytes, String>> + Send>>;

pub struct ChatUpResp {
    pub status: u16,
    pub text: String,
    pub body: Option<UnifStream>,
}

fn incoming_to_unif(inc: Incoming) -> UnifStream {
    use http_body_util::BodyExt as _;
    Box::pin(async_stream::stream! {
        let mut inc = inc;
        loop {
            match inc.frame().await {
                Some(Ok(f)) => {
                    if let Some(d) = f.data_ref() {
                        yield Ok(d.clone());
                    }
                }
                Some(Err(e)) => { yield Err(e.to_string()); break; }
                None => break,
            }
        }
    })
}

/// POST /api/v1/chat/completions — 流式恒开; 成功时返回原始 Response 供调用方读 SSE。
pub async fn chat_completions(
    base: &str,
    token: &str,
    instance_id: &str,
    payload: &Value,
) -> Result<ChatUpResp, String> {
    // 隧道直连优先 — 主机跟 base (freebuff 账号不得拨到 codebuff)
    if let Some((ob, port)) = crate::tunnel_client::tunnel_outbound().await {
        match crate::tunnel_client::request(
            ob, host_of(base), "POST", "/api/v1/chat/completions", token,
            &[("x-freebuff-instance-id", instance_id.to_string())], Some(payload), 45,
        ).await {
            Ok((st, inc)) => {
                if (200..300).contains(&st) {
                    return Ok(ChatUpResp { status: st, text: String::new(), body: Some(incoming_to_unif(inc)) });
                }
                let text = http_body_to_string(inc).await;
                return Ok(ChatUpResp { status: st, text, body: None });
            }
            Err(e) => {
                eprintln!("[chat] tunnel failed, fallback direct: {e}");
                crate::tunnel_client::mark_node_fail(port).await;
            }
        }
    }
    let client = http().clone();
    let resp = client
        .post(format!("{base}/api/v1/chat/completions"))
        .bearer_auth(token)
        .header("content-type", "application/json")
        .header("x-freebuff-instance-id", instance_id)
        .body(payload.to_string())
        .timeout(Duration::from_secs(45))
        .send()
        .await
        .map_err(|e| format!("chat upstream: {e}"))?;
    let status = resp.status().as_u16();
    if resp.status().is_success() {
        let unif: UnifStream = {
            use futures::StreamExt;
            Box::pin(resp.bytes_stream().map(|r| r.map_err(|e| e.to_string())))
        };
        return Ok(ChatUpResp { status, text: String::new(), body: Some(unif) });
    }
    let text = resp.text().await.unwrap_or_default();
    Ok(ChatUpResp { status, text, body: None })
}
