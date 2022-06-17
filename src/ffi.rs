#![allow(non_upper_case_globals, dead_code, non_snake_case)]

macro_rules! async_binding {
    ($functions: ident, &Arc, $root: ident::$func: ident, $ret: ty, [$($id: ident: $ty: ty),*]) => {
        {
            paste! {
                pub extern "Rust" fn [<$root __ $func>]<'a>(this: &'a ::std::sync::Arc<$root>$(, $id: $ty)*) -> BorrowingFfiFuture<'a, $ret> {
                    BorrowingFfiFuture::new($root::$func(this$(, $id)*))
                }

                $functions.push([<$root __ $func>] as usize);
            }
        }
    };
    ($functions: ident, $root: ident::$func: ident, $ret: ty, [$($id: ident: $ty: ty),*]) => {
        {
            paste! {
                pub extern "Rust" fn [<$root __ $func>]<'a>(this: &'a $root$(, $id: $ty)*) -> BorrowingFfiFuture<'a, $ret> {
                    BorrowingFfiFuture::new($root::$func(this$(, $id)*))
                }

                $functions.push([<$root __ $func>] as usize);
            }
        }
    };
    ($functions: ident, noself, $root: ident::$func: ident, $ret: ty, [$($id: ident: $ty: ty),*]) => {
        {
            paste! {
                pub extern "Rust" fn [<$root __ $func>]<'a>($($id: $ty),*) -> FfiFuture<$ret> {
                    BorrowingFfiFuture::new($root::$func($($id),*))
                }

                $functions.push([<$root __ $func>] as usize);
            }
        }
    };
}

macro_rules! fn_bindings {
    ($(fn_bind![$ty: ty, $root: ident::$func: ident];)+) => {
        vec![$($root::$func as usize),+]
    };
}

use std::{sync::Arc, collections::{HashSet, HashMap}};

use tokio::sync::{MutexGuard, OwnedMutexGuard};
use paste::paste;

use async_ffi::{BorrowingFfiFuture, FfiFuture};

use crate::{
    virtual_thread::{VThread, VirtualThread}, 
    extensions::{Extensions, Extension},
    executor::executor::ExecutorLock, 
    block_info::BlockInfo, 
    shared_memory::Memory, 
    vm_value::VMValue, 
    runtime::Runtime, 
    string::VMStr,
    stack::Stack, extension_data::ExtensionData, 
};

pub struct FfiBindings(pub Arc<[usize]>);

