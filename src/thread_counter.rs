use std::{sync::{Arc, atomic::{AtomicU32, Ordering, AtomicU8}}, mem, pin::Pin};

use tokio::sync::{mpsc::UnboundedSender, Mutex};

use crate::{virtual_thread::{VirtualThread, VThread}, runtime::Runtime};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ShutdownType {
    None       = 0,
    Gracefully = 1,
    Restarting = 2,
    Error      = 3,
}

pub struct ThreadCounter {
    error_data: Arc<Mutex<Option<String>>>,
    ch: UnboundedSender<ShutdownType>,
    shutdown_type: AtomicU8,
    counter: AtomicU32,
}

impl ThreadCounter {
    pub fn new(tx: UnboundedSender<ShutdownType>) -> ThreadCounter {
        ThreadCounter {
            shutdown_type: AtomicU8::new(ShutdownType::None as u8),
            error_data: Arc::new(Mutex::new(None)),
            counter: AtomicU32::new(0),
            ch: tx,
        }
    }

    pub fn delete(&self, thread: Pin<Arc<VirtualThread>>) {
        let arc_thread = unsafe { Pin::into_inner_unchecked(thread) };

        if Arc::strong_count(&arc_thread) == 1 {
            if self.counter.fetch_sub(1, Ordering::SeqCst) == 1 {
                self.ch.send(unsafe { mem::transmute(self.shutdown_type.load(Ordering::SeqCst)) }).unwrap();
            }
        }
    }

    pub async fn create(&self, runtime: Arc<Runtime>, stack_size: usize, addr: u64) -> VThread {
        self.counter.fetch_add(1, Ordering::SeqCst);

        VirtualThread::new(runtime, stack_size, addr).await
    }

    pub fn set_shutdown_type(&self, code: ShutdownType) {
        self.shutdown_type.store(code as u8, Ordering::SeqCst);
    }

    pub async fn set_error_data(&self, data: String) {
        *self.error_data.lock().await = Some(data);
    }

    pub async fn get_error_data(&self) -> Option<String> {
        self.error_data.lock().await.clone()
    }

    pub async fn reset(&self) {
        *self.error_data.lock().await = None;
        self.set_shutdown_type(ShutdownType::None);
    }
}