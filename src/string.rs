use std::{mem, ptr, alloc::{self, Layout}};

use crate::{vm_value::STR_SIGNATURE, virtual_thread::VThread};

pub struct VMStr(*const u64, VThread);

impl VMStr {
    pub(crate) fn deallocate(ptr: u64, len: usize) {
        unsafe {
            alloc::dealloc(ptr as _, Layout::from_size_align_unchecked(len, 1));
        }
    }

    pub async fn from_str(value: String, thread: VThread) -> VMStr {
        unsafe {
            let value = value.as_bytes();
            let ptr = alloc::alloc(Layout::from_size_align_unchecked(value.len() + 8, 1)) as u64;
            let len_buf = &mem::transmute::<u64, [u8; 8]>(value.len() as u64);
    
            ptr::copy_nonoverlapping(len_buf.as_ptr(), ptr as *mut u8, 8);
            ptr::copy_nonoverlapping(value.as_ptr(), (ptr as *mut u8).offset(8), value.len());
    
            thread.get_temp_vmstrs().await.insert((ptr, 8 + value.len()));

            VMStr(ptr as _, thread)
        }
    }

    pub fn from(value: u64, thread: VThread) -> VMStr {
        VMStr(value as *const u64, thread)
    }

    pub fn len(&self) -> u64 {
        unsafe { *self.0 }
    }

    pub fn as_str(&self) -> &str {
        unsafe { &*mem::transmute::<(*const u64, u64), *const str>((self.0.offset(1), self.len())) }
    }

    pub fn ptr(&self) -> *const u8 {
        self.0.cast::<u8>()
    }

    pub fn parse(&self) -> Option<f64> {
        self.as_str().parse::<f64>().ok()
    }

    pub async fn push(&mut self, other: &VMStr) {
        unsafe {
            let len = self.len();
            let additional = other.len();
            let new_len = len + additional;
            let len_buf = &mem::transmute::<u64, [u8; 8]>(new_len);

            let new_len = new_len + 8;

            let mut lock = self.1.get_temp_vmstrs().await;

            lock.remove(&(self.0 as u64, len as usize));

            self.0 = alloc::realloc(self.0 as _, Layout::from_size_align_unchecked(new_len as usize, 1), new_len as usize) as _;

            lock.insert((self.0 as u64, new_len as usize));

            drop(lock);

            let additional = additional as usize;

            ptr::copy_nonoverlapping(len_buf.as_ptr(), self.0.cast::<u8>() as _, 8);
            ptr::copy_nonoverlapping(other.ptr().offset(8), self.0.cast::<u8>().offset(len as isize + 8) as _, additional);
        }
    }

    pub async fn push_ffi(&mut self, other: VMStr) {
        unsafe {
            let len = self.len();
            let additional = other.len();
            let new_len = len + additional;
            let len_buf = &mem::transmute::<u64, [u8; 8]>(new_len);

            let new_len = new_len + 8;

            let mut lock = self.1.get_temp_vmstrs().await;

            lock.remove(&(self.0 as u64, len as usize));

            self.0 = alloc::realloc(self.0 as _, Layout::from_size_align_unchecked(new_len as usize, 1), new_len as usize) as _;

            lock.insert((self.0 as u64, new_len as usize));

            drop(lock);

            let additional = additional as usize;

            ptr::copy_nonoverlapping(len_buf.as_ptr(), self.0.cast::<u8>() as _, 8);
            ptr::copy_nonoverlapping(other.ptr().offset(8), self.0.cast::<u8>().offset(len as isize + 8) as _, additional);
        }
    }

    pub async fn cloned_push(&self, other: &VMStr) -> VMStr {
        unsafe {
            let len = self.len();
            let additional = other.len();
            let new_len = len + additional;
            let len_buf = &mem::transmute::<u64, [u8; 8]>(new_len);

            let new_len = new_len + 8;

            let ptr = alloc::alloc(Layout::from_size_align_unchecked(new_len as usize, 1)) as u64;

            let additional = additional as usize;

            ptr::copy_nonoverlapping(len_buf.as_ptr(), ptr as *mut u8, 8);
            ptr::copy_nonoverlapping(self.0.cast::<u8>().offset(8), (ptr as *mut u8).offset(8), len as usize);
            ptr::copy_nonoverlapping(other.ptr().offset(8), (ptr as *mut u8).offset(len as isize + 8), additional);

            self.1.get_temp_vmstrs().await.insert((ptr, new_len as usize));

            VMStr(ptr as _, self.1.clone())
        }
    }

    pub async fn cloned_push_ffi(&self, other: VMStr) -> VMStr {
        unsafe {
            let len = self.len();
            let additional = other.len();
            let new_len = len + additional;
            let len_buf = &mem::transmute::<u64, [u8; 8]>(new_len);

            let new_len = new_len + 8;

            let ptr = alloc::alloc(Layout::from_size_align_unchecked(new_len as usize, 1)) as u64;

            let additional = additional as usize;

            ptr::copy_nonoverlapping(len_buf.as_ptr(), ptr as *mut u8, 8);
            ptr::copy_nonoverlapping(self.0.cast::<u8>().offset(8), (ptr as *mut u8).offset(8), len as usize);
            ptr::copy_nonoverlapping(other.ptr().offset(8), (ptr as *mut u8).offset(len as isize + 8), additional);

            self.1.get_temp_vmstrs().await.insert((ptr, new_len as usize));

            VMStr(ptr as _, self.1.clone())
        }
    }

    pub async fn drop(&self) {
        unsafe {
            self.1.get_temp_vmstrs().await.remove(&(self.0 as u64, self.len() as usize + 8));
            alloc::dealloc(self.0 as _, Layout::from_size_align_unchecked(self.len() as usize + 8, 1));
        }
    }

    pub fn as_vm_value(&self) -> u64 {
        (self.0 as u64) | STR_SIGNATURE
    }

    pub fn str_eq(a: &VMStr, b: &VMStr) -> bool {
        a.as_str() == b.as_str()
    }
}

unsafe impl Send for VMStr {}
unsafe impl Sync for VMStr {}