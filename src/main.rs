//! freebuff-rs: codebuff 免费模型 → OpenAI/Anthropic 兼容网关 (Rust 重写 freebuff2api)。
//! 子命令: serve(默认) / login / show / chat <model> <msg>
mod admin;
mod auth_flow;
mod semaphore;
mod storage;
mod tunnel_client;
mod import;
mod usage;
mod anthropic;
mod creds;
mod gateway;
mod models;
mod pool;
mod protocol;
mod relay;
mod server;
mod tunnel;
mod upstream;

use pool::Pool;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(String::as_str).unwrap_or("serve");
    match cmd {
        "login" => {
            if let Err(e) = creds::login().await {
                eprintln!("login failed: {e}");
                std::process::exit(1);
            }
        }
        "show" => match creds::show_accounts().await {
            Ok(v) => println!("{}", serde_json::to_string_pretty(&v).unwrap()),
            Err(e) => eprintln!("show failed: {e}"),
        },
        "serve" => serve().await,
        "chat" => chat_cli(&args).await,
        "tunnelprobe" => tunnelprobe(&args).await,
        other => {
            eprintln!("unknown command: {other}\nusage: freebuff-rs [serve|login|show|chat MODEL MESSAGE]");
            std::process::exit(2);
        }
    }
}

/// 直连通道探针: tunnelprobe <share-link> — 经隧道拨号请求 codebuff /healthz
async fn tunnelprobe(args: &[String]) {
    let link = args.get(2).expect("usage: tunnelprobe <share-link>").clone();
    let relay = Arc::new(relay::Relay::new());
    tunnel_client::set_relay(relay.clone());
    let (scheme, proxy) = relay.import(&link).await.expect("import failed");
    std::env::set_var("FREEBUFF_PROXY", &proxy);
    let (ob, p) = tunnel_client::tunnel_outbound().await.expect("no outbound");
    eprintln!("[probe] {scheme} via :{p}");
    let t0 = std::time::Instant::now();
    match tunnel_client::request(ob, "www.codebuff.com", "GET", "/healthz", "", &[], None, 20).await {
        Ok((st, body)) => {
            let text = http_body_util::BodyExt::collect(body)
                .await
                .map(|c| String::from_utf8_lossy(&c.to_bytes()).to_string())
                .unwrap_or_default();
            println!("status={st} elapsed={:?} body={}", t0.elapsed(), &text[..text.len().min(120)]);
        }
        Err(e) => println!("failed: {e}"),
    }
}

