//! 进程内直连: 网关经隧道出站直接拨号, 并在隧道流上直接协商 TLS。
//! 相比"本机回环代理跳"少一次 localhost 连接与两轮双向拷贝。
use crate::tunnel::outbound::Outbound;
use crate::tunnel::stream::BoxStream;
use hyper::Uri;
use hyper_util::client::legacy::connect;
use hyper_util::client::legacy::Client;
use http_body_util::Full;
use hyper_util::rt::{TokioExecutor, TokioIo, TokioTimer};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

/// connector: 所有请求都拨到同一个隧道目标 (如 www.codebuff.com:443)。
#[derive(Clone)]
pub struct TunnelConnector {
    ob: Arc<Outbound>,
    insecure: bool,
    target: String,
}

pub struct TunnelIo {
    inner: TokioIo<tokio_rustls::client::TlsStream<BoxStream>>,
}

impl hyper::rt::Read for TunnelIo {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: hyper::rt::ReadBufCursor<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}

impl hyper::rt::Write for TunnelIo {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        Pin::new(&mut self.inner).poll_write(cx, buf)
    }
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }
    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

impl connect::Connection for TunnelIo {
    fn connected(&self) -> connect::Connected {
        connect::Connected::new()
    }
}

pub fn tunneled_client(
    ob: Arc<Outbound>,
    target: &str,
    insecure: bool,
) -> Client<TunnelConnector, Full<hyper::body::Bytes>> {
    Client::builder(TokioExecutor::new())
        .timer(TokioTimer::new())
        .build(TunnelConnector {
            ob,
            insecure,
            target: target.to_string(),
        })
}

impl tower::Service<Uri> for TunnelConnector {
    type Response = TunnelIo;
    type Error = String;
    type Future = Pin<Box<dyn Future<Output = Result<TunnelIo, String>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: Uri) -> Self::Future {
        let ob = self.ob.clone();
        let insecure = self.insecure;
        let target = self.target.clone();
        Box::pin(async move {
            let (h, p) = split_host_port(&target);
            let t0 = std::time::Instant::now();
            eprintln!("[tunnel-client] dial {h}:{p}");
            let io: BoxStream = ob.connect(&h, p).await?;
            eprintln!("[tunnel-client] dialed in {:?}", t0.elapsed());
            let r = crate::tunnel::tls_util::tls_wrap(io, &h, None, insecure).await;
            eprintln!("[tunnel-client] tls {:?} -> {}", t0.elapsed(), if r.is_ok() { "ok" } else { "err" });
            Ok(TunnelIo { inner: TokioIo::new(r?) })
        })
    }
}

/// 通用请求: 返回 (status, body)。经隧道拨号, 隧道流上 TLS。
pub async fn request(
    ob: Arc<Outbound>,
    target_host: &str,
    method: &str,
    path: &str,
    token: &str,
    headers: &[(&str, String)],
    body_json: Option<&serde_json::Value>,
    timeout_secs: u64,
) -> Result<(u16, hyper::body::Incoming), String> {
    let client: Client<TunnelConnector, Full<hyper::body::Bytes>> = Client::builder(TokioExecutor::new())
        .timer(TokioTimer::new())
        .build(TunnelConnector {
            ob,
            insecure: false,
            target: format!("{target_host}:443"),
        });
    let uri: Uri = format!("https://{target_host}{path}")
        .parse()
        .map_err(|e| format!("uri: {e}"))?;
    let m = http::Method::from_bytes(method.as_bytes()).map_err(|e| format!("method: {e}"))?;
    let mut req = http::Request::builder().method(m).uri(uri);
    if !token.is_empty() {
        req = req.header("authorization", format!("Bearer {token}"));
    }
    for (k, v) in headers {
        req = req.header(*k, v);
    }
    let req = if let Some(b) = body_json {
        req.header("content-type", "application/json")
            .body(Full::from(b.to_string()))
    } else {
        req.body(Full::from(""))
    }
    .map_err(|e| e.to_string())?;
    let resp = tokio::time::timeout(
        std::time::Duration::from_secs(timeout_secs),
        tower::ServiceExt::oneshot(client, req),
    )
    .await
    .map_err(|_| "upstream timeout".to_string())?
    .map_err(|e| format!("tunnel http: {e}"))?;
    let st = resp.status().as_u16();
    eprintln!("[tunnel-client] resp {st} for {path}");
    Ok((st, resp.into_body()))
}

/// 全局隧道注册 (启动时 set)
pub fn set_relay(relay: Arc<crate::relay::Relay>) {
    let _ = TUNNEL_RELAY.set(relay);
}

fn tunnel_relay() -> Option<&'static Arc<crate::relay::Relay>> {
    TUNNEL_RELAY.get()
}

static TUNNEL_RELAY: std::sync::OnceLock<Arc<crate::relay::Relay>> = std::sync::OnceLock::new();

/// 节点连接失败降权 (网关调用)
pub async fn mark_node_fail(port: u16) {
    if let Some(relay) = tunnel_relay() {
        relay.mark_fail(port).await;
    }
}

/// 网关出站选择: FREEBUFF_PROXY 显式指定则用之; 否则自动挑未受限出口
pub async fn tunnel_outbound() -> Option<(Arc<Outbound>, u16)> {
    let relay = tunnel_relay()?;
    if let Ok(p) = std::env::var("FREEBUFF_PROXY") {
        if let Some(port) = p.rsplit(':').next().and_then(|x| x.parse::<u16>().ok()) {
            return relay.outbound_for_port(port).await.map(|ob| (ob, port));
        }
    }
    // 自动调度: 未受限出口优先 (受限记忆来自节点探测)
    relay.best_outbound().await
}

pub fn split_host_port(hostport: &str) -> (String, u16) {
    if let Some(rest) = hostport.strip_prefix('[') {
        if let Some(idx) = rest.find(']') {
            let host = rest[..idx].to_string();
            let port = rest[idx + 2..].parse().unwrap_or(443);
            return (host, port);
        }
    }
    match hostport.rsplit_once(':') {
        Some((h, p)) => (h.to_string(), p.parse().unwrap_or(443)),
        None => (hostport.to_string(), 443),
    }
}

