//! Anthropic Messages 协议转换 (anthropicToChat + 响应回转)。
use crate::protocol;
use axum::response::Response;
use serde_json::{json, Value};

fn text_of(content: &Value) -> String {
    match content {
        Value::String(s) => s.clone(),
        Value::Array(parts) => parts
            .iter()
            .filter_map(|p| {
                if p["type"] == "text" {
                    p["text"].as_str().map(String::from)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("\n"),
        _ => String::new(),
    }
}

/// Anthropic 请求 → OpenAI chat 请求 (文本与 tool_use/tool_result; 多模态按官方路径转文本)。
pub fn to_chat(ab: &Value) -> Value {
    let mut messages: Vec<Value> = Vec::new();
    let system = ab["system"].as_str().map(String::from).unwrap_or_default();
    if !system.is_empty() {
        messages.push(json!({"role": "system", "content": system}));
    }
    for m in ab["messages"].as_array().unwrap_or(&vec![]).clone() {
        let role = m["role"].as_str().unwrap_or("user").to_string();
        match m["content"] {
            Value::String(ref text) => {
                messages.push(json!({"role": role, "content": text}));
            }
            Value::Array(ref parts) => {
                // tool_result → role:tool; 其余文本拼接
                let mut texts: Vec<String> = Vec::new();
                let mut tool_calls: Vec<Value> = Vec::new();
                for p in parts {
                    match p["type"].as_str() {
                        Some("tool_result") => {
                            messages.push(json!({
                                "role": "tool",
                                "tool_call_id": p["tool_use_id"].as_str().unwrap_or(""),
                                "content": text_of(&p["content"]),
                            }));
                        }
                        Some("tool_use") => {
                            tool_calls.push(json!({
                                "id": p["id"].as_str().unwrap_or(""),
                                "type": "function",
                                "function": {
                                    "name": p["name"].as_str().unwrap_or(""),
                                    "arguments": p["input"].clone(),
                                }
                            }));
                        }
                        Some("text") => {
                            if let Some(t) = p["text"].as_str() {
                                texts.push(t.to_string());
                            }
                        }
                        _ => {}
                    }
                }
                if !tool_calls.is_empty() {
                    messages.push(json!({"role": role, "content": null, "tool_calls": tool_calls}));
                } else if !texts.is_empty() {
                    messages.push(json!({"role": role, "content": texts.join("\n")}));
                }
            }
            _ => {}
        }
    }
    let mut chat = json!({"messages": messages});
    chat["model"] = ab["model"].clone();
    if ab["max_tokens"].is_u64() {
        chat["max_tokens"] = ab["max_tokens"].clone();
    }
    for k in ["temperature", "top_p"] {
        if !ab[k].is_null() {
            chat[k] = ab[k].clone();
        }
    }
    if let Some(stop) = ab["stop_sequences"].as_array() {
        if !stop.is_empty() {
            chat["stop"] = json!(stop);
        }
    }
    if ab["thinking"]["type"] == "enabled" {
        let budget = ab["thinking"]["budget_tokens"].as_i64().unwrap_or(0);
        chat["reasoning_effort"] = json!(if budget >= 16000 { "high" } else if budget >= 8000 { "medium" } else { "low" });
    }
    if let Some(tools) = ab["tools"].as_array() {
        if !tools.is_empty() {
            let mapped: Vec<Value> = tools
                .iter()
                .filter_map(|t| {
                    t["name"].as_str().map(|name| {
                        json!({"type": "function", "function": {
                            "name": name,
                            "description": t["description"].as_str().unwrap_or(""),
                            "parameters": t.get("input_schema").cloned().unwrap_or(json!({"type": "object", "properties": {}})),
                        }})
                    })
                })
                .collect();
            chat["tools"] = json!(mapped);
            match ab["tool_choice"]["type"].as_str() {
                Some("auto") => chat["tool_choice"] = json!("auto"),
                Some("any") => chat["tool_choice"] = json!("required"),
                Some("none") => chat["tool_choice"] = json!("none"),
                Some("tool") => {
                    if let Some(name) = ab["tool_choice"]["name"].as_str() {
                        chat["tool_choice"] = json!({"type": "function", "function": {"name": name}});
                    }
                }
                _ => {}
            }
        }
    }
    chat
}

pub fn text_response(model: &str, content: &str, pt: u64, ct: u64) -> Value {
    json!({
        "id": format!("msg_{:x}", protocol::now_secs()),
        "type": "message",
        "role": "assistant",
        "model": model,
        "content": [{"type": "text", "text": content}],
        "stop_reason": "end_turn",
        "stop_sequence": null,
        "usage": {"input_tokens": pt, "output_tokens": ct}
    })
}

pub fn stream_response(model: &str, content: &str, reasoning: &str, pt: u64, ct: u64) -> Response {
    let ts = protocol::now_secs();
    let mut events = String::new();
    let mut ev = |data: Value| {
        events.push_str(&format!("event: {}\ndata: {}\n\n", data["type"].as_str().unwrap_or(""), data));
    };
    ev(json!({"type": "message_start", "message": {"id": format!("msg_{ts:x}"), "type": "message", "role": "assistant", "model": model, "content": [], "usage": {"input_tokens": pt, "output_tokens": 0}}}));
    ev(json!({"type": "content_block_start", "index": 0, "content_block": {"type": "text", "text": ""}}));
    ev(json!({"type": "content_block_delta", "index": 0, "delta": {"type": "text_delta", "text": content}}));
    ev(json!({"type": "content_block_stop", "index": 0}));
    ev(json!({"type": "message_delta", "delta": {"stop_reason": "end_turn", "stop_sequence": null}, "usage": {"output_tokens": ct}}));
    ev(json!({"type": "message_stop"}));
    let _ = reasoning;
    Response::builder()
        .status(200)
        .header("content-type", "text/event-stream")
        .body(axum::body::Body::from(events))
        .unwrap()
}
