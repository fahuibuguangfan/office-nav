use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

/// 取消信号：可在线程/任务间共享的布尔标志
pub type CancelFlag = Arc<AtomicBool>;

/// 新建一个取消信号（初始为未取消）
pub fn new_cancel_flag() -> CancelFlag {
    Arc::new(AtomicBool::new(false))
}

/// 读取是否已被请求取消
pub fn is_cancelled(flag: &CancelFlag) -> bool {
    flag.load(Ordering::SeqCst)
}

/// 发布取消管理器：持有当前发布任务的取消信号
///
/// 单例（配合发布锁，同一时刻只有一个发布任务）。
/// 发布开始时 register 一个新信号，前端通过 `cancel_publish` 命令
/// 调用 `request_cancel` 把信号置为 true，发布流程在各步骤检测后中止。
/// 发布结束（成功/失败/取消）时 unregister，避免残留信号影响下次发布。
#[derive(Clone)]
pub struct PublishCancelManager {
    current: Arc<Mutex<Option<CancelFlag>>>,
}

impl PublishCancelManager {
    pub fn new() -> Self {
        Self {
            current: Arc::new(Mutex::new(None)),
        }
    }

    /// 注册当前发布任务的取消信号（发布开始时调用）
    pub fn register(&self, flag: CancelFlag) {
        *self.current.lock().unwrap() = Some(flag);
    }

    /// 注销当前发布任务的取消信号（发布结束时调用）
    pub fn unregister(&self) {
        *self.current.lock().unwrap() = None;
    }

    /// 请求取消当前发布任务，返回是否存在进行中的任务
    pub fn request_cancel(&self) -> bool {
        if let Some(ref flag) = *self.current.lock().unwrap() {
            flag.store(true, Ordering::SeqCst);
            true
        } else {
            false
        }
    }
}

impl Default for PublishCancelManager {
    fn default() -> Self {
        Self::new()
    }
}
