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
    let no_ua = extra_headers.iter().any(|(k, _)| *k == "x-no-ua");
    let extra_headers: Vec<&(&str, String)> = extra_headers.iter().filter(|(k, _)| *k != "x-no-ua").collect();
    let has_ua = extra_headers.iter().any(|(k, _)| k.eq_ignore_ascii_case("user-agent"));
    if no_ua {
        // 桌面版协议: session 端点用客户端默认 UA (不手动设)
    } else if !has_ua {
        req = req.header("user-agent", SDK_USER_AGENT);
        req = req.header("accept", SDK_ACCEPT);
    }
    for (k, v) in extra_headers.iter() {
        req = req.header(*k, *v);
    }
    if let Some(b) = body {
        req = req.body(b.to_string());
    }
    // 隧道直连优先 (免回环跳); 熔断态直接跳过 (连续失败 60s 内不试隧道)
    if crate::tunnel_client::tunnel_ok() {
    if let Some((ob, port)) = crate::tunnel_client::tunnel_outbound_for(token).await {
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
                crate::tunnel_client::tunnel_mark_success();
                return Ok(UpResp { status: st, text });
            }
            Err(e) => {
                // 隧道失败回落直连并降权该节点 + 记熔断
                eprintln!("[upstream] tunnel failed, fallback direct: {e}");
                crate::tunnel_client::mark_node_fail(port).await;
                crate::tunnel_client::tunnel_mark_failure();
            }
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
    // 官方 enhanced 指纹 = sha256(JSON 硬件面) base64url (~43 字符, 'enhanced-' 前缀)
    // 我们用同长度同字符集伪面: token 稳定 → 同账号永远同一指纹 (官方语义), 形状与真实 CLI 无差别
    let s = format!("freebuff-hw-v2:{token}");
    let mut h1: u32 = 0x811c_9dc5;
    let mut h2: u32 = 0x0100_0193;
    for c in s.encode_utf16() {
        h1 = (h1 ^ c as u32).wrapping_mul(0x0100_0193);
        h2 = (h2 ^ c as u32).wrapping_mul(0x85eb_ca6b);
    }
    // base64url 字符表 (官方 digest('base64url') 输出形状)
    const B64U: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::with_capacity(44);
    let mut mix = ((h1 as u64) << 32) | h2 as u64;
    for _ in 0..43 {
        out.push(B64U[(mix & 63) as usize] as char);
        mix = mix.wrapping_mul(0x9E37_79B9_7F4A_7C15).rotate_right(7) ^ (mix >> 13);
    }
    format!("enhanced-{out}")
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

/// 伪随机 (每 key 稳定 hash 熵): 时间纳秒 ^ 指针地址 — 每次调用不同, 但同 key 同轮只算一次
fn jitter_u64(key: &str, lo: u64, hi: u64) -> u64 {
    let t = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as u64)
        .unwrap_or(0);
    let h = t ^ (key.as_ptr() as u64).rotate_left(17) ^ (key.len() as u64);
    lo + (h % (hi - lo).max(1))
}

fn jitter_secs(key: &str, lo: u64, hi: u64) -> u64 {
    let t = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as u64)
        .unwrap_or(0);
    let h = t ^ (key.as_ptr() as u64).rotate_left(17) ^ (key.len() as u64);
    lo + (h % (hi - lo).max(1))
}

fn behavior_due_within(key: &str, secs: u64) -> bool {
    behavior_due_jitter(key, secs, secs)
}

fn behavior_due_jitter(key: &str, lo: u64, hi: u64) -> bool {
    let secs = jitter_secs(key, lo, hi);
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
    run_normal_client_behavior_src(base, token, reward_model, "freebuff").await
}

