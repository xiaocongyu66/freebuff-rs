//! 账号池: 轮换 + 冷却 + session 钉住 + 健康观测 (worker.js pickToken/recordAccountObservation 语义)。
use crate::upstream::{self, Session};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct Account {
    pub token: String,
    pub uid: Option<String>,
    /// 授权来源: "freebuff" | "codebuff" — 两个账号池分开
    pub source: String,
}

#[derive(Debug, Clone)]
pub struct Health {
    pub alive: Option<bool>,
    pub state: String,
    pub uid: Option<String>,
    pub checked_at: Instant,
}

pub struct Pool {
    pub accounts: std::sync::Mutex<Vec<Account>>,
    pub idx: AtomicUsize,
    /// token -> cooldown until (ms)
    pub cooldowns: Mutex<HashMap<String, i64>>,
    /// token:session_model -> session
    pub sessions: Mutex<HashMap<String, Session>>,
    /// token -> health
    pub health: Mutex<HashMap<String, Health>>,
}

pub fn parse_accounts(tokens_env: &str, accounts_json: &str) -> Vec<Account> {
    if !accounts_json.trim().is_empty() {
        if let Ok(v) = serde_json::from_str::<Value>(accounts_json) {
            if let Some(arr) = v.as_array() {
                let out: Vec<Account> = arr
                    .iter()
                    .filter_map(|a| {
                        let token = a["token"].as_str()?.trim().to_string();
                        if token.is_empty() {
                            return None;
                        }
                        Some(Account {
                            token,
                            uid: a["uid"].as_str().map(String::from),
                            source: a["source"].as_str().unwrap_or("").to_string(),
                        })
                    })
                    .collect();
                if !out.is_empty() {
                    return out;
                }
            }
        }
    }
    tokens_env
        .split([',', '\n'])
        .map(str::trim)
        .filter(|s| s.len() > 8)
        .map(|s| match s.find(':') {
            Some(i) if i > 0 => Account {
                token: s[..i].trim().to_string(),
                uid: Some(s[i + 1..].trim().to_string()).filter(|u| !u.is_empty()),
                source: String::new(),
            },
            _ => Account { token: s.to_string(), uid: None, source: String::new() },
        })
        .filter(|a| a.token.len() > 8)
        .collect()
}