impl FfiBindings {
    pub fn new() -> FfiBindings {
        let mut functions = fn_bindings![
            fn_bind![fn(&Arc<Runtime>, EventType), Runtime::dispatch_extension_event];
            fn_bind![fn(&Arc<Runtime>, u32, EventType), Runtime::send_extension_event];
            fn_bind![fn(&Runtime, VThread), Runtime::dispose_thread];
            fn_bind![fn(&Runtime, ShutdownType), Runtime::shutdown];

            fn_bind![fn(&Memory) -> *const u8, Memory::ptr];

            fn_bind![fn(&Extensions) -> Values<'_, u32, Arc<Extension>>, Extensions::iter];
            fn_bind![fn(&Extensions) -> Iter<'_, u32, Arc<Extension>>, Extensions::all];
            fn_bind![fn(&Extensions, u32) -> Arc<Extension>, Extensions::get];

            fn_bind![fn(&Extension, VThread, Lock, u32, bool) -> (Lock, ExecutorBehaviour), Extension::function_call];
            fn_bind![fn(&Extension, VThread, Lock, u32, bool) -> (Lock, ExecutorBehaviour), Extension::interrupt_call];
            fn_bind![fn(&Extension, Arc<Runtime>, EventType), Extension::dispatch_event];
            fn_bind![fn(&Extension, Arc<Runtime>, u32), Extension::init];

            fn_bind![fn(&BlockInfo, u64) -> Option<&UnlockInfo>, BlockInfo::get];
            fn_bind![fn(&Stack, usize), Stack::dispose];
            fn_bind![fn(&Stack) -> u64, Stack::ptr];

            fn_bind![fn(&ExecutorLock) -> &Arc<Mutex<()>>, ExecutorLock::sys];
            fn_bind![fn(&ExecutorLock) -> &Arc<SpinMutex<()>>, ExecutorLock::spin];

            fn_bind![fn(&VirtualThread, u32) -> Arc<Extension>, VirtualThread::get_extension];
            fn_bind![fn(&VirtualThread) -> Arc<BlockInfo>, VirtualThread::get_block_info];
            fn_bind![fn(&VirtualThread) -> bool, VirtualThread::should_stop];
            fn_bind![fn(&VirtualThread, u64, bool), VirtualThread::set_flag];
            fn_bind![fn(&VirtualThread, u64) -> bool, VirtualThread::get_flag];
            fn_bind![fn(&VirtualThread, u8, u32), VirtualThread::sub32];
            fn_bind![fn(&VirtualThread, u8, u32), VirtualThread::add32];
            fn_bind![fn(&VirtualThread, u64), VirtualThread::inc_inst];
            fn_bind![fn(&VirtualThread, u64), VirtualThread::push];
            fn_bind![fn(&VirtualThread) -> u64, VirtualThread::pop];
            fn_bind![fn(VThread, ShutdownType), VirtualThread::shutdown];
            fn_bind![fn(VThread), VirtualThread::dispose];
            
            fn_bind![fn(u64, VThread) -> VMValue, VMValue::from];
            fn_bind![fn(&mut VMValue) -> Option<(&mut VMStr, bool)>, VMValue::as_str];
            fn_bind![fn(&VMValue) -> Option<f64>, VMValue::as_f64];

            fn_bind![fn(&VMStr, &VMStr) -> bool, VMStr::str_eq];
            fn_bind![fn(&VMStr) -> Option<f64>, VMStr::parse];
            fn_bind![fn(u64, VThread) -> VMStr, VMStr::from];
            fn_bind![fn(&VMStr) -> u64, VMStr::as_vm_value];
            fn_bind![fn(&VMStr) -> *const u8, VMStr::ptr];
            fn_bind![fn(&VMStr) -> &str, VMStr::as_str];
            fn_bind![fn(&VMStr) -> u64, VMStr::len];
        ];

        async_binding!(functions, Runtime::set_error_data, (), [data: String]);
        async_binding!(functions, &Arc, Runtime::spawn, (), [addr: u64]);

        async_binding![functions, VirtualThread::get_temp_vmstrs, MutexGuard<'_, HashSet<(u64, usize)>>, []];
        async_binding![functions, VirtualThread::set_error_data, (), [data: String]];

        async_binding![functions, VirtualThread::spawn, (), [addr: u64]];

        async_binding![functions, noself, VMStr::from_str, VMStr, [value: String, thread: VThread]];

        async_binding![functions, VMStr::drop, (), []];

        functions.extend_from_slice(&[
            VirtualThread::get_mem::<u64> as usize,
            VirtualThread::get_mem::<u32> as usize,
            VirtualThread::get_mem::<u16> as usize,
            VirtualThread::get_mem::<u8> as usize,
            VirtualThread::get_mem_absolute::<u64> as usize,
            VirtualThread::get_mem_absolute::<u32> as usize,
            VirtualThread::get_mem_absolute::<u16> as usize,
            VirtualThread::get_mem_absolute::<u8> as usize,
            VirtualThread::set_mem_absolute::<u64> as usize,
            VirtualThread::set_mem_absolute::<u32> as usize,
            VirtualThread::set_mem_absolute::<u16> as usize,
            VirtualThread::set_mem_absolute::<u8> as usize,
            VirtualThread::set_reg::<u64> as usize,
            VirtualThread::set_reg::<u32> as usize,
            VirtualThread::set_reg::<u16> as usize,
            VirtualThread::set_reg::<u8> as usize,
            VirtualThread::get_reg::<u64> as usize,
            VirtualThread::get_reg::<u32> as usize,
            VirtualThread::get_reg::<u16> as usize,
            VirtualThread::get_reg::<u8> as usize,
        ]);

        {
            pub extern "Rust" fn VMStr__push<'a>(this: &'a mut VMStr, other: VMStr) -> BorrowingFfiFuture<'a, ()> {
                BorrowingFfiFuture::new(VMStr::push_ffi(this, other))
            }
            
            pub extern "Rust" fn VMStr__cloned_push<'a>(this: &'a VMStr, other: VMStr) -> BorrowingFfiFuture<'a, VMStr> {
                BorrowingFfiFuture::new(VMStr::cloned_push_ffi(this, other))
            }
            
            pub extern "Rust" fn ExtensionData__lock(this: &ExtensionData) -> FfiFuture<OwnedMutexGuard<HashMap<u32, usize>>> {
                this.lock()
            }

            functions.extend(&[
                VMStr__push as usize,
                VMStr__cloned_push as usize,
                ExtensionData__lock as usize,
            ]);
        }

        FfiBindings(functions.into())
    }
}