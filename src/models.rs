//! 模型注册表: 打包 JSON 兜底 (11 模型 + 池分类), 可选远程刷新。
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const BUNDLED_MODELS_JSON: &str = include_str!("../freebuff-models.json");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEntry {
    pub id: String,
    pub session: String,
    pub agent: String,
    pub upstream: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Pools {
    #[serde(default)]
    pub premium: Vec<String>,
    #[serde(default)]
    pub standard: Vec<String>,
    #[serde(default)]
    pub glm: Vec<String>,
}

static DYNAMIC: std::sync::OnceLock<std::sync::RwLock<Registry>> = std::sync::OnceLock::new();

/// 当前生效注册表: 官方真实列表优先, 打包 JSON 兜底
pub fn current() -> Registry {
    let cell = DYNAMIC.get_or_init(|| std::sync::RwLock::new(Registry::bundled()));
    cell.read().unwrap().clone()
}

pub fn set_dynamic(r: Registry) {
    let cell = DYNAMIC.get_or_init(|| std::sync::RwLock::new(Registry::bundled()));
    *cell.write().unwrap() = r;
}

/// 拉取官方真实模型表 (发布 JSON), 成功即替换全局
pub async fn refresh_real_models() -> Result<usize, String> {
    const SOURCES: [&str; 3] = [
        "https://github.com/pingmike2/freebuff2api-wokers/releases/latest/download/freebuff-models.json",
        "https://cdn.jsdelivr.net/gh/pingmike2/freebuff2api-wokers@main/freebuff-models.json",
        "https://raw.githubusercontent.com/CodebuffAI/freebuff/main/common/src/constants/freebuff-models.ts",
    ];
    for url in SOURCES {
        let resp = reqwest::get(url).await;
        let Ok(resp) = resp else { continue };
        if !resp.status().is_success() { continue; }
        let text = resp.text().await.unwrap_or_default();
        // 发布 JSON 或 TS 常量 (TS 源解析失败则跳过, 由 JSON 源兜底)
        if let Some(r) = Registry::from_real_json(&text) {
            let n = r.models.len();
            set_dynamic(r);
            return Ok(n);
        }
    }
    Err("all model sources failed".into())
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Registry {
    #[serde(default)]
    pub models: Vec<ModelEntry>,
    #[serde(default)]
    pub pools: Pools,
}

impl Registry {
    pub fn bundled() -> Self {
        serde_json::from_str(BUNDLED_MODELS_JSON).expect("bundled freebuff-models.json")
    }

    /// 官方发布 JSON → 注册表 (真实模型列表; 结构: {models:[{id,session,agent,...}],pools})
    pub fn from_real_json(text: &str) -> Option<Self> {
        let v: Value = serde_json::from_str(text).ok()?;
        let models: Vec<ModelEntry> = v["models"].as_array()?
            .iter()
            .filter_map(|m| {
                Some(ModelEntry {
                    id: m["id"].as_str()?.to_string(),
                    session: m["session"].as_str().or(m["id"].as_str())?.to_string(),
                    agent: m["agent"].as_str().or(m["root_agent"].as_str())?.to_string(),
                    upstream: m["upstream"].as_str().unwrap_or("").to_string(),
                })
            })
            .collect();
        if models.is_empty() {
            return None;
        }
        Some(Self {
            models,
            pools: serde_json::from_value(v["pools"].clone()).unwrap_or_default(),
        })
    }

    pub fn find(&self, id: &str) -> Option<&ModelEntry> {
        // 精确 id / 上游名 / 短名(最后一段) 三种匹配
        self.models.iter().find(|m| {
            m.id == id || m.upstream == id || m.id.rsplit('/').next() == Some(id)
        })
    }

    pub fn default_model(&self) -> &str {
        "deepseek/deepseek-v4-flash"
    }

    pub fn list_ids(&self) -> Vec<String> {
        self.models.iter().map(|m| m.id.clone()).collect()
    }
}

/// reasoning effort 档位梯子与 clamp-down (官方 clampReasoningEffort 语义)。
const EFFORT_LADDER: [&str; 7] = ["minimal", "low", "medium", "high", "xhigh", "max", "ultra"];

pub fn clamp_effort(allowed: &[&str], requested: &str) -> String {
    if allowed.is_empty() || allowed.contains(&requested) {
        return requested.to_string();
    }
    let req_rank = EFFORT_LADDER.iter().position(|e| *e == requested);
    let best = allowed
        .iter()
        .filter(|a| EFFORT_LADDER.contains(&a.as_ref()))
        .min_by_key(|a| {
            let rank = EFFORT_LADDER.iter().position(|e| *e == **a).unwrap_or(0);
            match req_rank {
                Some(r) => rank.abs_diff(r),
                None => rank,
            }
        })
        .cloned()
        .unwrap_or("medium");
    best.to_string()
}

pub fn effort_allowed(model_id: &str) -> Vec<&'static str> {
    // 官方 efforts 表 (worker.js normalizeReasoningEffort 源, 2026-08-12):
    match model_id.rsplit('/').next().unwrap_or(model_id) {
        // 桌面端 orchestrator 逆向: 无 efforts 字段 → 请求侧自动剥离
        "solar-pro4" | "kimi-k3-eco" => vec![],
        "muse-spark" => vec!["minimal", "low", "medium", "high", "xhigh"],
        "deepseek-v4-flash" => vec!["low", "medium", "high"],
        "mimo-v2.5" => vec!["low", "medium", "high", "xhigh"],
        "deepseek-v4-pro" | "minimax-m3" | "gpt-5.6-luna" => {
            vec!["minimal", "low", "medium", "high", "xhigh"]
        }
        "glm-5.2" | "glm-5.3-flash" => vec!["low", "medium", "high"],
        _ => vec!["low", "medium", "high", "xhigh"],
    }
}
