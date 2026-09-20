//! OAuth 设备码登录 (extract_freebuff.py 的 Rust 翻译) 与 token 文件管理。
//! 流程: POST /api/auth/cli/code → loginUrl + fingerprintHash →
//!       用户浏览器授权 → 轮询 GET /api/auth/cli/status → authToken。
use crate::upstream::{self, CODEBUFF_API};
use serde_json::Value;
use serde_json::json;
use std::path::PathBuf;

pub fn tokens_file() -> PathBuf {
    std::env::var("FREEBUFF_TOKENS_FILE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            crate::storage::path("freebuff_tokens.txt")
        })
}

pub fn load_tokens() -> Vec<String> {
    std::fs::read_to_string(tokens_file())
        .map(|s| s.lines().map(str::trim).filter(|l| l.len() > 8).map(String::from).collect())
        .unwrap_or_default()
}

pub fn save_token(token: &str) -> std::io::Result<PathBuf> {
    let path = tokens_file();
    let mut existing = load_tokens();
    if !existing.iter().any(|t| t == token) {
        existing.push(token.to_string());
    }
    std::fs::write(&path, existing.join("\n") + "\n")?;
    Ok(path)
}

pub async fn login() -> Result<String, String> {
    // 1) 申请授权码
    let fingerprint = uuid::Uuid::new_v4().to_string();
    let fingerprint = format!("enhanced-{:08x}{:08x}",
        crc32(fingerprint.as_bytes()),
        crc32(format!("{}2", fingerprint).as_bytes()));
    let resp = upstream::up_base(
        upstream::FREEBUFF_API,
        None,
        "POST",
        "/api/auth/cli/code",
        "",
        Some(&json!({"fingerprintId": fingerprint})),
        &[],
        std::time::Duration::from_secs(15),
    )
    .await
    .map_err(|e| e)?;
    let data = resp
        .json()
        .ok_or_else(|| format!("cli/code bad response: {} {}", resp.status, resp.text))?;
    // 兼容字段名: loginUrl / url, fingerprintHash / fingerprint_hash
    let login_url = data["loginUrl"]
        .as_str()
        .or(data["url"].as_str())
        .ok_or("missing loginUrl")?
        .to_string();
    let fp_hash = data["fingerprintHash"]
        .as_str()
        .or(data["fingerprint_hash"].as_str())
        .or(Some(fingerprint.as_str()))
        .unwrap()
        .to_string();
    let expires_at = data["expiresAt"].as_i64().unwrap_or(0);

    println!("=== Freebuff (codebuff) 账号授权 ===");
    println!("1. 在浏览器打开下面的链接并用 Google 账号登录:");
    println!();
    println!("   {login_url}");
    println!();
    println!("2. 授权完成后本命令会自动继续 (最长等 5 分钟)...");
    let _ = open_browser(&login_url);

    // 2) 轮询
    for i in 0..60 {
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        let status = upstream::up(
            "GET",
            &format!(
                "/api/auth/cli/status?fingerprintId={fingerprint}&fingerprintHash={fp_hash}&expiresAt={expires_at}"
            ),
            "",
            None,
            &[],
            std::time::Duration::from_secs(15),
        )
        .await
        .map_err(|e| e)?;
        if status.status == 200 {
            if let Some(token) = status
                .json()
                .and_then(|d| d["user"]["authToken"].as_str().map(String::from))
            {
                let path = save_token(&token).map_err(|e| e.to_string())?;
                println!();
                println!("✓ token 已获取并保存到 {}", path.display());
                let head: String = token.chars().take(12).collect();
                println!("  token: {head}...(共 {} 字符)", token.len());
                return Ok(token);
            }
        }
        if i % 6 == 5 {
            println!("   ...仍在等待授权 ({}s)", (i + 1) * 5);
        }
    }
    Err("授权超时 (5 分钟)".into())
}

fn open_browser(url: &str) -> Result<(), String> {
    // 尽力而为; 失败不影响流程 (用户手动打开)
    let _ = std::process::Command::new("sh")
        .arg("-c")
        .arg(format!("command -v xdg-open >/dev/null && xdg-open '{url}' &"))
        .spawn();
    Ok(())
}

fn crc32(data: &[u8]) -> u32 {
    let mut table = [0u32; 256];
    for (i, item) in table.iter_mut().enumerate() {
        let mut c = i as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
        }
        *item = c;
    }
    let mut crc = 0xFFFF_FFFFu32;
    for b in data {
        crc = table[((crc ^ *b as u32) & 0xFF) as usize] ^ (crc >> 8);
    }
    crc ^ 0xFFFF_FFFF
}

/// show: 探测全部 token 健康度 (GET /api/v1/me, 不耗额度)
pub async fn show_accounts() -> Result<Value, String> {
    let tokens = load_tokens();
    let mut details = Vec::new();
    for t in &tokens {
        let r = upstream::up("GET", "/api/v1/me", t, None, &[], std::time::Duration::from_secs(15)).await;
        let (alive, state) = match r {
            Ok(resp) if (200..300).contains(&resp.status) || resp.status == 404 => {
                let email = resp
                    .json()
                    .and_then(|d| d["user"]["email"].as_str().map(String::from))
                    .unwrap_or_default();
                (Some(true), format!("ok {email}"))
            }
            Ok(resp) => (Some(false), format!("status={}", resp.status)),
            Err(e) => (None, e),
        };
        let head: String = t.chars().take(10).collect();
        details.push(json!({"token": format!("{head}..."), "alive": alive, "state": state}));
    }
    Ok(json!({"accounts": details.len(), "details": details}))
}
