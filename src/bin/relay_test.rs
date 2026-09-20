// 对齐复现: socket2+REUSEADDR 监听, 嵌套spawn(模拟axum handler里import), 12个监听
#[tokio::main]
async fn main() {
    use tokio::io::AsyncWriteExt;
    let mut ports = vec![];
    for i in 0..12u16 {
        let addr: std::net::SocketAddr = format!("127.0.0.1:{}", 19100 + i).parse().unwrap();
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        ports.push(19100 + i);
        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((s, _)) => {
                        eprintln!("[T{}] accepted", 19100 + i);
                        tokio::spawn(async move {
                            eprintln!("[T{}] sniff start", 19100 + i);
                            let mut buf = [0u8; 1];
                            use tokio::io::AsyncReadExt;
                            match s.peek(&mut buf).await {
                                Ok(n) => eprintln!("[T{}] peek {n} b0={:#x}", 19100 + i, buf[0]),
                                Err(e) => eprintln!("[T{}] peek err {e}", 19100 + i),
                            }
                        });
                    }
                    Err(e) => eprintln!("[T{}] accept err {e}", 19100 + i),
                }
            }
        });
    }
    // 模拟: 从另一个 spawn (axum handler 语境) 里连
    tokio::spawn(async move {
        let mut c = tokio::net::TcpStream::connect("127.0.0.1:19100").await.unwrap();
        c.write_all(b"GET / HTTP/1.1\r\n\r\n").await.unwrap();
    });
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    eprintln!("[T] done");
}
