//! 账号池: 轮换 + 冷却 + session 钉住 + 健康观测 (worker.js pickToken/recordAccountObservation 语义)。
use crate::upstream::{self, Session};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub token: String,
    pub uid: Option<String>,
    /// 授权来源: "freebuff" | "codebuff" — 元数据; 池子整体轮询不按 source 分裂
    pub source: String,
    /// 自定义命名 (两渠道账号均可命名, 空 = 显示 token 前缀)
    #[serde(default)]
    pub alias: String,
}

#[derive(Debug, Clone)]
pub struct Health {
    pub alive: Option<bool>,
    pub state: String,
    pub uid: Option<String>,
    pub checked_at: Instant,
    /// 健康评分 0-100: 成功+5 失败-15; pick 优先高分账号
    pub score: i32,
}

pub struct Pool {
    /// 实测可用模型集 (balance/probe 从 rateLimitsByModel 汇总)
    pub available_models: std::sync::Mutex<std::collections::BTreeSet<String>>,
    /// 模型额度耗尽: (token, model) → 冷却至 resetAt_ms — 撞额度切下一账号, 不连累同账号其他模型
    pub model_exhausted: std::sync::Mutex<HashMap<(String, String), i64>>,
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
                            alias: a["alias"].as_str().unwrap_or("").to_string(),
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
                alias: String::new(),
            },
            _ => Account { token: s.to_string(), uid: None, source: String::new(), alias: String::new() },
        })
        .filter(|a| a.token.len() > 8)
        .collect()
}

impl Pool {
    /// 设置账号别名 (两渠道通用)
    pub fn set_alias(&self, token_head: &str, alias: &str) -> bool {
        let mut accs = self.accounts.lock().unwrap();
        let mut hit = false;
        for a in accs.iter_mut() {
            if a.token.starts_with(token_head) {
                a.alias = alias.to_string();
                hit = true;
            }
        }
        hit
    }

    /// 记录某账号某模型额度耗尽 (至重置时刻)
    pub fn note_model_exhausted(&self, token: &str, model: &str, until_ms: i64) {
        self.model_exhausted
            .lock()
            .unwrap()
            .insert((token.to_string(), model.to_string()), until_ms);
    }

    /// 该账号该模型当前是否还有额度 (耗尽且未重置 → false)
    pub fn model_has_quota(&self, token: &str, model: &str) -> bool {
        let m = self.model_exhausted.lock().unwrap();
        match m.get(&(token.to_string(), model.to_string())) {
            Some(until) => upstream::now_ms() >= *until,
            None => true,
        }
    }

    /// 记录实测可用模型 (取并集)
    pub fn note_available_models(&self, models: &[String]) {
        let mut set = self.available_models.lock().unwrap();
        for m in models {
            if !m.is_empty() {
                set.insert(m.clone());
            }
        }
    }

    /// 当前实测可用模型 (空 = 未知, 调用方回退完整目录)
    pub fn available_models_list(&self) -> Vec<String> {
        self.available_models.lock().unwrap().iter().cloned().collect()
    }

    /// 降级选择: 同家族优先, 否则可用集第一个; 无可用集返回 None
    pub fn fallback_model(&self, requested: &str) -> Option<String> {
        let avail = self.available_models_list();
        if avail.is_empty() {
            return None;
        }
        // 请求的本来就在可用集 → 不降级 (先判这个, 否则同家族 find 会返回自己→假降级噪音)
        if avail.iter().any(|m| m == requested) {
            return None;
        }
        let fam = requested.split('/').next().unwrap_or("").to_string();
        if let Some(m) = avail.iter().find(|m| m.starts_with(&format!("{fam}/"))) {
            return Some(m.clone());
        }
        avail.first().cloned()
    }

    /// 账号来源 ("freebuff"|"codebuff"), 未知默认 codebuff
    pub fn source_of(&self, token: &str) -> &'static str {
        let accs = self.accounts.lock().unwrap();
        match accs.iter().find(|a| a.token == token).map(|a| a.source.as_str()) {
            Some("freebuff") => "freebuff",
            _ => "codebuff",
        }
    }

    pub fn new(tokens_env: &str, accounts_json: &str) -> Self {
        let init: std::collections::BTreeSet<String> =
            crate::models::FREE_TIER_MODELS.iter().map(|s| s.to_string()).collect();
        Self {
            model_exhausted: std::sync::Mutex::new(HashMap::new()),
            available_models: std::sync::Mutex::new(init),
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

    /// 活跃 session 快照: (token, model, instance_id) — 心跳刷新用
    pub fn all_sessions(&self) -> Vec<(String, String, String)> {
        let m = self.sessions.lock().unwrap();
        m.iter()
            .filter_map(|(k, s)| {
                k.split_once(':').map(|(t, mo)| (t.to_string(), mo.to_string(), s.instance_id.clone()))
            })
            .collect()
    }

    /// 临期 session (剩余 < within_ms) — 心跳只刷这些, 避免全量轮询触发上游限流
    pub fn expiring_sessions(&self, within_ms: i64) -> Vec<(String, String, String)> {
        let m = self.sessions.lock().unwrap();
        let now = upstream::now_ms();
        m.iter()
            .filter(|(_, s)| s.expires_at_ms - now < within_ms && s.expires_at_ms > now)
            .filter_map(|(k, s)| {
                k.split_once(':').map(|(t, mo)| (t.to_string(), mo.to_string(), s.instance_id.clone()))
            })
            .collect()
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
        let mut use_pool = if alive_pool.is_empty() {
            self.accounts.lock().unwrap().clone()
        } else {
            alive_pool
        };
        // 健康评分优先: 高分账号先被轮询到 (评分相同时保持原轮询序)
        {
            let h = self.health.lock().unwrap();
            use_pool.sort_by(|a, b| {
                let sa = h.get(&a.token).map(|x| x.score).unwrap_or(60);
                let sb = h.get(&b.token).map(|x| x.score).unwrap_or(60);
                sb.cmp(&sa)
            });
        }

        if let Some(model) = session_model {
            for acct in &use_pool {
                if self.in_cooldown(&acct.token) || !self.model_has_quota(&acct.token, model) {
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
            let quota_ok = session_model.map(|m| self.model_has_quota(&acct.token, m)).unwrap_or(true);
            if !self.in_cooldown(&acct.token) && quota_ok {
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
            score: 60,
        });
        entry.alive = Some(state == "ok");
        entry.state = state.to_string();
        if uid.is_some() {
            entry.uid = uid;
        }
        entry.checked_at = Instant::now();
        // 健康评分: 成功+5 失败-15, 限幅 0..=100
        entry.score = (entry.score + if state == "ok" { 5 } else { -15 }).clamp(0, 100);
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
                    "score": info.map(|i| i.score).unwrap_or(60),
                    "source": a.source,
                    "alias": a.alias,
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
