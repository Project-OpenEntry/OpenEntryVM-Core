use crate::thread_counter::ShutdownType;

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum EventType {
    VMRun,
    VMEnd,
    VMShutdown(ShutdownType),
    Foreign {
        from: u32, 
        event: u32, 
        payload: EventArgs
    },
}


#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct EventArgs(usize);