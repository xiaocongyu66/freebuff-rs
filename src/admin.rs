//! 管理后台 API: 节点(代理出站) / 账号 / Key 三类资源的 CRUD + 测活与受限查询。
//! 持久化: freebuff_admin.json (工作目录)。
use crate::pool::{Account, Health, Pool};
use crate::upstream;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeEntry {
    pub id: String,
    pub name: String,
    pub url: String,
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyEntry {
    pub key: String,
    pub name: String,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub created_at: u64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AdminConfig {
    #[serde(default)]
    pub nodes: Vec<NodeEntry>,
    #[serde(default)]
    pub keys: Vec<KeyEntry>,
    /// 导入过的分享链接 — 启动时自动重连
    #[serde(default)]
    pub saved_links: Vec<String>,
    /// 账号授权来源: token → "freebuff" | "codebuff"
    #[serde(default)]
    pub account_sources: std::collections::HashMap<String, String>,
    /// 节点受限记忆: 端口 → [受限, 国家]
    #[serde(default)]
    pub node_restriction: std::collections::HashMap<u16, (bool, String)>,
    /// 账号自定义命名: token → 别名 (两渠道通用)
    #[serde(default)]
    pub account_alias: std::collections::HashMap<String, String>,
}

pub fn config_path() -> std::path::PathBuf {
    std::env::var("FREEBUFF_ADMIN_FILE")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| crate::storage::path("freebuff_admin.json"))
}

pub fn load_config() -> AdminConfig {
    std::fs::read_to_string(config_path())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_config(cfg: &AdminConfig) {
    let _ = std::fs::write(config_path(), serde_json::to_string_pretty(cfg).unwrap_or_default());
}

/// 日志总线: ring 回放 + 广播增量 (SSE)
pub struct LogBus {
    ring: std::sync::Mutex<std::collections::VecDeque<String>>,
    tx: tokio::sync::broadcast::Sender<String>,
}

impl LogBus {
    pub fn new() -> Self {
        let (tx, _) = tokio::sync::broadcast::channel(256);
        Self { ring: std::sync::Mutex::new(std::collections::VecDeque::with_capacity(500)), tx }
    }
    pub fn push(&self, line: &str) {
        let ts = chrono::Utc::now().format("%H:%M:%S");
        let line = format!("[{ts}] {line}");
        {
            let mut r = self.ring.lock().unwrap();
            if r.len() >= 500 { r.pop_front(); }
            r.push_back(line.clone());
        }
        let _ = self.tx.send(line);
    }
    pub fn replay(&self) -> Vec<String> {
        self.ring.lock().unwrap().iter().cloned().collect()
    }
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<String> {
        self.tx.subscribe()
    }
}

pub struct AdminState {
    pub pool: Arc<Pool>,
    pub config: std::sync::Mutex<AdminConfig>,
    pub relay: Arc<crate::relay::Relay>,
    pub auth_flows: Arc<crate::auth_flow::AuthFlows>,
    pub logbus: std::sync::Arc<LogBus>,
    /// 网关鉴权 keys 缓存 — admin 增删/启停时同步 (server 侧读它鉴权)
    pub gateway_keys: std::sync::Arc<std::sync::Mutex<Vec<KeyEntry>>>,
}

impl AdminState {
    /// 把 cfg.keys 同步进网关缓存
    pub fn sync_gateway_keys(&self) {
        let keys = self.config.lock().unwrap().keys.clone();
        *self.gateway_keys.lock().unwrap() = keys;
    }
}

pub fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn err(status: StatusCode, msg: &str) -> Response {
    (status, Json(json!({"error": {"message": msg}}))).into_response()
}

/// 单一探测: 走指定出站代理 GET /api/v1/me + /session, 返回健康与受限信息。
/// 无 token 探测 (仅测代理连通): GET https://www.codebuff.com (期待任意响应)。
pub async fn probe_exit(proxy_url: Option<&str>, token: Option<&str>, source: &str) -> Value {
    let started = std::time::Instant::now();
    let mut out = json!({
        "proxy": proxy_url.unwrap_or("direct"),
        "alive": false,
    });
    // 1) 代理连通性 (HTTP 层): 请求 /api/v1/me
    let me = upstream::up_base(upstream::base_for(source), proxy_url, "GET", "/api/v1/me", token.unwrap_or(""), None, &[], std::time::Duration::from_secs(15)).await;
    match me {
        Ok(r) => {
            out["latency_ms"] = json!(started.elapsed().as_millis() as u64);
            let data = r.json();
            let upstream_state = data.as_ref().and_then(|d| d["status"].as_str().or(d["state"].as_str())).unwrap_or("");
            if (200..300).contains(&r.status) {
                out["alive"] = json!(true);
            } else if r.status == 404 {
                out["alive"] = json!(false);
                out["state"] = json!("endpoint_missing");
            } else if r.status == 401 {
                out["state"] = json!("token_invalid");
            } else if r.status == 403 {
                out["state"] = json!(if upstream_state == "banned" { "banned" } else { "blocked" });
            } else if r.status == 429 {
                out["state"] = json!("rate_limited");
            }
            if let Some(email) = data.as_ref().and_then(|d| d["user"]["email"].as_str()) {
                out["email"] = json!(email);
            }
            // 2) 受限查询: GET session (仅带 token 时)
            if let Some(t) = token {
                if let Ok(sess) = upstream::get_session(upstream::base_for(source), t, None).await {
                    if let Some(d) = sess.json() {
                        out["session"] = json!({
                            "accessTier": d["accessTier"],
                            "countryCode": d["countryCode"],
                            "model": d["model"],
                            "sessionStatus": d["status"],
                            "remainingMs": d["remainingMs"],
                            "expiresAt": d["expiresAt"],
                        });
                        out["restricted"] = json!(d["accessTier"].as_str() == Some("limited"));
                        out["lockedModel"] = json!(if d["accessTier"].as_str() == Some("limited") { d["model"].clone() } else { Value::Null });
                    }
                }
            }
        }
        Err(e) => {
            out["error"] = json!(e);
        }
    }
    out
}

pub async fn router(state: Arc<AdminState>) -> axum::Router {
    axum::Router::new()
        .route("/admin/overview", axum::routing::get(overview))
        .route("/admin/nodes", axum::routing::get(nodes_list).post(nodes_add))
        .route("/admin/nodes/import", axum::routing::post(nodes_import))
        .route("/admin/nodes/{id}", axum::routing::delete(nodes_delete))
        .route("/admin/nodes/{id}/probe", axum::routing::post(nodes_probe))
        .route("/admin/nodes/{id}/stop", axum::routing::post(nodes_stop))
        .route("/admin/auth/start", axum::routing::post(auth_start))
        .route("/admin/auth/{flow_id}", axum::routing::get(auth_poll))
        .route("/admin/accounts", axum::routing::get(accounts_list).post(accounts_add))
        .route("/admin/accounts/{token_head}/source", axum::routing::patch(accounts_move_pool))
        .route("/admin/accounts/{token_head}/balance", axum::routing::get(accounts_balance))
        .route("/admin/accounts/{token_head}/alias", axum::routing::patch(accounts_alias))
        .route("/admin/keys/{key}/toggle", axum::routing::patch(key_toggle))
        .route("/admin/logs/stream", axum::routing::get(logs_stream))
        .route("/admin/usage/summary", axum::routing::get(usage_summary))
        .route("/admin/usage/recent", axum::routing::get(usage_recent))
        .route("/admin/tokens/import", axum::routing::post(tokens_import))
        .route("/admin/accounts/{token_head}", axum::routing::delete(accounts_delete))
        .route("/admin/accounts/{token_head}/probe", axum::routing::post(accounts_probe))
        .route("/admin/keys", axum::routing::get(keys_list).post(keys_add))
        .route("/admin/keys/{key}", axum::routing::delete(keys_delete))
        .with_state(state)
}

async fn overview(State(st): State<Arc<AdminState>>) -> Json<Value> {
    // std Mutex 不可跨 await — 先取完再异步查隧道
    let (nodes, keys) = {
        let cfg = st.config.lock().unwrap();
        (cfg.nodes.clone(), cfg.keys.clone())
    };
    // 网关当前实际出口: FREEBUFF_PROXY 指向的隧道节点 (直连模式)
    let tunnel = match std::env::var("FREEBUFF_PROXY").ok().and_then(|p| p.rsplit(':').next()?.parse::<u16>().ok()) {
        Some(port) => match st.relay.node_brief(port).await {
            Some((scheme, host)) => Some(json!({"scheme": scheme, "host": host, "mode": "直连"})),
            None => None,
        },
        None => None,
    };
    Json(json!({
        "accounts": st.pool.health_summary(),
        "nodes": nodes,
        "keys": keys.iter().map(|k| json!({"name": k.name, "enabled": k.enabled, "key": format!("{}...{}", &k.key[..4.min(k.key.len())], &k.key[k.key.len().saturating_sub(4)..])})).collect::<Vec<_>>(),
        "models": crate::models::current().list_ids(),
        "tunnel": tunnel,
    }))
}

// ---------------- 节点管理 ----------------

async fn nodes_list(State(st): State<Arc<AdminState>>) -> Json<Value> {
    let manual = {
        let cfg = st.config.lock().unwrap();
        cfg.nodes.clone()
    };
    let relay_state = st.relay.state().await;
    Json(json!({"nodes": manual, "relay": relay_state}))
}

/// 快速导入: 一行一个分享链接 (socks5/hy2/trojan/ss/vless/vmess/tuic/http)。
#[axum::debug_handler]
async fn nodes_import(
    State(st): State<Arc<AdminState>>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, Response> {
    let text = body["text"].as_str().unwrap_or("");
    if text.trim().is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, "text required (one link per line)"));
    }
    let (imported, skipped, errors) = st.relay.import_many(text).await;
    // 持久化新链接
    if !imported.is_empty() {
        let mut cfg = load_config(); // 磁盘最新 — 防止内存旧副本覆掉并发写入(如授权来源)
        for n in &imported {
            if let Some(link) = n["link"].as_str() {
                if !cfg.saved_links.iter().any(|l| l == link) {
                    cfg.saved_links.push(link.to_string());
                }
            }
        }
        save_config(&cfg);
        *st.config.lock().unwrap() = cfg;
    }
    Ok(Json(json!({
        "ok": true,
        "imported": imported,
        "imported_count": imported.len(),
        "skipped": skipped,
        "errors": errors,
    })))
}

