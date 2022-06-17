#![feature(core_intrinsics, let_chains)]

use archive::Archive;
use runtime::Runtime;

mod extension_data;
mod virtual_thread;
mod thread_counter;
mod vm_intrinsics;
mod shared_memory;
mod block_info;
mod extensions;
mod vm_config;
mod register;
mod vm_value;
mod op_codes;
mod executor;
mod archive;
mod js_impl;
mod runtime;
mod string;
mod stack;
mod event;
mod utils;
mod tests;
mod ffi;

#[cfg(target_pointer_width = "32")]
compile_error!("This program is only for 64-bit or higher operating system.");

#[cfg(all(not(unix), not(windows)))]
compile_error!("Unsupported Operating System");

fn main() {
    if std::mem::size_of::<usize>() < 8 { panic!("This program is only for 64-bit or higher operating system.") }

    // Not implemented yet

    let archive = Archive::open("./Demo.entx");

    let runtime = Runtime::new(archive);

    runtime.run();
}