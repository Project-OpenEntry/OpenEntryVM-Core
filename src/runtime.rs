use tokio::{runtime::{Runtime as TokioRuntime, Builder as TokioBuilder}, sync::{mpsc::{self, UnboundedReceiver}, Mutex, RwLock}};

use crate::{
    virtual_thread::VThread, executor::executor::{Executor, ExecutorExt}, 
    thread_counter::{ThreadCounter, ShutdownType}, 
    shared_memory::{SharedMemory, Memory}, 
    vm_config::ThreadingKind,
    archive::Archive, string::VMStr, extensions::Extensions, event::EventType, extension_data::ExtensionData, ffi::FfiBindings,
};
use std::{sync::{Arc, atomic::{AtomicBool, Ordering}}, process, collections::HashSet};

pub struct Runtime {
    pub temp_vmstr: Arc<Mutex<HashSet<(u64, usize)>>>,
    pub memory: RwLock<SharedMemory>,
    pub tokio_rt: Arc<TokioRuntime>,
    pub extensions: Extensions,
    pub archive: Arc<Archive>,
    pub shutdown: AtomicBool,
    pub initial_inst: u64,
    pub base: u64,

    pub ffi: FfiBindings,
    pub extension_data: ExtensionData,

    shutdown_rx: Mutex<UnboundedReceiver<ShutdownType>>,
    threads: ThreadCounter,
    stack_size: usize,

    executor: Executor
}

impl Runtime {
    fn tokio_rt(archive: &Archive) -> TokioRuntime {
        match archive.conf.threading_kind {
            ThreadingKind::Single => TokioBuilder::new_current_thread().enable_all().worker_threads(1).build().unwrap(),
            _ => TokioBuilder::new_multi_thread().enable_all().worker_threads(archive.conf.max_threads as usize).build().unwrap()
        }
    }

    pub fn dispatch_extension_event(self: &Arc<Self>, event: EventType) {
        for ext in self.extensions.iter() {
            ext.dispatch_event(self.clone(), event);
        }
    }

    #[allow(dead_code)]
    pub fn send_extension_event(self: &Arc<Self>, target: u32, event: EventType) {
        self.extensions.get(target).dispatch_event(self.clone(), event);
    }

    pub fn new(archive: Archive) -> Arc<Runtime> {
        let channel = mpsc::unbounded_channel::<ShutdownType>();
        let executor = Executor::from_archive(&archive);
        let memory = Memory::from_archive(&archive);

        let runtime = Arc::new(Runtime {
            temp_vmstr: Arc::new(Mutex::new(HashSet::with_capacity(128))),
            tokio_rt: Arc::new(Runtime::tokio_rt(&archive)),
            stack_size: archive.conf.stack_size as usize,
            threads: ThreadCounter::new(channel.0),
            extensions: Extensions::parse_from_env(),
            memory: RwLock::new(memory.clone()),
            shutdown_rx: Mutex::new(channel.1),
            shutdown: AtomicBool::new(false),
            archive: Arc::new(archive),

            ffi: FfiBindings::new(),
            extension_data: ExtensionData::new(),
            
            initial_inst: unsafe { *memory.ptr().cast::<u64>() } * 8 + 0x8,
            base: memory.ptr() as u64,
            
            executor,
        });
        
        for (&id, ext) in runtime.extensions.all() {
            ext.init(runtime.clone(), id);
        }

        runtime
    }

    pub fn run(self: Arc<Self>) {
        let tokio_rt = self.tokio_rt.clone();

        tokio_rt.block_on(async move {
            let runtime = self;

            runtime.dispatch_extension_event(EventType::VMRun);

            loop {
                let shutdown_type = runtime.run_once().await;

                // Preventing Memory Leaks
                for (ptr, len) in runtime.temp_vmstr.lock().await.drain() {
                    VMStr::deallocate(ptr, len);
                }

                if shutdown_type == ShutdownType::Restarting {
                    runtime.threads.reset().await;

                    *runtime.memory.write().await = Memory::from_archive(&runtime.archive);

                    runtime.shutdown.store(false, Ordering::SeqCst);

                    runtime.dispatch_extension_event(EventType::VMShutdown(shutdown_type));
                } else {
                    runtime.dispatch_extension_event(EventType::VMShutdown(shutdown_type));

                    break;
                }
            };

            runtime.dispatch_extension_event(EventType::VMEnd);
        });
    }

    async fn run_once(self: &Arc<Self>) -> ShutdownType {
        self.shutdown.store(false, Ordering::SeqCst);

        self.spawn(self.initial_inst).await;

        match self.shutdown_rx.lock().await.recv().await.unwrap() {
            code => {
                match code {
                    ShutdownType::None => {
                        println!("OpenEntry VM Stopped Running with Unknown Reason.");

                        process::exit(1);
                    }
                    ShutdownType::Error => {
                        if let Some(data) = self.threads.get_error_data().await {
                            println!("OpenEntry VM Stopped Running with Error: \n{}", data);
                        } else {
                            println!("OpenEntry VM Stopped Running with Error.");
                            println!("Extra Error Details weren't provided.");
                        }

                        process::exit(1);
                    }
                    x => x
                }
            }
        }
    }

    pub async fn spawn(self: &Arc<Self>, addr: u64) {
        let thread = self.threads.create(self.clone(), self.stack_size, addr).await;
        let executor = self.executor.clone();

        if self.archive.conf.threading_kind == ThreadingKind::Managed {
            tokio::task::spawn(async move {
                executor.call(thread).await;
            });
        } else {
            tokio::spawn(async move {
                executor.call(thread).await;
            });
        }
    }

    pub fn shutdown(&self, shutdown_type: ShutdownType) {
        self.threads.set_shutdown_type(shutdown_type);
        self.shutdown.store(true, Ordering::SeqCst);
    }

    pub fn dispose_thread(&self, thread: VThread) {
        self.threads.delete(thread);
    }

    pub async fn set_error_data(&self, data: String) {
        self.threads.set_error_data(data).await;
    }
}