async fn nodes_add(
    State(st): State<Arc<AdminState>>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, Response> {
    let name = body["name"].as_str().unwrap_or("").to_string();
    let url = body["url"].as_str().unwrap_or("").to_string();
    if url.is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, "url required"));
    }
    let mut cfg = st.config.lock().unwrap();
    let entry = NodeEntry {
        id: uuid::Uuid::new_v4().simple().to_string()[..8].to_string(),
        name: if name.is_empty() { format!("node-{}", cfg.nodes.len() + 1) } else { name },
        url,
        enabled: body["enabled"].as_bool().unwrap_or(true),
    };
    let id = entry.id.clone();
    cfg.nodes.push(entry);
    save_config(&cfg);
    Ok(Json(json!({"ok": true, "id": id})))
}

async fn nodes_delete(
    State(st): State<Arc<AdminState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Json<Value> {
    let mut cfg = load_config();
    cfg.nodes.retain(|n| n.id != id);
    save_config(&cfg);
    *st.config.lock().unwrap() = cfg;
    Json(json!({"ok": true}))
}

async fn nodes_probe(
    State(st): State<Arc<AdminState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Json<Value> {
    // relay 节点: 端口精确匹配 (UI 传端口), 其次链接前缀 — 直连该节点出站测受限
    let by_port = id.parse::<u16>().ok();
    let (ob, port) = match st.relay.find_node(id.parse::<u64>().ok(), &id).await {
        Some((ob, port)) => (Some(ob), Some(port)),
        None => (None, None),
    };
    let out = match ob {
        Some(ob) => {
            // 经该节点出站请求 /api/v1/freebuff/session → accessTier + 出口国家 (需登录态)
            let acct_tok = st.pool.accounts.lock().unwrap().first().map(|a| a.token.clone()).unwrap_or_default();
            let t0 = std::time::Instant::now();
            let r = crate::tunnel_client::request(
                ob, upstream::host_of(upstream::CODEBUFF_API), "GET",
                "/api/v1/freebuff/session", &acct_tok, &[("x-freebuff-include-unused-rate-limits", "1".into())],
                None, 15,
            ).await;
            match r {
                Ok((stt, body)) => {
                    let text = crate::upstream::body_to_string(body).await;
                    let d = serde_json::from_str::<Value>(&text).ok();
                    let tier = d.as_ref().and_then(|d| d["accessTier"].as_str()).unwrap_or("").to_string();
                    let country = d.as_ref().and_then(|d| d["countryCode"].as_str()).unwrap_or("").to_string();
                    let restricted = tier == "limited";
                    if let Some(p) = port {
                        st.relay.mark_probe(p, restricted, &country).await;
                        // 持久化受限记忆 (重启后仍避开受限出口)
                        let mut cfg = load_config();
                        cfg.node_restriction.insert(p, (restricted, country.clone()));
                        save_config(&cfg);
                        *st.config.lock().unwrap() = cfg;
                    }
                    json!({
                        "alive": (200..500).contains(&stt),
                        "latency_ms": t0.elapsed().as_millis() as u64,
                        "restricted": restricted,
                        "accessTier": tier,
                        "countryCode": country,
                        "status": stt,
                    })
                }
                Err(e) => json!({"alive": false, "error": e}),
            }
        }
        None => {
            // manual 节点回退
            let url = {
                let cfg = st.config.lock().unwrap();
                cfg.nodes.iter().find(|n| n.id == id).map(|n| n.url.clone())
            };
            match url {
                Some(u) => probe_exit(Some(&u), None, "codebuff").await,
                None => json!({"error": "node not found"}),
            }
        }
    };
    Json(out)
}

async fn nodes_stop(
    State(st): State<Arc<AdminState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Json<Value> {
    // 端口精确匹配优先, 其次链接前缀
    let by_port = id.parse::<u64>().ok();
    let stopped = st.relay.stop_node(by_port, &id).await;
    Json(json!({"ok": stopped}))
}

// ---------------- 在线授权 ----------------

async fn auth_start(
    State(st): State<Arc<AdminState>>,
    body: Option<Json<Value>>,
) -> Result<Json<Value>, Response> {
    let provider = body
        .as_ref()
        .and_then(|b| b.0["provider"].as_str())
        .unwrap_or("freebuff")
        .to_string();
    match crate::auth_flow::start(&st.auth_flows, &provider).await {
        Ok(v) => Ok(Json(v)),
        Err(e) => Err(err(StatusCode::BAD_GATEWAY, &e)),
    }
}

async fn auth_poll(
    State(st): State<Arc<AdminState>>,
    axum::extract::Path(flow_id): axum::extract::Path<String>,
) -> Result<Json<Value>, Response> {
    match crate::auth_flow::poll(&st.auth_flows, &st.pool, &flow_id).await {
        Ok(v) => Ok(Json(v)),
        Err(e) => Err(err(StatusCode::BAD_GATEWAY, &e)),
    }
}

// ---------------- 账号管理 ----------------

async fn accounts_list(State(st): State<Arc<AdminState>>) -> Json<Value> {
    Json(st.pool.health_summary())
}

async fn accounts_add(
    State(st): State<Arc<AdminState>>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, Response> {
    let token = body["token"].as_str().unwrap_or("").trim().to_string();
    if token.len() < 8 {
        return Err(err(StatusCode::BAD_REQUEST, "token too short"));
    }
    let provider = body["provider"].as_str().unwrap_or("freebuff").to_string();
    st.pool.accounts.lock().unwrap().push(Account { token: token.clone(), uid: None, source: provider.clone(), alias: String::new() });
    let mut cfg = load_config();
    cfg.account_sources.insert(token.clone(), provider);
    save_config(&cfg);
    *st.config.lock().unwrap() = cfg;
    // 追加到 tokens 文件
    let path = crate::creds::tokens_file();
    let mut tokens = crate::creds::load_tokens();
    if !tokens.iter().any(|t| t == &token) {
        tokens.push(token);
        let _ = std::fs::write(&path, tokens.join("\n") + "\n");
    }
    Ok(Json(json!({"ok": true, "accounts": st.pool.accounts.lock().unwrap().len()})))
}

/// 账号移池: body {"provider": "freebuff"|"codebuff"}
async fn accounts_move_pool(
    State(st): State<Arc<AdminState>>,
    axum::extract::Path(token_head): axum::extract::Path<String>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, Response> {
    let provider = body["provider"].as_str().unwrap_or("").to_string();
    if provider != "freebuff" && provider != "codebuff" {
        return Err(err(StatusCode::BAD_REQUEST, "provider must be freebuff|codebuff"));
    }
    let moved = {
        let mut accs = st.pool.accounts.lock().unwrap();
        let mut moved = 0;
        for a in accs.iter_mut() {
            if a.token.starts_with(&token_head) {
                a.source = provider.clone();
                moved += 1;
                // 持久化来源 (磁盘回写防覆写)
                let mut cfg = load_config();
                cfg.account_sources.insert(a.token.clone(), provider.clone());
                save_config(&cfg);
                *st.config.lock().unwrap() = cfg;
            }
        }
        moved
    };
    if moved == 0 {
        return Err(err(StatusCode::NOT_FOUND, "account not found"));
    }
    Ok(Json(json!({"ok": true, "moved": moved})))
}

/// 账号别名: body {"name": "主号"} — 两渠道账号均可命名
async fn accounts_alias(
    State(st): State<Arc<AdminState>>,
    axum::extract::Path(token_head): axum::extract::Path<String>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, Response> {
    let name = body["name"].as_str().unwrap_or("").trim().to_string();
    if name.len() > 32 {
        return Err(err(StatusCode::BAD_REQUEST, "name too long (max 32)"));
    }
    // 全量 token 找到才可持久化完整键 (head 匹配可能多个 — 全部改)
    let full_tokens: Vec<String> = {
        let accs = st.pool.accounts.lock().unwrap();
        accs.iter().filter(|a| a.token.starts_with(&token_head)).map(|a| a.token.clone()).collect()
    };
    if full_tokens.is_empty() {
        return Err(err(StatusCode::NOT_FOUND, "account not found"));
    }
    st.pool.set_alias(&token_head, &name);
    let mut cfg = load_config();
    for t in &full_tokens {
        if name.is_empty() {
            cfg.account_alias.remove(t);
        } else {
            cfg.account_alias.insert(t.clone(), name.clone());
        }
    }
    save_config(&cfg);
    *st.config.lock().unwrap() = cfg;
    st.logbus.push(&format!("[account] 改名 {token_head}… → {name}"));
    Ok(Json(json!({"ok": true, "renamed": full_tokens.len()})))
}

/// Key 启用/禁用: body {"enabled": bool} — 禁用后网关拒绝该 key (401)
async fn key_toggle(
    State(st): State<Arc<AdminState>>,
    axum::extract::Path(key): axum::extract::Path<String>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, Response> {
    let enabled = body["enabled"].as_bool().unwrap_or(true);
    let mut cfg = load_config();
    let found = cfg.keys.iter_mut().find(|k| k.key == key).map(|k| {
        k.enabled = enabled;
        k.name.clone()
    });
    match found {
        Some(kname) => {
            save_config(&cfg);
            *st.config.lock().unwrap() = cfg;
            // 同步网关鉴权缓存
            st.sync_gateway_keys();
            st.logbus.push(&format!("[key] {} {}", kname, if enabled { "已启用" } else { "已禁用" }));
            Ok(Json(json!({"ok": true, "enabled": enabled})))
        }
        None => Err(err(StatusCode::NOT_FOUND, "key not found")),
    }
}

/// 实时日志 SSE: ring buffer 回放 + 增量推送
async fn logs_stream(
    State(st): State<Arc<AdminState>>,
) -> axum::response::Response {
    use axum::response::sse::{Event, Sse};
    let mut rx = st.logbus.subscribe();
    let replay = st.logbus.replay();
    let stream = async_stream::stream! {
        for line in replay {
            yield Ok::<_, std::convert::Infallible>(Event::default().data(line));
        }
        loop {
            match rx.recv().await {
                Ok(line) => yield Ok::<_, std::convert::Infallible>(Event::default().data(line)),
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                Err(_) => break,
            }
        }
    };
    Sse::new(stream)
        .keep_alive(axum::response::sse::KeepAlive::default())
        .into_response()
}

async fn usage_summary(
    axum::extract::Query(q): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Json<Value> {
    let hours: i64 = q.get("hours").and_then(|h| h.parse().ok()).unwrap_or(24).clamp(1, 720);
    Json(crate::usage::summary(hours))
}

async fn usage_recent(
    axum::extract::Query(q): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Json<Value> {
    let limit: i64 = q.get("limit").and_then(|l| l.parse().ok()).unwrap_or(50).clamp(1, 500);
    Json(crate::usage::recent(limit))
}

async fn tokens_import(
    State(st): State<Arc<AdminState>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let text = body["text"].as_str().unwrap_or("");
    Json(crate::import::import_into_pool(&st.pool, text))
}

/// 账号余额/积分: session 响应里的 freebucks + subscription + 每模型 rateLimits
async fn accounts_balance(
    State(st): State<Arc<AdminState>>,
    axum::extract::Path(token_head): axum::extract::Path<String>,
) -> Result<Json<Value>, Response> {
    // 注意: 不可持 accounts 锁调 source_of (同线程双锁死锁) — 先取 token 再查来源
    let token = {
        let accs = st.pool.accounts.lock().unwrap();
        match accs.iter().find(|a| a.token.starts_with(&token_head)) {
            Some(a) => a.token.clone(),
            None => return Err(err(StatusCode::NOT_FOUND, "account not found")),
        }
    };
    let source = st.pool.source_of(&token);
    let base = crate::upstream::base_for(source);
    let sess = crate::upstream::get_session(base, &token, None)
        .await
        .map_err(|e| err(StatusCode::BAD_GATEWAY, &e))?;
    let d = sess.json().unwrap_or(Value::Null);
    // 汇总实测可用模型 (rateLimitsByModel 键集)
    if let Some(rl) = d["rateLimitsByModel"].as_object() {
        let models: Vec<String> = rl.keys().cloned().collect();
        if !models.is_empty() {
            st.pool.note_available_models(&models);
        }
    }
    Ok(Json(json!({
        "source": source,
        "accessTier": d["accessTier"],
        "countryCode": d["countryCode"],
        "status": d["status"],
        "freebucks": d["freebucks"],
        "subscription": d["subscription"],
        "rateLimitsByModel": d["rateLimitsByModel"],
    })))
}

async fn accounts_delete(
    State(st): State<Arc<AdminState>>,
    axum::extract::Path(token_head): axum::extract::Path<String>,
) -> Json<Value> {
    st.pool.accounts.lock().unwrap().retain(|a| !a.token.starts_with(&token_head));
    let path = crate::creds::tokens_file();
    let tokens = crate::creds::load_tokens();
    let _ = std::fs::write(&path, tokens.iter().filter(|t| !t.starts_with(&token_head)).cloned().collect::<Vec<_>>().join("\n") + "\n");
    Json(json!({"ok": true, "accounts": st.pool.accounts.lock().unwrap().len()}))
}

async fn accounts_probe(
    State(st): State<Arc<AdminState>>,
    axum::extract::Path(token_head): axum::extract::Path<String>,
) -> Json<Value> {
    let token = st.pool.accounts.lock().unwrap().iter().find(|a| a.token.starts_with(&token_head)).map(|a| a.token.clone());
    let Some(token) = token else {
        return Json(json!({"error": "account not found"}));
    };
    // 账号受限查询: 用该账号当前配置的节点 (第一个启用的) 或直连
    let proxy = {
        let cfg = st.config.lock().unwrap();
        cfg.nodes.iter().find(|n| n.enabled).map(|n| n.url.clone())
    };
    let source = st.pool.source_of(&token);
    let out = probe_exit(proxy.as_deref(), Some(&token), source).await;
    // 同步健康观测
    if let Some(alive) = out["alive"].as_bool() {
        let mut h = st.pool.health.lock().unwrap();
        let entry = h.entry(token.clone()).or_insert_with(|| Health {
            alive: None, state: "unknown".into(), uid: None, checked_at: std::time::Instant::now(),
            score: 60,
        });
        entry.alive = Some(alive);
        if let Some(s) = out["state"].as_str() {
            entry.state = s.to_string();
        }
        entry.checked_at = std::time::Instant::now();
    }
    Json(out)
}

// ---------------- Key 管理 ----------------

async fn keys_list(State(st): State<Arc<AdminState>>) -> Json<Value> {
    let cfg = st.config.lock().unwrap();
    Json(json!({"keys": cfg.keys}))
}

async fn keys_add(
    State(st): State<Arc<AdminState>>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, Response> {
    let name = body["name"].as_str().unwrap_or("").to_string();
    let mut cfg = load_config();
    let key = format!("fb-{}", uuid::Uuid::new_v4().simple());
    let key_name = if name.is_empty() { format!("key-{}", cfg.keys.len() + 1) } else { name };
    let key_count = cfg.keys.len() + 1;
    cfg.keys.push(KeyEntry {
        key: key.clone(),
        name: key_name.clone(),
        enabled: true,
        created_at: now(),
    });
    save_config(&cfg);
    *st.config.lock().unwrap() = cfg;
    st.sync_gateway_keys();
    st.logbus.push(&format!("[key] 签发 {key_name} ({key_count}个)"));
    Ok(Json(json!({"ok": true, "key": key})))
}

async fn keys_delete(
    State(st): State<Arc<AdminState>>,
    axum::extract::Path(key): axum::extract::Path<String>,
) -> Json<Value> {
    let mut cfg = load_config();
    cfg.keys.retain(|k| k.key != key);
    let remaining = cfg.keys.len();
    save_config(&cfg);
    *st.config.lock().unwrap() = cfg;
    st.sync_gateway_keys();
    st.logbus.push(&format!("[key] 删除 {key} 前缀 (剩 {remaining}个)"));
    Json(json!({"ok": true}))
}
