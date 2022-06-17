use std::{sync::{Arc, atomic::{AtomicU64, Ordering}}, intrinsics, mem, pin::Pin, marker::PhantomPinned, collections::HashSet};

use tokio::sync::MutexGuard;

use crate::{register::Register, shared_memory::SharedMemory, runtime::Runtime, thread_counter::ShutdownType, stack::Stack, executor::executor::ExecutorLock, block_info::BlockInfo, extensions::Extension, extension_data::ExtensionData};

pub type VThread = Pin<Arc<VirtualThread>>;

pub struct VirtualThread {
    pub runtime: Arc<Runtime>,
    pub memory: SharedMemory,
    pub lock: ExecutorLock,
    pub stack: Stack,

    pub extension_data: ExtensionData,

    registers: [Register; 16],
    stack_size: usize,
    flags: AtomicU64,

    _phantom: PhantomPinned
}

impl VirtualThread {
    pub async fn new(runtime: Arc<Runtime>, stack_size: usize, addr: u64) -> VThread {
        let mut registers: [Register; 16] = Default::default();

        registers[0] = Register { r64: addr };
        registers[1] = Register { r64: runtime.base };

        let memory_lock = runtime.memory.read().await;
        let memory = memory_lock.clone();

        drop(memory_lock);

        let vthread = Arc::pin(VirtualThread {
            lock: ExecutorLock::from_archive(&runtime.archive),
            
            extension_data: ExtensionData::new(),
            stack: Stack::new(stack_size),
            flags: AtomicU64::new(0),
            registers: registers,
            runtime: runtime,
            stack_size,
            memory,

            _phantom: PhantomPinned
        });

        vthread.set_reg(4, vthread.stack.ptr() + stack_size as u64 - 8);

        vthread
    }

    pub fn get_extension(&self, id: u32) -> Arc<Extension> {
        self.runtime.extensions.get(id)
    }

    pub async fn get_temp_vmstrs(&self) -> MutexGuard<'_, HashSet<(u64, usize)>> {
        self.runtime.temp_vmstr.lock().await
    }

    pub async fn set_error_data(&self, data: impl Into<String>) {
        self.runtime.set_error_data(data.into()).await;
    }

    pub fn get_block_info(&self) -> Arc<BlockInfo> {
        self.runtime.archive.block_info.clone().unwrap()
    }

    pub fn should_stop(&self) -> bool {
        self.runtime.shutdown.load(Ordering::SeqCst)
    }

    pub async fn spawn(&self, addr: u64) {
        self.runtime.spawn(addr).await;
    }

    pub fn set_flag(&self, id: u64, value: bool) {
        self.flags.store((self.flags.load(Ordering::SeqCst) & !(1u64 << id)) | value as u64, Ordering::SeqCst);
    }

    pub fn get_flag(&self, id: u64) -> bool {
        unsafe { mem::transmute(((self.flags.load(Ordering::SeqCst) >> id) & 0b1) as u8) }
    }

    pub fn sub32(&self, reg: u8, amount: u32) {
        unsafe {
            let register = &self.registers[reg as usize].r64 as *const u64 as *mut u64;

            intrinsics::atomic_xsub(register, amount as u64);
        }
    }

    pub fn add32(&self, reg: u8, amount: u32) {
        unsafe {
            let register = &self.registers[reg as usize].r64 as *const u64 as *mut u64;

            intrinsics::atomic_xadd(register, amount as u64);
        }
    }

    pub fn inc_inst(&self, amount: u64) {
        unsafe {
            let register = &self.registers[0].r64 as *const u64 as *mut u64;

            intrinsics::atomic_xadd(register, amount);
        }
    }

    pub fn push(&self, data: u64) {
        unsafe {
            let register = &self.registers[4].r64 as *const u64 as *mut u64;

            self.set_mem_absolute(intrinsics::atomic_xsub(register, 8) as usize, data);
        }
    }

    pub fn pop(&self) -> u64 {
        unsafe {
            let register = &self.registers[4].r64 as *const u64 as *mut u64;

            self.get_mem_absolute(intrinsics::atomic_xadd(register, 8) as usize + 8)
        }
    }

    pub fn shutdown(self: VThread, shutdown_type: ShutdownType) {
        self.runtime.shutdown(shutdown_type);
        self.dispose();
    }

    pub fn dispose(self: VThread) {
        self.stack.dispose(self.stack_size);
        self.runtime.clone().dispose_thread(self);
    }

    pub fn get_mem<T: Copy>(&self, addr: usize) -> T {
        unsafe {
            intrinsics::atomic_load(self.memory.ptr().add(addr) as *const T)
        }
    }
    
    pub fn get_mem_absolute<T: Copy>(&self, addr: usize) -> T {
        unsafe {
            intrinsics::atomic_load(addr as *const T)
        }
    }

    pub fn set_mem_absolute<T: Copy>(&self, addr: usize, value: T) {
        unsafe {
            intrinsics::atomic_store(addr as *mut T, value);
        }
    }

    pub fn set_reg<T: Copy>(&self, idx: u8, value: T) {
        unsafe {
            let register = (&self.registers[idx as usize].r8 as *const u8 as *mut u8) as *mut T;

            intrinsics::atomic_store(register, value);
        }
    }

    pub fn get_reg<T: Copy>(&self, idx: u8) -> T {
        unsafe {
            let register = (&self.registers[idx as usize].r8 as *const u8) as *const T;

            intrinsics::atomic_load(register)
        }
    }
}

unsafe impl Send for VirtualThread {}
unsafe impl Sync for VirtualThread {}