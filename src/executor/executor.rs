use std::{future::Future, pin::Pin, sync::Arc, any::Any};

use fast_async_mutex::mutex::Mutex as SpinMutex;
use tokio::sync::Mutex;

use crate::{virtual_thread::VThread, archive::Archive, vm_config::ExecutorKind, thread_counter::ShutdownType};

use super::basic_executors::{AtomicExecutor, SysLockInstExecutor, SpinLockInstExecutor, SysLockBlockExecutor, SpinLockBlockExecutor};

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
pub type Executor = Arc<dyn ExecutorFunc + Send + Sync>;

pub trait ExecutorFunc {
    fn call(&self, thread: VThread) -> BoxFuture<'static, ()>;
}

impl<T, F> ExecutorFunc for T
where
    T: Fn(VThread) -> F,
    F: Future<Output = ()> + 'static + Send,
{
    fn call(&self, thread: VThread) -> BoxFuture<'static, ()> {
        Box::pin(self(thread))
    }
}

pub trait ExecutorExt {
    fn from_archive(archive: &Archive) -> Executor;
}

impl ExecutorExt for Executor {
    fn from_archive(archive: &Archive) -> Executor {
        match archive.conf.executor_kind {
            ExecutorKind::Atomic => Arc::new(AtomicExecutor::run),
            ExecutorKind::SysLockInst => Arc::new(SysLockInstExecutor::run),
            ExecutorKind::SpinLockInst => Arc::new(SpinLockInstExecutor::run),
            ExecutorKind::SysLockBlock => Arc::new(SysLockBlockExecutor::run),
            ExecutorKind::SpinLockBlock => Arc::new(SpinLockBlockExecutor::run),
        }
    }
}

pub enum ExecutorBehaviour {
    None,
    Shutdown(ShutdownType)
}

pub enum ExecutorLock {
    None,
    Sys(Arc<Mutex<()>>),
    Spin(Arc<SpinMutex<()>>),
}

impl ExecutorLock {
    pub fn from_archive(archive: &Arc<Archive>) -> ExecutorLock {
        match archive.conf.executor_kind {
            ExecutorKind::Atomic => ExecutorLock::None,
            ExecutorKind::SpinLockBlock | ExecutorKind::SpinLockInst => ExecutorLock::Spin(Arc::new(SpinMutex::new(()))),
            ExecutorKind::SysLockBlock | ExecutorKind::SysLockInst => ExecutorLock::Sys(Arc::new(Mutex::new(()))),
        }
    }

    pub fn sys(&self) -> &Arc<Mutex<()>> {
        match self {
            ExecutorLock::Sys(mutex) => mutex,
            _ => unreachable!()
        }
    }
    
    pub fn spin(&self) -> &Arc<SpinMutex<()>> {
        match self {
            ExecutorLock::Spin(mutex) => mutex,
            _ => unreachable!()
        }
    }
}

pub type Lock = Option<Box<dyn Any + Send + Sync>>;