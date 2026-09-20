//! 双桶并发信号量 — 逆向自桌面端 orchestrator 语义:
//! 上游免费层并发墙 = {槽:1, 并发:3}, 订阅层 = {槽:3, 并发:8}。
//! 网关全局级: 每请求 acquire 同时占 槽+并发 各一 permit, 实际并发上限 = min(槽, 并发)。
//! 首字节写出前 acquire; 超时 2s → 429 语义 (Busy), 不无限排队。RAII 归还。
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::time::{timeout, Duration};

pub const DEFAULT_FREE_SLOTS: usize = 1;
pub const DEFAULT_FREE_MULTI: usize = 3;
pub const DEFAULT_SUB_SLOTS: usize = 3;
pub const DEFAULT_SUB_MULTI: usize = 8;
/// acquire 超时: 超时视为并发繁忙 (429 语义, 对齐上游 waiting_room)
pub const ACQUIRE_TIMEOUT_MS: u64 = 2000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcquireError {
    /// 桶容量耗尽且超时未等到 → 429
    Busy,
}

/// 双桶信号量 (free / subscriber 互不干扰)
pub struct TieredSemaphore {
    free_slots: Arc<Semaphore>,
    free_multi: Arc<Semaphore>,
    sub_slots: Arc<Semaphore>,
    sub_multi: Arc<Semaphore>,
}

/// RAII 守卫: Drop 时归还两桶 permit
pub struct TierGuard {
    _slots: Option<tokio::sync::OwnedSemaphorePermit>,
    _multi: Option<tokio::sync::OwnedSemaphorePermit>,
}

impl TieredSemaphore {
    pub fn new(free_slots: usize, free_multi: usize, sub_slots: usize, sub_multi: usize) -> Self {
        Self {
            free_slots: Arc::new(Semaphore::new(free_slots.max(1))),
            free_multi: Arc::new(Semaphore::new(free_multi.max(1))),
            sub_slots: Arc::new(Semaphore::new(sub_slots.max(1))),
            sub_multi: Arc::new(Semaphore::new(sub_multi.max(1))),
        }
    }

    /// 默认容量 (免费 {1,3} / 订阅 {3,8})
    pub fn defaults() -> Self {
        Self::new(DEFAULT_FREE_SLOTS, DEFAULT_FREE_MULTI, DEFAULT_SUB_SLOTS, DEFAULT_SUB_MULTI)
    }

    /// 占用桶; 超时 2s 返回 Busy (调用方转 429)
    pub async fn acquire(&self, is_subscriber: bool) -> Result<TierGuard, AcquireError> {
        let (slots, multi) = if is_subscriber {
            (&self.sub_slots, &self.sub_multi)
        } else {
            (&self.free_slots, &self.free_multi)
        };
        timeout(Duration::from_millis(ACQUIRE_TIMEOUT_MS), async {
            let s = slots.clone().acquire_owned().await.map_err(|_| AcquireError::Busy)?;
            let m = multi.clone().acquire_owned().await.map_err(|_| AcquireError::Busy)?;
            Ok::<_, AcquireError>(TierGuard { _slots: Some(s), _multi: Some(m) })
        })
        .await
        .unwrap_or(Err(AcquireError::Busy))
    }
}
