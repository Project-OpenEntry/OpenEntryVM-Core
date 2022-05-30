use std::{env, sync::Arc, collections::{HashMap, hash_map::Values}};
use libloading::Library;

use crate::{virtual_thread::VThread, executor::executor::{Lock, ExecutorBehaviour}, runtime::Runtime, event::EventType};

pub struct Extensions(HashMap<u32, Arc<Extension>>);

impl Extensions {
    pub fn parse_from_env() -> Extensions {
        let iter = env::args().skip_while(|x| x == "--ext");
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
}

pub type ExtensionCall = fn(VThread, Lock, u32) -> ExecutorBehaviour;
pub type EventCall = fn(Arc<Runtime>, EventType);
pub struct Extension {
    _lib: Library, 
    env_fn: ExtensionCall, 
    envj_fn: ExtensionCall,
    event: EventCall,
}

impl Extension {
    pub fn new(path: String) -> Extension {
        unsafe {
            let lib = Library::new(path).unwrap();

            Extension {
                env_fn: *lib.get::<ExtensionCall>(b"vm_function_call").unwrap(),
                envj_fn: *lib.get::<ExtensionCall>(b"vm_interrupt").unwrap(),
                event: *lib.get::<EventCall>(b"vm_event_recv").unwrap(),
                _lib: lib,
            }
        }
    }

    pub fn function_call(&self, vthread: VThread, lock: Lock, id: u32) -> ExecutorBehaviour { (self.env_fn)(vthread, lock, id) }
    pub fn interrupt_call(&self, vthread: VThread, lock: Lock, id: u32) -> ExecutorBehaviour { (self.envj_fn)(vthread, lock, id) }
    pub fn dispatch_event(&self, runtime: Arc<Runtime>, event: EventType) {
        (self.event)(runtime, event)
    }
}