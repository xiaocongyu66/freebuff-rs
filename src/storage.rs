//! 统一持久化路径: 首次运行初始化一个固定数据目录, 之后永远固定使用。
//! 目录不跟随二进制位置 —— 来回下载/替换二进制不影响已有数据。
//! 优先级: FREEBUFF_DATA_DIR 环境变量 > XDG 数据目录 (~/.local/share/freebuff-rs)。
use std::path::PathBuf;
use std::sync::OnceLock;

pub fn data_dir() -> PathBuf {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        // 1) 显式指定
        if let Ok(d) = std::env::var("FREEBUFF_DATA_DIR") {
            if !d.is_empty() {
                let p = PathBuf::from(d);
                let _ = std::fs::create_dir_all(&p);
                return p;
            }
        }
        // 2) 固定的 XDG 数据目录 (Linux 标准, 与二进制位置无关)
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        let p = PathBuf::from(home).join(".local/share/freebuff-rs");
        let _ = std::fs::create_dir_all(&p);
        p
    })
    .clone()
}

pub fn path(file: &str) -> PathBuf {
    data_dir().join(file)
}
