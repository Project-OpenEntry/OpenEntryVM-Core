use std::alloc::{self, Layout};

#[repr(transparent)]
pub struct Stack(usize);

impl Stack {
    pub fn new(stack_size: usize) -> Stack {
        unsafe {
            Stack(alloc::alloc(Layout::from_size_align_unchecked(stack_size, 8)) as usize)
        }
    }

    pub fn dispose(&self, stack_size: usize) {
        unsafe {
            alloc::dealloc(self.0 as *mut u8, Layout::from_size_align_unchecked(stack_size, 8));
        }
    }

    pub fn ptr(&self) -> u64 {
        self.0 as u64
    }
}