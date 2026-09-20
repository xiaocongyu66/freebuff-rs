//! token 一键导入 — 粘贴任意混合文本自动抽取 token 入池:
//! 支持 curl(-H 'authorization: Bearer x') / HAR JSON(entries[].request.headers)
//! / Cookie 串 / 裸 Bearer / uuid 形状 token。去重(已在池的跳过)。
use serde_json::{json, Value};

pub fn extract_tokens(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();

    // 1) HAR JSON: entries[].request.headers[] 里 authorization/cookie
    if let Ok(v) = serde_json::from_str::<Value>(text) {
        if let Some(entries) = v["log"]["entries"].as_array() {
            for e in entries {
                for h in e["request"]["headers"].as_array().unwrap_or(&vec![]).iter() {
                    let name = h["name"].as_str().unwrap_or("").to_lowercase();
                    if name == "authorization" || name == "cookie" {
                        if let Some(val) = h["value"].as_str() {
                            harvest(val, &mut out);
                        }
                    }
                }
            }
            if !out.is_empty() {
                return out;
            }
        }
    }

    harvest(text, &mut out);
    out
}

fn harvest(text: &str, out: &mut Vec<String>) {
    fn push(out: &mut Vec<String>, t: &str) {
        let t = t.trim();
        let ok = (8..=200).contains(&t.len())
            && !t.chars().any(|c| c.is_whitespace())
            && t.chars().any(|c| c.is_ascii_alphanumeric());
        if ok && !out.iter().any(|x| x == t) {
            out.push(t.to_string());
        }
    }

    // Bearer xxx
    let lower = text.to_lowercase();
    let mut rest = lower.as_str();
    let mut offset = 0usize;
    while let Some(i) = rest.find("bearer ") {
        let abs = offset + i + 7;
        let tail = &text[abs.min(text.len())..];
        let end = tail
            .char_indices()
            .find(|(_, c)| c.is_whitespace() || *c == '\'' || *c == '"' || *c == ',')
            .map(|(j, _)| j)
            .unwrap_or(tail.len());
        push(out, &tail[..end]);
        rest = &rest[i + 7..];
        offset = abs;
    }
    // uuid 形状
    let bytes = text.as_bytes();
    let mut i = 0usize;
    while i + 36 <= bytes.len() {
        let seg = &text[i..i + 36];
        let b = seg.as_bytes();
        let is_uuid = b[8] == b'-'
            && b[13] == b'-'
            && b[18] == b'-'
            && b[23] == b'-'
            && seg.chars().enumerate().all(|(k, c)| [8, 13, 18, 23].contains(&k) || c.is_ascii_hexdigit());
        if is_uuid {
            push(out, seg);
            i += 36;
        } else {
            i += 1;
        }
    }
}

/// 导入: 未在池的候选 token 收录 (source 默认 freebuff — 项目主渠道)
pub fn import_into_pool(pool: &std::sync::Arc<crate::pool::Pool>, text: &str) -> Value {
    let candidates = extract_tokens(text);
    let mut imported = Vec::new();
    let mut skipped = 0usize;
    for t in candidates {
        let already = {
            let accs = pool.accounts.lock().unwrap();
            accs.iter().any(|a| a.token == t)
        };
        if already {
            skipped += 1;
            continue;
        }
        pool.accounts.lock().unwrap().push(crate::pool::Account {
            token: t.clone(),
            uid: None,
            source: "freebuff".into(),
        });
        imported.push(format!("{}...", &t[..8.min(t.len())]));
        let mut tokens = crate::creds::load_tokens();
        if !tokens.iter().any(|x| x == &t) {
            tokens.push(t.clone());
            let _ = std::fs::write(crate::creds::tokens_file(), tokens.join("\n") + "\n");
        }
        let mut cfg = crate::admin::load_config();
        cfg.account_sources.insert(t, "freebuff".into());
        crate::admin::save_config(&cfg);
    }
    json!({"imported": imported, "skipped": skipped, "total": pool.accounts.lock().unwrap().len()})
}