async fn serve() {
    let tokens_env = std::env::var("FREEBUFF_TOKEN").unwrap_or_default();
    let accounts_json = std::env::var("FREEBUFF_ACCOUNTS").unwrap_or_default();
    // env 未配时回退 tokens 文件 (login 写入)
    let tokens_env = if tokens_env.trim().is_empty() && accounts_json.trim().is_empty() {
        creds::load_tokens().join(",")
    } else {
        tokens_env
    };
    let pool = Arc::new(Pool::new(&tokens_env, &accounts_json));
    let registry = models::current();
    let port: u16 = std::env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8787);
    let api_key = std::env::var("FREEBUFF_API_KEY").ok().filter(|k| !k.is_empty());

    if pool.is_empty() {
        eprintln!("[freebuff-rs] 警告: 无账号 token。先运行 `freebuff-rs login` 完成授权。");
    } else {
        {
        // 回填账号来源 (旧 token 默认 codebuff — 原工具走 codebuff 授权)
        let sources = admin::load_config().account_sources;
        let mut accs = pool.accounts.lock().unwrap();
        for a in accs.iter_mut() {
            if a.source.is_empty() {
                a.source = sources.get(&a.token).cloned().unwrap_or_else(|| "codebuff".into());
            }
        }
    }
    println!("[freebuff-rs] {} 个账号已加载", pool.accounts.lock().unwrap().len());
    }
    println!("[freebuff-rs] {} 个模型可用", registry.list_ids().len());
    println!("[freebuff-rs] listening on 0.0.0.0:{port}");
    // 真实模型表: 启动拉取 + 6h 刷新 (失败沿用打包兜底)
    tokio::spawn(async move {
        loop {
            match models::refresh_real_models().await {
                Ok(n) => eprintln!("[models] 真实模型表已加载: {n} 个"),
                Err(e) => eprintln!("[models] 官方源拉取失败, 沿用打包列表: {e}"),
            }
            tokio::time::sleep(std::time::Duration::from_secs(6 * 3600)).await;
        }
    });

    let logbus = Arc::new(admin::LogBus::new());
    let gateway_keys = Arc::new(std::sync::Mutex::new(Vec::new()));
    let state = server::AppState {
        pool: pool.clone(),
        registry,
        api_key,
        sem: Arc::new(semaphore::TieredSemaphore::for_accounts(pool.accounts.lock().unwrap().len())),
        gateway_keys: Arc::clone(&gateway_keys),
        logbus: Arc::clone(&logbus),
    };
    let relay = Arc::new(relay::Relay::new());
    tunnel_client::set_relay(relay.clone());
    let auth_flows = Arc::new(auth_flow::AuthFlows::new());
    let saved_links: Vec<String> = {
        let cfg = admin::load_config();
        cfg.saved_links.clone()
    };
    // 会话保活心跳 (45s 刷新活跃 session, 失效删缓存)
    {
        let pool_hb = std::sync::Arc::clone(&pool);
        tokio::spawn(async move { gateway::session_heartbeat(pool_hb).await });
    }
    let cfg0 = admin::load_config();
    // 回填账号别名 (两渠道通用)
    {
        let aliases = cfg0.account_alias.clone();
        if !aliases.is_empty() {
            let mut accs = pool.accounts.lock().unwrap();
            for a in accs.iter_mut() {
                if let Some(al) = aliases.get(&a.token) {
                    a.alias = al.clone();
                }
            }
        }
    }
    *gateway_keys.lock().unwrap() = cfg0.keys.clone();
    let admin_state = Arc::new(admin::AdminState {
        pool: std::sync::Arc::clone(&pool),
        config: std::sync::Mutex::new(cfg0),
        relay: relay.clone(),
        auth_flows,
        logbus: Arc::clone(&logbus),
        gateway_keys: Arc::clone(&gateway_keys),
    });
    // 回填节点受限记忆 (端口 → 已测出的 restricted/country)
    {
        let restrictions = admin::load_config().node_restriction;
        if !restrictions.is_empty() {
            let relay_r = relay.clone();
            let n = restrictions.len();
            tokio::spawn(async move {
                // 等节点导入完成
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                for (port, (restricted, country)) in restrictions {
                    relay_r.mark_probe(port, restricted, &country).await;
                }
                eprintln!("[relay] 受限记忆已回填: {n} 条");
            });
        }
    }
    if !saved_links.is_empty() {
        eprintln!("[freebuff-rs] 重连 {} 个已存节点", saved_links.len());
        let relay_c = relay.clone();
        tokio::spawn(async move {
            let text = saved_links.join("\n");
            let (imported, _, errors) = relay_c.import_many(&text).await;
            eprintln!("[freebuff-rs] 已重连 {}, 失败 {}", imported.len(), errors.len());
        });
    }
    let app = server::router(state).merge(admin::router(admin_state).await);
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await.unwrap();
    // 优雅退出 (Rust 惯用): SIGINT/SIGTERM → 停止收新请求 → 清理全部活跃 session → 退出
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(std::sync::Arc::clone(&pool)))
        .await
        .unwrap();
    eprintln!("[freebuff-rs] bye");
}

/// 等待 Ctrl-C / SIGTERM; 触发后 DELETE 上游全部活跃 session (不留挂尸会话等过期)
async fn shutdown_signal(pool: Arc<Pool>) {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut s) => { s.recv().await; }
            Err(_) => std::future::pending::<()>().await,
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    eprintln!("[freebuff-rs] shutting down, cleaning sessions…");
    let items = pool.all_sessions();
    let n = items.len();
    let base = upstream::base_for("");
    for (token, _model, inst) in items {
        let _ = upstream::delete_upstream_session(&base, &token, &inst).await;
    }
    eprintln!("[freebuff-rs] {n} session(s) cleaned");
}

async fn chat_cli(args: &[String]) {
    let model = args.get(2).cloned().unwrap_or_else(|| "deepseek/deepseek-v4-flash".into());
    let msg = args.get(3).cloned().unwrap_or_else(|| "say OK".into());
    let tokens = creds::load_tokens();
    if tokens.is_empty() {
        eprintln!("无 token, 先 login");
        std::process::exit(1);
    }
    let pool = Arc::new(Pool::new(&tokens.join(","), ""));
    let registry = models::current();
    let params = serde_json::json!({
        "model": model,
        "max_tokens": 1024,
        "messages": [{"role": "user", "content": msg}],
    });
    match gateway::execute_chat(&pool, &registry, &params, &model).await {
        Ok(exec) => {
            let (_, body) = exec.response.into_parts();
            let bytes = axum::body::to_bytes(body, 32 * 1024 * 1024).await.unwrap();
            let (content, _reasoning, _f, model_up, _usage) =
                protocol::aggregate_stream_text(&String::from_utf8_lossy(&bytes));
            println!("[{model_up}] {content}");
        }
        Err(resp) => {
            let (_, body) = resp.into_parts();
            let bytes = axum::body::to_bytes(body, 1024 * 1024).await.unwrap();
            eprintln!("chat failed: {}", String::from_utf8_lossy(&bytes));
            std::process::exit(1);
        }
    }
}
