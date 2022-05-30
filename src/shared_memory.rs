use std::{sync::Arc, mem};

use crate::archive::Archive;

pub type SharedMemory = Arc<Memory>;

#[repr(transparent)]
pub struct Memory {
    memory: [u8]
}

impl Memory {
    pub fn from_archive(archive: &Archive) -> SharedMemory {
        unsafe { mem::transmute(Arc::<[u8]>::from(archive.code.clone())) }
    }

    pub fn ptr(&self) -> *const u8 {
        self.memory.as_ptr()
    }
}