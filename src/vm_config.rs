use crate::utils::ReadBuffer;

#[allow(dead_code)] // Constructed with ptr::read::<ExecutorKind>()
#[derive(Clone, Copy)]
pub enum ExecutorKind {
    Atomic = 0,
    SysLockInst = 1,
    SpinLockInst = 2,
    SysLockBlock = 3,
    SpinLockBlock = 4,
}

#[allow(dead_code)] // Constructed with ptr::read::<ThreadingKind>()
#[derive(Clone, Copy)]
pub enum ThreadingKind {
    Single = 0,
    Managed = 1,
    Unmanaged = 2,
}

pub struct VMConfig {
    pub executor_kind: ExecutorKind,
    pub threading_kind: ThreadingKind,
    pub max_threads: u16,
    pub stack_size: u64,
}

impl VMConfig {
    pub fn read(buffer: Box<[u8]>) -> VMConfig {
        VMConfig {
            executor_kind: buffer.const_read::<0, ExecutorKind>(),
            threading_kind: buffer.const_read::<1, ThreadingKind>(),
            max_threads: match buffer.const_read::<2, u16>() {
                0 => num_cpus::get() as u16,
                x => x
            },
            stack_size: buffer.const_read::<8, u64>(),
        }
    }
}