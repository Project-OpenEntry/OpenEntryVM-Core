use tokio::time::Duration;

use crate::{virtual_thread::VThread, vm_value::VMValue, thread_counter::ShutdownType, executor::executor::{ExecutorBehaviour, Lock}};

use self::intrinsics::Intrinsic;

mod intrinsics;

pub async fn call(thread: VThread, id: u8, lock: Lock) -> ExecutorBehaviour {
    match id {
        Intrinsic::Debug => {
            eprintln!("----- Register Dump -----");
            eprintln!("INST: {}", VMValue::from(thread.get_reg::<u64>(0), thread.clone()));
            eprintln!("BASE: {}", VMValue::from(thread.get_reg::<u64>(1), thread.clone()));
            eprintln!("FUNC: {}", VMValue::from(thread.get_reg::<u64>(2), thread.clone()));
            eprintln!("OBJ:  {}", VMValue::from(thread.get_reg::<u64>(3), thread.clone()));
            eprintln!("TOP:  {}\n", VMValue::from(thread.get_reg::<u64>(4), thread.clone()));
            eprintln!("RET0: {}\n", VMValue::from(thread.get_reg::<u64>(5), thread.clone()));
            eprintln!("D0:   {}", VMValue::from(thread.get_reg::<u64>(6), thread.clone()));
            eprintln!("D1:   {}\n", VMValue::from(thread.get_reg::<u64>(7), thread.clone()));
            eprintln!("R0:   {}", VMValue::from(thread.get_reg::<u64>(8), thread.clone()));
            eprintln!("R1:   {}", VMValue::from(thread.get_reg::<u64>(9), thread.clone()));
            eprintln!("R2:   {}", VMValue::from(thread.get_reg::<u64>(10), thread.clone()));
            eprintln!("R3:   {}", VMValue::from(thread.get_reg::<u64>(11), thread.clone()));
            eprintln!("R4:   {}", VMValue::from(thread.get_reg::<u64>(12), thread.clone()));
            eprintln!("R5:   {}", VMValue::from(thread.get_reg::<u64>(13), thread.clone()));
            eprintln!("R6:   {}", VMValue::from(thread.get_reg::<u64>(14), thread.clone()));
            eprintln!("R7:   {}", VMValue::from(thread.get_reg::<u64>(15), thread.clone()));
        }
        Intrinsic::Sleep => {
            if let VMValue::Float(value) = VMValue::from(thread.get_reg::<u64>(8), thread.clone()) {
                // Only locking mutex for fetch/writing data with VThread.
                // Locking mutex while waiting can make other threads can't do anything while waiting.
                // So, We MUST drop after we passed critical section.
                drop(lock);

                tokio::time::sleep(Duration::from_secs_f64(value)).await;
            }
        }
        Intrinsic::Restart => {
            return ExecutorBehaviour::Shutdown(ShutdownType::Restarting);
        }
        Intrinsic::Throw => {
            thread.set_error_data("Called Intrinsic::Throw").await;
            
            return ExecutorBehaviour::Shutdown(ShutdownType::Error);
        }
        _ => panic!("Unsupported VM Intrinsic Call")
    }

    return ExecutorBehaviour::None;
}