pub async fn run_normal_client_behavior_src(base: &str, token: &str, reward_model: bool, source: &str) {
    let ads_key = format!("ads:{token}");
    // 耗尽熔断: 上游 429 后 30 分钟内不发任何广告/签到 (空转烧池 + 滥用信号)
    {
        let m = behavior_cache().lock().unwrap();
        if let Some(t) = m.get(&format!("exhausted:{token}")) {
            if t.elapsed() < std::time::Duration::from_secs(30 * 60) {
                return;
            }
        }
    }
    if reward_model {
        // glm Reward: 2-4 分钟窗口内随机 — 统一间隔是蜜罐签名 (同刻同模式 = 批量特征)
        run_ads_round(base, token, 2 * 60, 4 * 60, source).await;
    } else if behavior_due_jitter(&ads_key, 20 * 60, 40 * 60) {
        run_ads_round(base, token, 20 * 60, 40 * 60, source).await;
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

async fn run_ads_round(base: &str, token: &str, lo: u64, hi: u64, source: &str) {
    let ads_key = format!("ads:{token}");
    if !behavior_due_jitter(&ads_key, lo, hi) {
        return;
    }
    // 突发模式 (更真人): 真人看广告是一波连看 4-6 个 (攒次数), 不是匀速单发
    // 波内间隔 2-5s 随机; 波与波之间仍是节流窗口
    let burst: u64 = 4 + (jitter_u64(&ads_key, 0, 3)); // 4..=6
    for i in 0..burst {
        if i > 0 {
            let gap = jitter_u64(&format!("{ads_key}:{i}"), 2, 6);
            tokio::time::sleep(Duration::from_secs(gap)).await;
        }
        watch_one_ad(base, token, source).await;
    }
}

/// 账号耗尽标记 (广告波熔断用)
pub fn mark_exhausted(token: &str) {
    let key = format!("exhausted:{token}");
    let mut m = behavior_cache().lock().unwrap();
    m.insert(key, Instant::now());
}

/// auction 时的浏览器 UA (官方: impression 的 userAgent 必须= auction 时 UA, 否则 Gravity bot 过滤判死)
fn ad_browser_ua() -> &'static str {
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36"
}

/// 双 CLI UA: 按 source 选品牌 (CodebuffAI/codebuff: IS_FREEBUFF ? 'Freebuff-CLI' : 'Codebuff-CLI')
pub fn cli_product_ua(source: &str) -> String {
    if source == "freebuff" {
        format!("Freebuff-CLI/{}", CLI_VERSION)
    } else {
        format!("Codebuff-CLI/{}", CODEBUFF_CLI_VERSION)
    }
}

/// codebuff CLI 版本 (npm: codebuff@1.0.688)
pub const CODEBUFF_CLI_VERSION: &str = "1.0.688";

/// freebuff CLI 版本 (本机官方 CLI freebuff@0.0.180)
pub const CLI_VERSION: &str = "0.0.180";

async fn watch_one_ad(base: &str, token: &str, source: &str) {
    let product_ua = cli_product_ua(source);
    let body = json!({
        "provider": "gravity",
        "sessionId": uuid::Uuid::new_v4().to_string(),
        "surface": "waiting_room",
        "device": {"os": "macos", "timezone": "Asia/Shanghai", "locale": "zh-CN"},
        "userAgent": ad_browser_ua(),
    });
    if let Ok(ad) = up_at(base, "POST", "/api/v1/ads", token, Some(&body),
        &[("User-Agent", product_ua.clone())], Duration::from_secs(6),
    ).await {
        let imp_url = ad.json()
            .and_then(|d| d["ads"][0]["impUrl"].as_str().map(String::from));
        if ad.status == 200 {
            if let Some(imp) = imp_url {
                // 官方新版 impression: POST /ads/impression { impUrl, mode, userAgent(浏览器UA), os, clientEventId } + x-client-event-id 头
                let ev = uuid::Uuid::new_v4().to_string();
                let ib = json!({
                    "impUrl": imp,
                    "mode": "lite",
                    "userAgent": ad_browser_ua(),
                    "os": "macos",
                    "clientEventId": ev,
                });
                let _ = up_at(base, "POST", "/api/v1/ads/impression", token, Some(&ib),
                    &[
                        ("User-Agent", product_ua.clone()),
                        ("x-client-event-id", ev.into()),
                    ], Duration::from_secs(6),
                ).await;
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
    /// 动态次数表: model → remaining (rateLimitsByModel 实时)
    pub quota_map: std::collections::HashMap<String, i64>,
    /// 账号层剩余 (freebucks daily remaining; None=响应无)
    pub layer_remaining: Option<i64>,
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
    let mut quota_map: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    if let Some(rl) = data["rateLimitsByModel"].as_object() {
        for (m, v) in rl {
            let remaining = v["remaining"].as_i64()
                .or_else(|| v["limit"].as_i64().map(|l| l - v["recentCount"].as_i64().unwrap_or(0)));
            if let Some(r) = remaining {
                quota_map.insert(m.clone(), r);
            }
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
    // 账号层剩余: freebucks.daily.remaining
    let layer_remaining = data["freebucks"]["daily"]["remaining"].as_i64();
    Some(Session {
        model: data["model"].as_str().unwrap_or(requested_model).to_string(),
        instance_id,
        expires_at_ms: expires_at_ms.unwrap_or(0),
        exhausted_models,
        quota_map,
        layer_remaining,
    })
}

pub async fn delete_upstream_session(base: &str, token: &str, instance_id: &str) {
    // 官方 session-api.ts: DELETE 也带 base 头(x-fb-timezone/first-tab) + instance 头
    let _ = up_at(base, "DELETE",
        &format!("/api/v1/freebuff/session/{instance_id}"),
        token,
        None,
        &[
            ("x-fb-timezone", "Asia/Shanghai".into()),
            ("x-fb-first-tab-discount", "1".into()),
            ("x-freebuff-instance-id", instance_id.to_string()),
        ],
        Duration::from_secs(10),
    )
    .await;
}

/// GET 当前 session; 返回 (session|None, UpResp)
pub async fn get_session(base: &str, token: &str, instance_hint: Option<&str>) -> Result<UpResp, String> {
    // 官方 session-api.ts: headers base 块(Authorization+x-fb-timezone+first-tab-discount)对全部 method 生效
    // 桌面版协议: session 端点不手动设 UA (fetch 默认) — 剥离 SDK UA
    let mut headers: Vec<(&str, String)> = vec![
        ("x-freebuff-include-unused-rate-limits", "1".to_string()),
        ("x-fb-timezone", "Asia/Shanghai".into()),
        ("x-fb-first-tab-discount", "1".into()),
        ("x-no-ua", "1".into()),
    ];
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
    ensure_session_src(base, token, session_model, cached, force_create, "freebuff").await
}

pub async fn ensure_session_src(
    base: &str,
    token: &str,
    session_model: &str,
    cached: &Option<Session>,
    force_create: bool,
    source: &str,
) -> Result<Session, String> {
    // 官方 worker 语义: 广告/签到在 session 创建前发起, 但失败静默跳过不阻塞聊天 → 异步 fire-and-forget
    // 智能触发: 只在 (a) glm reward 池 (countsAdmissions 需看广告攒次数) 或 (b) 探测性维持(30-40min 稀疏) 时看
    let base_s = base.to_string();
    let token_s = token.to_string();
    let is_reward_model = session_model.contains("glm");
    let source_s = source.to_string();
    tokio::spawn(async move {
        run_normal_client_behavior_src(&base_s, &token_s, is_reward_model, &source_s).await;
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
            // 官方 session-api.ts: x-fb-timezone(调度偏好) + x-fb-first-tab-discount(首屏折扣标记) — 缺头=非真实 CLI
            ("x-fb-timezone", "Asia/Shanghai".into()),
            ("x-fb-first-tab-discount", "1".into()),
        ],
        Duration::from_secs(10),
    )
    .await?;
    let data = r.json();
    // create 被拒: 409=已有活跃会话(删旧重建安全); 403=池级拒绝(删了活的建不出新的→死活抖动, 保留旧会话)
    if r.status != 200 && r.status != 429 && r.status != 403 {
        eprintln!("[session] create {} -> {}, try delete+recreate", session_model, r.status);
        if let Ok(cur) = get_session(base, token, None).await {
            if cur.status == 200 {
                if let Some(d) = cur.json() {
                    if let Some(old) = d["instanceId"].as_str() {
                        delete_upstream_session(base, token, old).await;
                    }
                }
            }
        }
        let retry = up_at(base, "POST", "/api/v1/freebuff/session", token, None,
            &[
                ("x-freebuff-model", session_model.to_string()),
                ("x-freebuff-instance-id", inst_id.clone()),
                ("x-fb-timezone", "Asia/Shanghai".into()),
                ("x-fb-first-tab-discount", "1".into()),
            ],
            Duration::from_secs(10),
        ).await?;
        if retry.status == 200 {
            if let Some(s) = retry.json().as_ref().and_then(|d| parse_session(d, session_model)) {
                return Ok(s);
            }
        }
    }
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
    // 隧道直连优先 — 熔断态跳过; 主机跟 base
    if crate::tunnel_client::tunnel_ok() {
    if let Some((ob, port)) = crate::tunnel_client::tunnel_outbound_for(token).await {
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
