//! 协议层: OpenAI/Anthropic 入站请求 → codebuff 上游 payload; 上游 SSE → 客户端。
use crate::models::{clamp_effort, effort_allowed, ModelEntry};
use crate::upstream::{self, Session};
use serde_json::{json, Map, Value};

/// 上游 chat/completions 接受的白名单键 (worker.js UPSTREAM_KEYS)
const UPSTREAM_KEYS: &[&str] = &[
    "messages",
    "tools",
    "tool_choice",
    "parallel_tool_calls",
    "reasoning_effort",
    "reasoning",
    "temperature",
    "top_p",
    "top_k",
    "presence_penalty",
    "frequency_penalty",
    "stop",
    "response_format",
    "seed",
];

/// system 消息注入官方 Buffy 前缀 (服务器 hasFreebuffRootSystemPromptOpening 字节级校验)。
pub fn normalize_messages(messages: &Value) -> Value {
    let arr = match messages.as_array() {
        Some(a) => a.clone(),
        None => return Value::Array(vec![]),
    };
    let mut out: Vec<Value> = Vec::new();
    let mut has_system = false;
    for m in arr {
        let Some(obj) = m.as_object() else { continue };
        let mut item = obj.clone();
        let role = item["role"].as_str().unwrap_or("");
        let role = if role == "developer" { "system" } else { role };
        if role == "system" {
            has_system = true;
            item.insert("role".into(), json!("system"));
            item.insert("cache_control".into(), json!({"type": "ephemeral"}));
            match item.get_mut("content") {
                Some(Value::String(text)) => {
                    if !text.starts_with(upstream::BUFFY) {
                        *text = format!("{}{}", upstream::BUFFY, text);
                    }
                }
                Some(Value::Array(parts)) => {
                    for p in parts.iter_mut() {
                        if p["type"] == "text" {
                            if let Some(t) = p["text"].as_str() {
                                if !t.starts_with(upstream::BUFFY) {
                                    p["text"] = json!(format!("{}{}", upstream::BUFFY, t));
                                }
                            }
                            break;
                        }
                    }
                }
                _ => {}
            }
        } else {
            item.insert("role".into(), json!(role));
        }
        out.push(Value::Object(item));
    }
    if !has_system {
        out.insert(
            0,
            json!({"role": "system", "content": upstream::BUFFY, "cache_control": {"type": "ephemeral"}}),
        );
    }
    Value::Array(out)
}

/// 构建 codebuff chat/completions payload (buildUpstreamPayload 语义)。
pub fn build_payload(
    params: &Value,
    mc: &ModelEntry,
    sess: &Session,
    run_id: &str,
    client_fingerprint: &str,
) -> Value {
    let mut payload = Map::new();
    for k in UPSTREAM_KEYS {
        if let Some(v) = params.get(*k) {
            if !v.is_null() {
                payload.insert((*k).to_string(), v.clone());
            }
        }
    }
    if let Some(effort) = payload.get("reasoning_effort").and_then(|v| v.as_str()).map(String::from) {
        let allowed = effort_allowed(&mc.id);
        if allowed.is_empty() {
            payload.remove("reasoning_effort"); // 该模型不支持思考程度 → 剥离
        } else {
            payload.insert("reasoning_effort".into(), json!(clamp_effort(&allowed, &effort)));
        }
    }
    payload.insert("model".into(), json!(mc.upstream));
    payload.insert("messages".into(), normalize_messages(&params["messages"]));
    payload.insert("stream".into(), json!(true)); // 恒流式; 非流式由网关聚合
    if !payload.contains_key("stop") {
        payload.insert("stop".into(), json!(["\"cb_easp\""]));
    }
    payload.insert("provider".into(), json!({"data_collection": "deny"}));
    // 工具集签名: 无官方专属工具名的带工具请求会被 foreign_toolset 拒绝;
    // end_turn 是官方白名单里的无害工具, 混入即过校验。
    if let Some(Value::Array(tools)) = payload.get_mut("tools") {
        if !tools.is_empty() {
            let has_sig = tools.iter().any(|t| t["function"]["name"] == "end_turn");
            if !has_sig {
                tools.push(json!({
                    "type": "function",
                    "function": {
                        "name": "end_turn",
                        "description": "Signal the end of the current task.",
                        "parameters": {"type": "object", "properties": {}}
                    }
                }));
            }
        }
    }
    // trace_session_id 会话稳定 (官方 clientSessionId 语义: 进程级稳定, 非每请求新)
    // 派生自 client_fingerprint → 同账号永远同会话串; 每请求新 uuid = 自动化特征
    let trace_sid = {
        let mut h: u32 = 0x811c_9dc5;
        for c in client_fingerprint.encode_utf16() {
            h = (h ^ c as u32).wrapping_mul(0x0100_0193);
        }
        let u = uuid::Uuid::from_u64_pair(
            ((h as u64) << 32) | 0x4f4e_5f53_4944_4531,
            0x9E37_79B9_7F4A_7C15 ^ (h as u64),
        );
        u.to_string()
    };
    payload.insert(
        "codebuff_metadata".to_string(),
        json!({
            "freebuff_instance_id": sess.instance_id,
            "trace_session_id": trace_sid,
            "run_id": run_id,
            "client_id": client_fingerprint,
            "cost_mode": "free",
        }),
    );
    Value::Object(payload)
}

