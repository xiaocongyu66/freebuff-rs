//! 内置代理中继: 分享链接 (socks5/hy2/trojan/ss/vless/vmess/tuic) → 本地 mixed 出站。
//! 移植自 grok-free-register BuiltinProxyRelay — 每节点一个 sing-box 进程。
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Mutex;
use tokio::io::AsyncWriteExt;

#[derive(Default)]
pub struct RelayNode {
    pub link: String,
    pub scheme: String,
    pub local_port: u16,
    pub proxy: String,
    pub config_path: PathBuf,
    pub log_path: PathBuf,
    pub child_id: Option<u32>,
    pub alive: bool,
    pub ob: Option<Arc<crate::tunnel::outbound::Outbound>>,
    /// 该出口 IP 是否受限 (limited 层) — probe 时更新, 调度优先避开
    pub restricted: bool,
    /// 出口国家 (probe 时更新)
    pub country: String,
    /// 连接失败计数 — 调度降权, 探测成功清零
    pub fail_count: u32,
}

pub struct Relay {
    pub nodes: tokio::sync::Mutex<HashMap<String, RelayNode>>,
    pub work_dir: PathBuf,
    pub host: String,
    pub max_nodes: usize,
    pub start_port: u16,
}

impl Relay {
    pub fn new() -> Self {
        Self {
            nodes: tokio::sync::Mutex::new(HashMap::new()),
            work_dir: std::env::var("FREEBUFF_RELAY_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|_| crate::storage::path("logs/proxy-relay")),
            host: "127.0.0.1".into(),
            max_nodes: 48,
            start_port: 19080,
        }
    }

    fn allocate_port(&self, busy: &HashMap<String, RelayNode>) -> u16 {
        let used: std::collections::HashSet<u16> = busy.values().map(|n| n.local_port).collect();
        let mut p = self.start_port;
        while used.contains(&p) {
            p += 1;
        }
        p
    }

    /// 活进程检查: kill(0) 语义 — child_id 存在且进程在。
    fn prune_dead(&self, nodes: &mut HashMap<String, RelayNode>) {
        let dead: Vec<String> = nodes
            .iter()
            .filter(|(_, n)| !node_process_alive(n))
            .map(|(k, _)| k.clone())
            .collect();
        for k in dead {
            if let Some(n) = nodes.get_mut(&k) {
                n.alive = false;
            }
        }
    }

    /// 导入单个分享链接: 纯 Rust 校验 + 每节点一个本地 mixed 入站端口。
    pub async fn import(&self, raw_link: &str) -> Result<(String, String), String> {
        let link = raw_link.trim().to_string();
        if link.is_empty() {
            return Err("empty link".into());
        }
        // 段1: 短锁 — 查重/容量/端口分配 (不跨 await)
        let port;
        {
            let mut nodes = self.nodes.lock().await;
            self.prune_dead(&mut nodes);
            if let Some(n) = nodes.get(&link) {
                if n.alive {
                    return Ok((n.scheme.clone(), n.proxy.clone()));
                }
            }
            if nodes.values().filter(|n| n.alive).count() >= self.max_nodes {
                return Err(format!("max nodes reached: {}", self.max_nodes));
            }
            port = self.allocate_port(&nodes);
        }
        // 纯自实现: 链接 → 出站 (无需外部进程)
        let ob = std::sync::Arc::new(crate::tunnel::outbound::Outbound::from_link(&link)?);
        let ob_for_node = ob.clone();
        let scheme = match &*ob {
            crate::tunnel::outbound::Outbound::Socks5(_) => "socks5",
            crate::tunnel::outbound::Outbound::Trojan(_) => "trojan",
            crate::tunnel::outbound::Outbound::Vless(_, _) => "vless",
            crate::tunnel::outbound::Outbound::Shadowsocks(_) => "ss",
            crate::tunnel::outbound::Outbound::Hy2(_) => "hy2",
        }
        .to_string();
        // 纯直连: 不再建 127.0.0.1 本地入站监听 — 网关直接持有 Outbound 进程内拨号
        let proxy = format!("direct://");
        let mut nodes = self.nodes.lock().await;
        nodes.insert(
            link.clone(),
            RelayNode {
                link: link.clone(),
                scheme: scheme.clone(),
                local_port: port,
                proxy: proxy.clone(),
                config_path: std::path::PathBuf::new(),
                log_path: std::path::PathBuf::new(),
                child_id: None,
                alive: true,
                ob: Some(ob_for_node),
                restricted: false,
                country: String::new(),
                fail_count: 0,
            },
        );
        Ok((scheme, proxy))
    }

    /// 按本地端口取隧道出站 (网关直连通道)
    pub async fn outbound_for_port(&self, port: u16) -> Option<Arc<crate::tunnel::outbound::Outbound>> {
        let nodes = self.nodes.lock().await;
        nodes.values().find(|n| n.local_port == port).and_then(|n| n.ob.clone())
    }

    /// 账号粘性出站: token hash → 稳定节点 (同账号恒同 IP, 多账号分散到多 IP)
    /// 目标节点不可用(fail 多/死)时顺延到下一候选
    pub async fn sticky_outbound(&self, token: &str) -> Option<(Arc<crate::tunnel::outbound::Outbound>, u16)> {
        let nodes = self.nodes.lock().await;
        let mut candidates: Vec<&RelayNode> = nodes.values().filter(|n| n.ob.is_some()).collect();
        if candidates.is_empty() {
            return None;
        }
        candidates.sort_by_key(|n| (n.fail_count, n.restricted));
        // 稳定 hash (FNV-1a): 同 token 恒同序位
        let mut h: u64 = 0xcbf29ce484222325;
        for b in token.bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        let start = (h as usize) % candidates.len();
        for k in 0..candidates.len() {
            let n = candidates[(start + k) % candidates.len()];
            if n.fail_count < 3 {
                return n.ob.clone().map(|ob| (ob, n.local_port));
            }
        }
        candidates[start].ob.clone().map(|ob| (ob, candidates[start].local_port))
    }

    /// 调度: 优先未受限出口 (受限=出口IP导致 limited 层), 全受限再退回
    pub async fn best_outbound(&self) -> Option<(Arc<crate::tunnel::outbound::Outbound>, u16)> {
        let mut nodes = self.nodes.lock().await;
        self.prune_dead(&mut nodes);
        let mut candidates: Vec<&RelayNode> = nodes.values().filter(|n| n.ob.is_some()).collect();
        if candidates.is_empty() {
            return None;
        }
        // 排序: 少失败 + 未受限在前
        candidates.sort_by_key(|n| (n.fail_count, n.restricted));
        let n = candidates[0];
        n.ob.clone().map(|ob| (ob, n.local_port))
    }

    /// 按端口/链接前缀找节点 → (出站, 端口)
    pub async fn find_node(&self, by_port: Option<u64>, prefix: &str) -> Option<(Arc<crate::tunnel::outbound::Outbound>, u16)> {
        let nodes = self.nodes.lock().await;
        nodes
            .values()
            .find(|n| Some(n.local_port as u64) == by_port || n.link.starts_with(prefix))
            .and_then(|n| n.ob.clone().map(|ob| (ob, n.local_port)))
    }

    /// probe 后更新节点受限记忆 (同时清零失败计数)
    pub async fn mark_probe(&self, port: u16, restricted: bool, country: &str) {
        let mut nodes = self.nodes.lock().await;
        if let Some(n) = nodes.values_mut().find(|n| n.local_port == port) {
            n.restricted = restricted;
            n.country = country.to_string();
            n.fail_count = 0;
        }
    }

    /// 连接失败 → 降权 (best_outbound 排后)
    pub async fn mark_fail(&self, port: u16) {
        let mut nodes = self.nodes.lock().await;
        if let Some(n) = nodes.values_mut().find(|n| n.local_port == port) {
            n.fail_count = n.fail_count.saturating_add(1);
        }
    }

    /// 按本地端口取节点摘要 (协议 + 服务器主机)
    pub async fn node_brief(&self, port: u16) -> Option<(String, String)> {
        let nodes = self.nodes.lock().await;
        nodes.values().find(|n| n.local_port == port).map(|n| (n.scheme.clone(), link_host(&n.link)))
    }

    /// 批量导入: 一行一个链接。返回 (imported, skipped, errors)。
    pub async fn import_many(&self, text: &str) -> (Vec<Value>, usize, Vec<String>) {
        let mut imported = Vec::new();
        let mut skipped = 0usize;
        let mut errors = Vec::new();
        for line in text.lines() {
            let raw = line.trim();
            if raw.is_empty() || raw.starts_with('#') {
                continue;
            }
            let link = normalize_link(raw);
            let link = link.as_str();
            {
                let nodes = self.nodes.lock().await;
                if let Some(n) = nodes.get(link) {
                    if n.alive {
                        skipped += 1;
                        continue;
                    }
                }
            }
            match self.import(link).await {
                Ok((scheme, proxy)) => imported.push(json!({
                    "link": link,
                    "scheme": scheme,
                    "proxy": proxy,
                })),
                Err(e) => errors.push(format!("{}: {e}", trunc(link, 48))),
            }
        }
        (imported, skipped, errors)
    }

    pub async fn stop_node(&self, port: Option<u64>, link_prefix: &str) -> bool {
        let mut nodes = self.nodes.lock().await;
        let keys: Vec<String> = nodes
            .iter()
            .filter(|(_, n)| Some(n.local_port as u64) == port || n.link.starts_with(link_prefix))
            .map(|(k, _)| k.clone())
            .collect();
        let mut stopped = false;
        for k in keys {
            if let Some(n) = nodes.get(&k) {
                if let Some(pid) = n.child_id {
                    let _ = kill_pid(Some(pid));
                }
            }
            if let Some(n) = nodes.get_mut(&k) {
                n.alive = false;
            }
            stopped = true;
        }
        stopped
    }

    pub async fn state(&self) -> Vec<Value> {
        let mut nodes = self.nodes.lock().await;
        self.prune_dead(&mut nodes);
        nodes
            .values()
            .map(|n| {
                json!({
                    "link": trunc(&n.link, 64),
                    "full_link": n.link,
                    "scheme": n.scheme,
                    "local_port": n.local_port,
                    "alive": n.alive,
                    "restricted": n.restricted,
                    "country": n.country,
                    "host": link_host(&n.link),
                })
            })
            .collect()
    }
}

impl Default for Relay {
    fn default() -> Self {
        Self::new()
    }
}

fn trunc(s: &str, n: usize) -> String {
    if s.len() <= n { s.to_string() } else { format!("{}...", &s[..n]) }
}

fn node_process_alive(n: &RelayNode) -> bool {
    match n.child_id {
        Some(pid) => {
            // 兼容遗留字段: 外部进程模式
            let status = std::process::Command::new("sh")
                .arg("-c")
                .arg(format!("kill -0 {pid} 2>/dev/null && echo alive || echo dead"))
                .output();
            matches!(status, Ok(o) if String::from_utf8_lossy(&o.stdout).contains("alive"))
        }
        // 纯 Rust 进程内出站: 对象存在即可用 (真实连通性由测活判定)
        None => n.ob.is_some(),
    }
}

fn kill_pid(pid: Option<u32>) -> std::io::Result<()> {
    if let Some(pid) = pid {
        std::process::Command::new("sh")
            .arg("-c")
            .arg(format!("kill {pid} 2>/dev/null; true"))
            .output()?;
    }
    Ok(())
}

async fn wait_port(host: &str, port: u16, timeout_secs: u64) -> bool {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(timeout_secs);
    while std::time::Instant::now() < deadline {
        if tokio::net::TcpStream::connect((host, port)).await.is_ok() {
            return true;
        }
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    }
    false
}

// ---------------------------------------------------------------------------
// 分享链接 → sing-box outbound (七协议)
// ---------------------------------------------------------------------------

pub fn share_link_to_outbound(link: &str) -> Result<(String, Value), String> {
    let (scheme_raw, rest) = link.split_once("://").ok_or("missing scheme")?;
    let scheme = scheme_raw.to_ascii_lowercase();
    match scheme.as_str() {
        "vmess" => vmess_outbound(link).map(|o| ("vmess".into(), o)),
        "vless" => vless_outbound(rest).map(|o| ("vless".into(), o)),
        "trojan" | "tj" => trojan_outbound(rest).map(|o| ("trojan".into(), o)),
        "ss" => shadowsocks_outbound(rest).map(|o| ("ss".into(), o)),
        "hy2" | "hysteria2" => hysteria2_outbound(rest).map(|o| ("hy2".into(), o)),
        "tuic" => tuic_outbound(rest).map(|o| ("tuic".into(), o)),
        "socks" | "socks5" | "socks5h" => socks_outbound(rest).map(|o| ("socks5".into(), o)),
        "http" | "https" => http_outbound(rest).map(|o| (scheme.clone(), o)),
        other => Err(format!("unsupported scheme: {other}")),
    }
}

fn user_info(rest: &str) -> (String, Option<String>) {
    // rest = [user[:pass]@]host:port/path
    match rest.split_once('@') {
        Some((creds, _)) => match creds.split_once(':') {
            Some((u, p)) => (u.to_string(), Some(p.to_string())),
            None => (creds.to_string(), None),
        },
        None => (String::new(), None),
    }
}

fn host_port(rest: &str) -> (String, u16) {
    // 去掉 @ 前的 userinfo 与路径/查询/fragment
    let hostpart = rest.rsplit('@').next().unwrap_or(rest);
    let hostpart = hostpart.split(['/', '?', '#']).next().unwrap_or(hostpart);
    // IPv6 [::1]:443 简化处理
    match hostpart.rsplit_once(':') {
        Some((h, p)) => (h.trim_matches(|c| c == '[' || c == ']').to_string(), p.parse().unwrap_or(0)),
        None => (hostpart.to_string(), 0),
    }
}

fn vless_outbound(rest: &str) -> Result<Value, String> {
    let (user, _) = user_info(rest);
    let (server, port) = host_port(rest);
    if user.is_empty() || server.is_empty() || port == 0 {
        return Err("invalid vless link".into());
    }
    let mut out = json!({"type": "vless", "server": server, "server_port": port, "uuid": user});
    apply_transport_params(rest, &mut out);
    Ok(out)
}

fn trojan_outbound(rest: &str) -> Result<Value, String> {
    let (user, _) = user_info(rest);
    let (server, port) = host_port(rest);
    if user.is_empty() || server.is_empty() || port == 0 {
        return Err("invalid trojan link".into());
    }
    let mut out = json!({"type": "trojan", "server": server, "server_port": port, "password": user});
    apply_transport_params(rest, &mut out);
    Ok(out)
}

fn hysteria2_outbound(rest: &str) -> Result<Value, String> {
    let (user, _) = user_info(rest);
    let (server, port) = host_port(rest);
    if server.is_empty() || port == 0 {
        return Err("invalid hy2 link".into());
    }
    let mut out = json!({"type": "hysteria2", "server": server, "server_port": port});
    if !user.is_empty() {
        out["password"] = json!(user);
    }
    // ?insecure=1 → tls.insecure
    if rest.contains("insecure=1") || rest.contains("allowInsecure=1") {
        out["tls"] = json!({"enabled": true, "insecure": true});
    }
    Ok(out)
}

fn tuic_outbound(rest: &str) -> Result<Value, String> {
    let (user, _) = user_info(rest);
    let (server, port) = host_port(rest);
    if server.is_empty() || port == 0 {
        return Err("invalid tuic link".into());
    }
    Ok(json!({"type": "tuic", "server": server, "server_port": port, "uuid": user, "password": user}))
}

fn socks_outbound(rest: &str) -> Result<Value, String> {
    let (user, pass) = user_info(rest);
    let (server, port) = host_port(rest);
    if server.is_empty() || port == 0 {
        return Err("invalid socks link".into());
    }
    let mut out = json!({"type": "socks", "server": server, "server_port": port, "version": "5"});
    if !user.is_empty() {
        out["username"] = json!(user);
        if let Some(p) = pass {
            out["password"] = json!(p);
        }
    }
    Ok(out)
}

fn http_outbound(rest: &str) -> Result<Value, String> {
    let (user, pass) = user_info(rest);
    let (server, port) = host_port(rest);
    if server.is_empty() || port == 0 {
        return Err("invalid http link".into());
    }
    let mut out = json!({"type": "http", "server": server, "server_port": port});
    if !user.is_empty() {
        out["username"] = json!(user);
        if let Some(p) = pass {
            out["password"] = json!(p);
        }
    }
    Ok(out)
}

/// vless/trojan 链接的 query 传输参数 (ws/grpc 等, 对齐 sing-box outbound 字段)。
fn apply_transport_params(rest: &str, out: &mut Value) {
    let query = rest.split_once('?').map(|(_, q)| q).unwrap_or("");
    let query = query.split('#').next().unwrap_or(query);
    let mut params: HashMap<String, String> = HashMap::new();
    for kv in query.split('&') {
        if let Some((k, v)) = kv.split_once('=') {
            params.insert(k.to_string(), v.to_string());
        }
    }
    let vtype = params.get("type").map(String::as_str).unwrap_or("");
    match vtype {
        "ws" => {
            let mut transport = json!({"type": "ws"});
            if let Some(p) = params.get("path") {
                transport["path"] = json!(p);
            }
            if let Some(h) = params.get("host") {
                transport["headers"] = json!({"Host": h});
            }
            out["transport"] = transport;
        }
        "grpc" => {
            let mut transport = json!({"type": "grpc"});
            if let Some(sn) = params.get("serviceName") {
                transport["service_name"] = json!(sn);
            }
            out["transport"] = transport;
        }
        _ => {}
    }
    let security = params.get("security").map(String::as_str).unwrap_or("");
    if security == "tls" || vtype == "ws" && security.is_empty() && params.contains_key("sni") {
        let mut tls = json!({"enabled": true});
        if let Some(sni) = params.get("sni") {
            tls["server_name"] = json!(sni);
        }
        if params.get("allowInsecure").map(String::as_str) == Some("1") {
            tls["insecure"] = json!(true);
        }
        out["tls"] = tls;
    }
}

pub fn b64_url_decode_pub(input: &str) -> Result<Vec<u8>, String> {
    b64_url_decode(input)
}

/// URL 百分号解码 (无依赖简版)。
pub fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() + 1 && i + 2 < bytes.len() + 1 {
            if i + 2 < bytes.len() || i + 2 == bytes.len() {
                let hi = (bytes.get(i+1)).and_then(|b| (*b as char).to_digit(16));
                let lo = (bytes.get(i+2)).and_then(|b| (*b as char).to_digit(16));
                if let (Some(h), Some(l)) = (hi, lo) {
                    out.push((h * 16 + l) as u8);
                    i += 3;
                    continue;
                }
            }
            out.push(bytes[i]);
            i += 1;
        } else if bytes[i] == b'+' {
            out.push(b' ');
            i += 1;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8_lossy(&out).to_string()
}

/// t.me/socks 与 tg://socks 链接 → socks5://user:pass@host:port
pub fn normalize_link(raw: &str) -> String {
    let t = raw.trim();
    for prefix in ["https://t.me/socks?", "http://t.me/socks?", "tg://socks?", "tg://proxy?"] {
        if let Some(q) = t.strip_prefix(prefix) {
            let mut server = String::new();
            let mut port = String::new();
            let mut user = String::new();
            let mut pass = String::new();
            for kv in q.split('&') {
                let (k, v) = kv.split_once('=').unwrap_or((kv, ""));
                let v = percent_decode(v);
                match k {
                    "server" => server = v,
                    "port" => port = v,
                    "user" => user = v,
                    "pass" | "password" => pass = v,
                    "secret" => {}, // mtproto 代理暂不支持
                    _ => {},
                }
            }
            if !server.is_empty() && !port.is_empty() {
                if user.is_empty() {
                    return format!("socks5://{server}:{port}");
                }
                return format!("socks5://{}:{}@{}:{}", user, pass, server, port);
            }
        }
    }
    t.to_string()
}

fn b64_url_decode(input: &str) -> Result<Vec<u8>, String> {
    let mut s = input.trim().replace('-', "+").replace('_', "/");
    while s.len() % 4 != 0 {
        s.push('=');
    }
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(s.as_bytes())
        .map_err(|e| format!("base64: {e}"))
}

fn vmess_outbound(link: &str) -> Result<Value, String> {
    let body = link.split_once("://").map(|(_, r)| r).unwrap_or("");
    let body = body.split('#').next().unwrap_or(body);
    let decoded = b64_url_decode(body)?;
    let data: Value = serde_json::from_slice(&decoded).map_err(|e| format!("vmess json: {e}"))?;
    let port = data["port"].as_u64().unwrap_or(0);
    let security = data["scy"].as_str().unwrap_or("auto");
    Ok(json!({
        "type": "vmess",
        "server": data["add"],
        "server_port": port,
        "uuid": data["id"],
        "security": security,
        "alter_id": 0,
    }))
}

fn shadowsocks_outbound(rest: &str) -> Result<Value, String> {
    let body = rest.split(['?', '#']).next().unwrap_or(rest);
    if body.contains('@') {
        let (creds, hostpart) = body.split_once('@').ok_or("invalid ss")?;
        let creds = if creds.contains(':') {
            creds.to_string()
        } else {
            String::from_utf8(b64_url_decode(creds)?).map_err(|e| e.to_string())?
        };
        let (method, password) = creds.split_once(':').ok_or("invalid ss credentials")?;
        let (server, port) = host_port(hostpart);
        if server.is_empty() || port == 0 {
            return Err("invalid ss link".into());
        }
        Ok(json!({
            "type": "shadowsocks", "server": server, "server_port": port,
            "method": method, "password": password,
        }))
    } else {
        let decoded = String::from_utf8(b64_url_decode(body)?).map_err(|e| e.to_string())?;
        let (creds, hostpart) = decoded.split_once('@').ok_or("invalid ss")?;
        let (method, password) = creds.split_once(':').ok_or("invalid ss credentials")?;
        let (server, port) = host_port(hostpart);
        Ok(json!({
            "type": "shadowsocks", "server": server, "server_port": port,
            "method": method, "password": password,
        }))
    }
}

/// 从分享链接提取服务器主机 (隐藏端口与凭据)
fn link_host(link: &str) -> String {
    let rest = link.split("://").nth(1).unwrap_or(link);
    let after_at = rest.rsplit('@').next().unwrap_or(rest);
    let hostport = after_at.split('/').next().unwrap_or(after_at);
    let host = match hostport.rsplit_once(':') {
        Some((h, _)) if !h.contains(']') => h.to_string(),
        Some((h, _)) => h.trim_matches(['[', ']']).to_string(),
        None => hostport.to_string(),
    };
    host
}