impl Pool {
    /// 账号来源 ("freebuff"|"codebuff"), 未知默认 codebuff
    pub fn source_of(&self, token: &str) -> &'static str {
        let accs = self.accounts.lock().unwrap();
        match accs.iter().find(|a| a.token == token).map(|a| a.source.as_str()) {
            Some("freebuff") => "freebuff",
            _ => "codebuff",
        }
    }

    pub fn new(tokens_env: &str, accounts_json: &str) -> Self {
        Self {
            accounts: std::sync::Mutex::new(parse_accounts(tokens_env, accounts_json)),
            idx: AtomicUsize::new(0),
            cooldowns: Mutex::new(HashMap::new()),
            sessions: Mutex::new(HashMap::new()),
            health: Mutex::new(HashMap::new()),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.accounts.lock().unwrap().is_empty()
    }

    pub fn session_key(token: &str, model: &str) -> String {
        format!("{token}:{model}")
    }

    pub fn cached_session(&self, token: &str, model: &str) -> Option<Session> {
        let m = self.sessions.lock().unwrap();
        let s = m.get(&Self::session_key(token, model))?;
        upstream::is_usable_session(&Some(s.clone())).then(|| s.clone())
    }

    pub fn store_session(&self, token: &str, model: &str, s: Session) {
        self.sessions
            .lock()
            .unwrap()
            .insert(Self::session_key(token, model), s);
    }

    pub fn drop_session(&self, token: &str, model: &str) {
        self.sessions.lock().unwrap().remove(&Self::session_key(token, model));
    }

    pub fn cooldown(&self, token: &str, ms: i64) {
        if ms > 0 {
            self.cooldowns.lock().unwrap().insert(token.to_string(), upstream::now_ms() + ms);
        }
    }

    fn in_cooldown(&self, token: &str) -> bool {
        self.cooldowns
            .lock()
            .unwrap()
            .get(token)
            .map(|t| *t > upstream::now_ms())
            .unwrap_or(false)
    }

    /// pickToken: 优先钉住已有活跃 session 的号; 否则轮询跳冷却; 全冷却取最早到期。
    pub fn pick(&self, session_model: Option<&str>) -> Option<Account> {
        let alive_pool: Vec<Account> = self
            .accounts
            .lock()
            .unwrap()
            .iter()
            .filter(|a| {
                self.health
                    .lock()
                    .unwrap()
                    .get(&a.token)
                    .map(|h| h.alive != Some(false))
                    .unwrap_or(true)
            })
            .cloned()
            .collect();
        let use_pool = if alive_pool.is_empty() {
            self.accounts.lock().unwrap().clone()
        } else {
            alive_pool
        };

        if let Some(model) = session_model {
            for acct in &use_pool {
                if self.in_cooldown(&acct.token) {
                    continue;
                }
                if self.cached_session(&acct.token, model).is_some() {
                    return Some(acct.clone());
                }
            }
        }
        let start = self.idx.fetch_add(1, Ordering::Relaxed);
        for k in 0..use_pool.len() {
            let acct = &use_pool[(start + k) % use_pool.len()];
            if !self.in_cooldown(&acct.token) {
                return Some(acct.clone());
            }
        }
        let oldest = self
            .cooldowns
            .lock()
            .unwrap()
            .iter()
            .min_by_key(|(_, t)| **t)
            .map(|(t, _)| t.clone());
        if let Some(t) = oldest {
            self.cooldowns.lock().unwrap().remove(&t);
            return use_pool.into_iter().find(|a| a.token == t);
        }
        use_pool.first().cloned()
    }

    /// recordAccountObservation: 只记真实业务请求观察到的结果。
    pub fn observe(&self, token: &str, status: u16, body: &str) {
        let data: Value = serde_json::from_str(body).unwrap_or(Value::Null);
        let upstream_state = data["status"].as_str().or(data["state"].as_str()).unwrap_or("");
        let state = if status == 404 {
            "ok"
        } else if matches!(upstream_state, "banned" | "country_blocked" | "rate_limited" | "model_locked" | "ip_capped") {
            upstream_state
        } else if (200..300).contains(&status) {
            "ok"
        } else if status == 401 {
            "token_invalid"
        } else if status == 403 {
            if upstream_state == "banned" {
                "banned"
            } else if upstream_state == "country_blocked" {
                "country_blocked"
            } else {
                "blocked"
            }
        } else if status == 429 {
            "rate_limited"
        } else {
            return;
        };
        let uid = data["uid"].as_str().map(String::from);
        let mut h = self.health.lock().unwrap();
        let entry = h.entry(token.to_string()).or_insert_with(|| Health {
            alive: None,
            state: "unknown".into(),
            uid: None,
            checked_at: Instant::now(),
        });
        entry.alive = Some(state == "ok");
        entry.state = state.to_string();
        if uid.is_some() {
            entry.uid = uid;
        }
        entry.checked_at = Instant::now();
    }

    pub fn health_summary(&self) -> Value {
        let h = self.health.lock().unwrap();
        let accounts = self.accounts.lock().unwrap();
        let details: Vec<Value> = accounts
            .iter()
            .map(|a| {
                let info = h.get(&a.token);
                json!({
                    "token": format!("{}...", &a.token[..a.token.len().min(8)]),
                    "alive": info.map(|i| i.alive),
                    "state": info.map(|i| i.state.as_str()).unwrap_or("unknown"),
                    "source": a.source,
                })
            })
            .collect();
        let alive = details.iter().filter(|d| d["alive"] == json!(true)).count();
        let unhealthy = details.iter().filter(|d| d["alive"] == json!(false)).count();
        let unknown = details.len() - alive - unhealthy;
        let status = if accounts.is_empty() || (alive == 0 && unhealthy + unknown > 0) {
            "critical"
        } else if unhealthy + unknown > 0 {
            "degraded"
        } else {
            "ok"
        };
        json!({
            "status": status,
            "accounts": accounts.len(),
            "alive_accounts": alive,
            "unknown_accounts": unknown,
            "account_details": details,
        })
    }
}