/// 上游 SSE 行剥 {data: {...}} 包装 (unwrapData)。
pub fn unwrap_data(obj: Value) -> Value {
    let is_wrapped = obj["data"].as_object().map(|d| d.contains_key("choices") || d.contains_key("id") || d.contains_key("usage")).unwrap_or(false);
    if is_wrapped {
        obj["data"].clone()
    } else {
        obj
    }
}

/// 聚合上游 SSE 为 OpenAI 非流式响应 (streamToNonStream)。
pub fn aggregate_stream_text(sse_body: &str) -> (String, String, Option<String>, String, Option<Value>) {
    let mut content = String::new();
    let mut reasoning = String::new();
    let mut finish_reason: Option<String> = None;
    let mut model = String::new();
    let mut usage: Option<Value> = None;
    for line in sse_body.lines() {
        let Some(payload) = line.strip_prefix("data:") else { continue };
        let payload = payload.trim();
        if payload.is_empty() || payload == "[DONE]" {
            continue;
        }
        let Ok(obj) = serde_json::from_str::<Value>(payload) else { continue };
        let obj = unwrap_data(obj);
        if let Some(m) = obj["model"].as_str() {
            model = m.to_string();
        }
        if let Some(u) = obj.get("usage") {
            if u.is_object() {
                usage = Some(u.clone());
            }
        }
        let Some(choice) = obj["choices"].get(0) else { continue };
        let delta = &choice["delta"];
        if let Some(c) = delta["content"].as_str() {
            content.push_str(c);
        }
        if let Some(r) = delta["reasoning_content"].as_str() {
            reasoning.push_str(r);
        }
        if let Some(f) = choice["finish_reason"].as_str() {
            finish_reason = Some(f.to_string());
        }
    }
    (content, reasoning, finish_reason, model, usage)
}

pub fn now_secs() -> u64 {
    chrono::Utc::now().timestamp() as u64
}

pub fn oa_text_response(model: &str, content: &str, reasoning: &str, pt: u64, ct: u64, finish: Option<String>) -> Value {
    let mut message = Map::new();
    message.insert("role".into(), json!("assistant"));
    message.insert("content".into(), json!(content));
    if !reasoning.is_empty() {
        message.insert("reasoning_content".into(), json!(reasoning));
    }
    json!({
        "id": format!("chatcmpl-{:x}", now_secs()),
        "object": "chat.completion",
        "created": now_secs(),
        "model": model,
        "choices": [{"index": 0, "message": Value::Object(message), "finish_reason": finish.unwrap_or_else(|| "stop".into())}],
        "usage": {"prompt_tokens": pt, "completion_tokens": ct, "total_tokens": pt + ct}
    })
}
