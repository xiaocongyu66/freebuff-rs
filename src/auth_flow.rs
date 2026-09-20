//! 在线授权流 (对齐原版 startAuthorization/pollAuthorization)。
//! POST /admin/auth/start → {flow_id, url}; 前端新窗口完成登录后
//! 轮询 GET /admin/auth/{flow_id} → pending/complete + 自动收录账号。
use crate::upstream;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct AuthFlow {
    pub fingerprint_id: String,
    pub fingerprint_hash: String,
    pub expires_at: i64,
    pub created_at: std::time::Instant,
    /// 该 flow 使用的上游基址 (freebuff.com / codebuff.com), 轮询必须同源
    pub base: String,
}

pub struct AuthFlows(pub Mutex<HashMap<String, AuthFlow>>);

impl AuthFlows {
    pub fn new() -> Self {
        AuthFlows(Mutex::new(HashMap::new()))
    }
}

impl Default for AuthFlows {
    fn default() -> Self {
        Self::new()
    }
}

/// 发起授权: 申请 cli/code 授权码, 存 flow, 返回 (flow_id, 授权页 URL)。
pub async fn start(flows: &AuthFlows, provider: &str) -> Result<Value, String> {
    let base = match provider {
        "codebuff" => upstream::CODEBUFF_API,
        _ => upstream::FREEBUFF_API,
    };
    let fingerprint_id = format!(
        "codebuff-cli-{}",
        uuid::Uuid::new_v4().simple().to_string()[..8].to_string()
    );
    let body = json!({"fingerprintId": fingerprint_id});
    let mut resp = Err(String::new());
    for _ in 0..3 {
        resp = upstream::up_base(
            base,
            None,
            "POST",
            "/api/auth/cli/code",
            "",
            Some(&body),
            &[],
            std::time::Duration::from_secs(15),
        )
        .await;
        if resp.is_ok() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(600)).await;
    }
    let resp = resp?;
    let data = resp
        .json()
        .ok_or_else(|| format!("cli/code bad response: {} {}", resp.status, resp.text))?;
    if !(200..300).contains(&resp.status) {
        return Err(format!("cli/code failed ({}): {}", resp.status, resp.text));
    }
    let url = data["loginUrl"]
        .as_str()
        .or(data["url"].as_str())
        .or(data["authUrl"].as_str())
        .ok_or("missing loginUrl")?
        .to_string();
    let flow = AuthFlow {
        fingerprint_id: fingerprint_id.clone(),
        fingerprint_hash: data["fingerprintHash"].as_str().unwrap_or("").to_string(),
        expires_at: data["expiresAt"].as_i64().unwrap_or(0),
        created_at: std::time::Instant::now(),
        base: base.to_string(),
    };
    let flow_id = uuid::Uuid::new_v4().simple().to_string();
    flows.0.lock().await.insert(flow_id.clone(), flow);
    Ok(json!({
        "flowId": flow_id,
        "url": url,
        "expiresAt": data["expiresAt"],
    }))
}

/// 轮询授权状态: 401=等待; 拿到 authToken 即收录账号并清理 flow。
pub async fn poll(
    flows: &AuthFlows,
    pool: &Arc<crate::pool::Pool>,
    flow_id: &str,
) -> Result<Value, String> {
    let flow = {
        let map = flows.0.lock().await;
        map.get(flow_id).map(|f| (f.fingerprint_id.clone(), f.fingerprint_hash.clone(), f.expires_at, f.base.clone()))
    };
    let Some((fp_id, fp_hash, expires_at, base)) = flow else {
        return Ok(json!({"status": "expired"}));
    };
    // flow 有效期 1 小时
    let resp = upstream::up_base(
        &base,
        None,
        "GET",
        &format!(
            "/api/auth/cli/status?fingerprintId={fp_id}&fingerprintHash={fp_hash}&expiresAt={expires_at}"
        ),
        "",
        None,
        &[],
        std::time::Duration::from_secs(15),
    )
    .await;
    match resp {
        Err(e) if e.contains("401") || e.contains("status 401") => {
            Ok(json!({"status": "pending"}))
        }
        Err(e) => Err(e),
        Ok(r) => {
            if r.status == 401 {
                return Ok(json!({"status": "pending"}));
            }
            let data = r.json().ok_or_else(|| {
                format!("status bad response ({}): {}", r.status, r.text)
            })?;
            if !(200..300).contains(&r.status) {
                return Err(format!("status failed ({}): {}", r.status, r.text));
            }
            let user = &data["user"];
            let Some(auth_token) = user["authToken"].as_str().map(String::from) else {
                return Ok(json!({"status": "pending"}));
            };
            // 收录账号: 内存池 + tokens 文件
            let source = if base.contains("freebuff.com") { "freebuff" } else { "codebuff" };
            pool.accounts.lock().unwrap().push(crate::pool::Account {
                token: auth_token.clone(),
                uid: user["id"].as_str().map(String::from),
                source: source.to_string(),
                alias: String::new(),
            });
            // 关键: 持久化来源, 否则重启回填默认 codebuff 会混池
            let mut cfg = crate::admin::load_config();
            cfg.account_sources.insert(auth_token.clone(), source.to_string());
            crate::admin::save_config(&cfg);
            let mut tokens = crate::creds::load_tokens();
            if !tokens.iter().any(|t| t == &auth_token) {
                tokens.push(auth_token);
                let _ = std::fs::write(
                    crate::creds::tokens_file(),
                    tokens.join("\n") + "\n",
                );
            }
            flows.0.lock().await.remove(flow_id);
            Ok(json!({
                "status": "complete",
                "account": {
                    "email": user["email"],
                    "name": user["name"],
                },
            }))
        }
    }
}
