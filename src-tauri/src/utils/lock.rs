use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// 发布锁管理器：按工作目录加锁，防止同一项目同时发布到多个服务器
#[derive(Clone)]
pub struct PublishLockManager {
    locks: Arc<Mutex<HashMap<String, bool>>>,
}

impl PublishLockManager {
    pub fn new() -> Self {
        Self {
            locks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 尝试获取锁，返回是否成功
    pub fn try_lock(&self, work_dir: &str) -> bool {
        let mut locks = self.locks.lock().unwrap();
        if locks.get(work_dir).copied().unwrap_or(false) {
            // 已被锁定
            false
        } else {
            // 加锁
            locks.insert(work_dir.to_string(), true);
            true
        }
    }

    /// 释放锁
    pub fn unlock(&self, work_dir: &str) {
        let mut locks = self.locks.lock().unwrap();
        locks.remove(work_dir);
    }

    /// 检查是否已锁定
    pub fn is_locked(&self, work_dir: &str) -> bool {
        let locks = self.locks.lock().unwrap();
        locks.get(work_dir).copied().unwrap_or(false)
    }
}

impl Default for PublishLockManager {
    fn default() -> Self {
        Self::new()
    }
}
