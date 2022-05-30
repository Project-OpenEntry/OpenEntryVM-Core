use std::ptr;

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

#[allow(dead_code)]
impl EventArgs {
    pub fn new<T: Sized>(args: T) -> EventArgs {
        EventArgs(Box::leak(Box::new(args)) as *mut T as usize)
    }

    pub unsafe fn unwrap<T: Sized>(&self) -> &'static mut T {
        &mut *(self.0 as *mut T)
    }

    pub unsafe fn dispose<T: Sized>(self) {
        ptr::drop_in_place(self.0 as *mut T);
    }
}