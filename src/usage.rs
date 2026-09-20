//! 用量统计 — SQLite (data_dir/usage.sqlite), 记录每次网关请求的
//! 模型/账号/状态/token/延迟/错误; 提供 summary(按小时聚合) 与 recent 明细。
use crate::storage;
use rusqlite::Connection;
use serde_json::{json, Value};
use std::sync::Mutex;

fn db_path() -> std::path::PathBuf {
    storage::path("usage.sqlite")
}

fn conn() -> Connection {
    let c = Connection::open(db_path()).expect("open usage.sqlite");
    c.execute_batch(
        "PRAGMA journal_mode=WAL;
         CREATE TABLE IF NOT EXISTS requests (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            ts INTEGER NOT NULL,
            model TEXT NOT NULL,
            source TEXT NOT NULL DEFAULT '',
            account_head TEXT NOT NULL DEFAULT '',
            stream INTEGER NOT NULL DEFAULT 0,
            status INTEGER NOT NULL,
            prompt_tokens INTEGER NOT NULL DEFAULT 0,
            completion_tokens INTEGER NOT NULL DEFAULT 0,
            latency_ms INTEGER NOT NULL DEFAULT 0,
            error TEXT NOT NULL DEFAULT ''
         );
         CREATE INDEX IF NOT EXISTS idx_requests_ts ON requests(ts);",
        );
    let _ = c.execute_batch("ALTER TABLE requests ADD COLUMN cached_tokens INTEGER NOT NULL DEFAULT 0;");
    c
}

/// 全局连接 (WAL 支持多读单写; 写入串行化)
static WRITER: Mutex<()> = Mutex::new(());

pub fn record(r: &Value) {
    let _g = WRITER.lock().unwrap();
    let c = conn();
    let _ = c.execute(
        "INSERT INTO requests (ts, model, source, account_head, stream, status, prompt_tokens, completion_tokens, latency_ms, error, cached_tokens)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
        rusqlite::params![
            r["ts"].as_i64().unwrap_or(0),
            r["model"].as_str().unwrap_or(""),
            r["source"].as_str().unwrap_or(""),
            r["account_head"].as_str().unwrap_or(""),
            r["stream"].as_bool().unwrap_or(false) as i64,
            r["status"].as_i64().unwrap_or(0),
            r["prompt_tokens"].as_i64().unwrap_or(0),
            r["completion_tokens"].as_i64().unwrap_or(0),
            r["latency_ms"].as_i64().unwrap_or(0),
            r["error"].as_str().unwrap_or(""),
            r["cached_tokens"].as_i64().unwrap_or(0),
        ],
    );
}

/// 汇总: 最近 hours 小时的 请求/token/错误/延迟分位/按模型分布
pub fn summary(hours: i64) -> Value {
    let _g = WRITER.lock().unwrap();
    let c = conn();
    let since = (crate::protocol::now_secs() as i64) * 1000 - hours * 3600 * 1000;
    let mut q = c
        .prepare(
            "SELECT status, latency_ms, prompt_tokens, completion_tokens, model, error, cached_tokens FROM requests WHERE ts > ?1",
        )
        .expect("q");
    let mut rows = q.query(rusqlite::params![since]).expect("q rows");
    let (mut total, mut errors, mut pt, mut ct, mut cached) = (0i64, 0i64, 0i64, 0i64, 0i64);
    let mut latencies: Vec<i64> = Vec::new();
    let mut by_model: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    while let Ok(Some(row)) = rows.next() {
        total += 1;
        let status: i64 = row.get(0).unwrap_or(0);
        let latency: i64 = row.get(1).unwrap_or(0);
        if !(200..300).contains(&status) {
            errors += 1;
        }
        pt += row.get::<_, i64>(2).unwrap_or(0);
        ct += row.get::<_, i64>(3).unwrap_or(0);
        cached += row.get::<_, i64>(6).unwrap_or(0);
        latencies.push(latency);
        let model: String = row.get(4).unwrap_or_default();
        *by_model.entry(model).or_insert(0) += 1;
    }
    latencies.sort_unstable();
    let pct = |p: f64| -> i64 {
        if latencies.is_empty() {
            0
        } else {
            latencies[((latencies.len() as f64 - 1.0) * p).round() as usize]
        }
    };
    let mut models: Vec<Value> = by_model
        .into_iter()
        .map(|(m, n)| json!({"model": m, "requests": n}))
        .collect();
    models.sort_by(|a, b| b["requests"].as_i64().cmp(&a["requests"].as_i64()));
    json!({
        "hours": hours,
        "requests": total,
        "errors": errors,
        "prompt_tokens": pt,
        "completion_tokens": ct,
        "cached_tokens": cached,
        "cache_hit_rate": if pt > 0 { (cached as f64 / pt as f64 * 100.0 * 10.0).round() / 10.0 } else { 0.0 },
        "latency_p50_ms": pct(0.5),
        "latency_p95_ms": pct(0.95),
        "by_model": models.into_iter().take(10).collect::<Vec<_>>(),
    })
}

/// 最近明细 limit 条 (倒序)
pub fn recent(limit: i64) -> Value {
    let _g = WRITER.lock().unwrap();
    let c = conn();
    let mut q = c
        .prepare(
            "SELECT ts, model, source, account_head, stream, status, prompt_tokens, completion_tokens, latency_ms, error
             FROM requests ORDER BY id DESC LIMIT ?1",
        )
        .expect("q");
    let rows = q
        .query_map(rusqlite::params![limit], |row| {
            Ok(json!({
                "ts": row.get::<_, i64>(0).unwrap_or(0),
                "model": row.get::<_, String>(1).unwrap_or_default(),
                "source": row.get::<_, String>(2).unwrap_or_default(),
                "account": row.get::<_, String>(3).unwrap_or_default(),
                "stream": row.get::<_, i64>(4).unwrap_or(0) == 1,
                "status": row.get::<_, i64>(5).unwrap_or(0),
                "prompt_tokens": row.get::<_, i64>(6).unwrap_or(0),
                "completion_tokens": row.get::<_, i64>(7).unwrap_or(0),
                "latency_ms": row.get::<_, i64>(8).unwrap_or(0),
                "error": row.get::<_, String>(9).unwrap_or_default(),
            }))
        })
        .expect("q rows");
    let items: Vec<Value> = rows.filter_map(|r| r.ok()).collect();
    json!({"items": items})
}
