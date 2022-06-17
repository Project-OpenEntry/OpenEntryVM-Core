use std::{collections::HashMap, sync::Arc};

use tokio::sync::{Mutex, OwnedMutexGuard};
use async_ffi::FfiFuture;

pub struct ExtensionData(Arc<Mutex<HashMap<u32, usize>>>);

impl ExtensionData {
    pub fn new() -> ExtensionData {
        ExtensionData(Arc::new(Mutex::new(HashMap::with_capacity(8))))
    }

    pub fn lock(&self) -> FfiFuture<OwnedMutexGuard<HashMap<u32, usize>>> {
        FfiFuture::new(self.0.clone().lock_owned())
    }
}