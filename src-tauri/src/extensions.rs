use std::sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

pub trait IgnoreLockPoison<T> {
    fn lock_or_panic(&self) -> MutexGuard<T>;
}
impl<T> IgnoreLockPoison<T> for Mutex<T> {
    /// 如果发生了lock poison，则直接panic
    #[allow(clippy::unwrap_used)]
    fn lock_or_panic(&self) -> MutexGuard<T> {
        self.lock().unwrap()
    }
}

pub trait IgnoreRwLockPoison<T> {
    fn read_or_panic(&self) -> RwLockReadGuard<T>;
    fn write_or_panic(&self) -> RwLockWriteGuard<T>;
}

impl<T> IgnoreRwLockPoison<T> for RwLock<T> {
    /// 如果发生了lock poison，则直接panic
    #[allow(clippy::unwrap_used)]
    fn read_or_panic(&self) -> RwLockReadGuard<T> {
        self.read().unwrap()
    }

    /// 如果发生了lock poison，则直接panic
    #[allow(clippy::unwrap_used)]
    fn write_or_panic(&self) -> RwLockWriteGuard<T> {
        self.write().unwrap()
    }
}

pub trait AnyhowErrorToStringChain {
    /// 将 `anyhow::Error` 转换为chain格式
    /// # Example
    /// 0: error message
    /// 1: error message
    /// 2: error message
    fn to_string_chain(&self) -> String;
}

impl AnyhowErrorToStringChain for anyhow::Error {
    fn to_string_chain(&self) -> String {
        use std::fmt::Write;
        self.chain()
            .enumerate()
            .fold(String::new(), |mut output, (i, e)| {
                let _ = writeln!(output, "{i}: {e}");
                output
            })
    }
}
