use std::{env, sync::Arc, collections::{HashMap, hash_map::{Values, Iter}}};
use libloading::Library;

use crate::{virtual_thread::VThread, executor::executor::{Lock, ExecutorBehaviour}, runtime::Runtime, event::EventType};

pub struct Extensions(HashMap<u32, Arc<Extension>>);

impl Extensions {
    pub fn parse_from_env() -> Extensions {
        let iter = env::args().skip_while(|x| x != "--ext").skip(1);
        let mut ext_paths = Vec::new();
        
        for x in iter {
            if let Some(i) = x.find('=') {
                let id = (&x[0..i]).parse::<u32>().unwrap();

                ext_paths.push((id, (&x[(i + 1)..]).to_owned()));
            } else {
                break;
            }
        }

        Extensions(ext_paths.into_iter().map(|(id, x)| (id, Arc::new(Extension::new(x)))).collect::<HashMap<_, _>>())
    }

    pub fn get(&self, id: u32) -> Arc<Extension> {
        self.0.get(&id).unwrap().clone()
    }

    pub fn iter<'a>(&self) -> Values<'_, u32, Arc<Extension>> {
        self.0.values()
    }

    pub fn all<'a>(&self) -> Iter<'_, u32, Arc<Extension>> {
        self.0.iter()
    }
}

pub type ExtensionCall = fn(VThread, Lock, u32, bool) -> (Lock, ExecutorBehaviour);
pub type EventCall = fn(Arc<Runtime>, EventType);
pub type InitCall = fn(Arc<Runtime>, u32);
pub struct Extension {
    _lib: Library, 
    env_fn: Option<ExtensionCall>, 
    envj_fn: Option<ExtensionCall>,
    init_fn: InitCall,
    event: EventCall,
}

impl Extension {
    pub fn new(path: String) -> Extension {
        unsafe {
            let lib = Library::new(path).unwrap();

            Extension {
                env_fn: lib.get::<ExtensionCall>(b"vm_function_call").ok().map(|x| *x),
                envj_fn: lib.get::<ExtensionCall>(b"vm_interrupt").ok().map(|x| *x),
                event: *lib.get::<EventCall>(b"vm_event_recv").unwrap(),
                init_fn: *lib.get::<InitCall>(b"vm_init").unwrap(),
                _lib: lib,
            }
        }
    }

    pub fn init(&self, runtime: Arc<Runtime>, id: u32) { (self.init_fn)(runtime, id) }

    pub fn function_call(&self, vthread: VThread, lock: Lock, id: u32, drop: bool) -> (Lock, ExecutorBehaviour) {
        (self.env_fn.expect("This Library does not support Function Call or does not contain"))(vthread, lock, id, drop)
    }

    pub fn interrupt_call(&self, vthread: VThread, lock: Lock, id: u32, drop: bool) -> (Lock, ExecutorBehaviour) {
        (self.envj_fn.expect("This Library does not support Interrupt Call or does not contain"))(vthread, lock, id, drop)
    }

    pub fn dispatch_event(&self, runtime: Arc<Runtime>, event: EventType) {
        (self.event)(runtime, event)
    }
}