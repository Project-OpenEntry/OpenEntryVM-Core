use crate::{virtual_thread::VThread, block_info::UnlockInfo};
use super::{instructions, executor::ExecutorBehaviour};

pub struct AtomicExecutor;
pub struct SysLockInstExecutor;
pub struct SpinLockInstExecutor;
pub struct SysLockBlockExecutor;
pub struct SpinLockBlockExecutor;

impl AtomicExecutor {
    pub async fn run(thread: VThread) {
        loop {
            let behaviour = instructions::run(thread.clone(), None).await;
            
            if let ExecutorBehaviour::Shutdown(shutdown_type) = behaviour {
                thread.shutdown(shutdown_type);

                break;
            } else if thread.should_stop() {
                thread.dispose();

                break;
            }
        }
    }
}

impl SysLockInstExecutor {
    pub async fn run(thread: VThread) {
        loop {
            let lock = Box::new(thread.lock.sys().clone().lock_owned().await);

            let behaviour = instructions::run(thread.clone(), Some(lock)).await;
            
            if let ExecutorBehaviour::Shutdown(shutdown_type) = behaviour {
                thread.shutdown(shutdown_type);

                break;
            } else if thread.should_stop() {
                thread.dispose();

                break;
            }
        }
    }
}

impl SpinLockInstExecutor {
    pub async fn run(thread: VThread) {
        loop {
            let lock = Box::new(thread.lock.spin().clone().lock_owned().await);

            let behaviour = instructions::run(thread.clone(), Some(lock)).await;
            
            if let ExecutorBehaviour::Shutdown(shutdown_type) = behaviour {
                thread.shutdown(shutdown_type);

                break;
            } else if thread.should_stop() {
                thread.dispose();

                break;
            }
        }
    }
}

impl SysLockBlockExecutor {
    pub async fn run(thread: VThread) {
        let block_info = thread.get_block_info();
        
        'executor: loop {
            let inst = thread.get_reg::<u64>(0);

            if let Some(info) = block_info.get(inst) {
                match info {
                    UnlockInfo::Current => {
                        let lock = Box::new(thread.lock.sys().clone().lock_owned().await);
                        let behaviour = instructions::run(thread.clone(), Some(lock)).await;
                
                        if let ExecutorBehaviour::Shutdown(shutdown_type) = behaviour {
                            thread.shutdown(shutdown_type);
            
                            break 'executor;
                        } else if thread.should_stop() {
                            thread.dispose();
            
                            break 'executor;
                        }
                    }
                    &UnlockInfo::Addr(end) => {
                        let lock = Box::new(thread.lock.sys().clone().lock_owned().await);

                        loop {
                            let have_to_unlock = thread.get_reg::<u64>(0) == end;
                            let behaviour = instructions::run(thread.clone(), None).await;
                    
                            if let ExecutorBehaviour::Shutdown(shutdown_type) = behaviour {
                                thread.shutdown(shutdown_type);
                
                                break 'executor;
                            } else if thread.should_stop() {
                                thread.dispose();
                
                                break 'executor;
                            }

                            if have_to_unlock {
                                drop(lock);

                                break;
                            }
                        }
                    }
                }
            } else {
                let behaviour = instructions::run(thread.clone(), None).await;
                
                if let ExecutorBehaviour::Shutdown(shutdown_type) = behaviour {
                    thread.shutdown(shutdown_type);
    
                    break 'executor;
                } else if thread.should_stop() {
                    thread.dispose();
    
                    break 'executor;
                }
            }
        }
    }
}

impl SpinLockBlockExecutor {
    pub async fn run(thread: VThread) {
        let block_info = thread.get_block_info();
        
        'executor: loop {
            let inst = thread.get_reg::<u64>(0);

            if let Some(info) = block_info.get(inst) {
                match info {
                    UnlockInfo::Current => {
                        let lock = Box::new(thread.lock.spin().clone().lock_owned().await);
                        let behaviour = instructions::run(thread.clone(), Some(lock)).await;
                
                        if let ExecutorBehaviour::Shutdown(shutdown_type) = behaviour {
                            thread.shutdown(shutdown_type);
            
                            break 'executor;
                        } else if thread.should_stop() {
                            thread.dispose();
            
                            break 'executor;
                        }
                    }
                    &UnlockInfo::Addr(end) => {
                        let lock = Box::new(thread.lock.spin().clone().lock_owned().await);

                        loop {
                            let have_to_unlock = thread.get_reg::<u64>(0) == end;
                            let behaviour = instructions::run(thread.clone(), None).await;
                    
                            if let ExecutorBehaviour::Shutdown(shutdown_type) = behaviour {
                                drop(lock);
    
                                thread.shutdown(shutdown_type);
                
                                break 'executor;
                            } else if thread.should_stop() {
                                drop(lock);
    
                                thread.dispose();
                
                                break 'executor;
                            }

                            if have_to_unlock {
                                drop(lock);

                                break;
                            }
                        }
                    }
                }
            } else {
                let behaviour = instructions::run(thread.clone(), None).await;
                
                if let ExecutorBehaviour::Shutdown(shutdown_type) = behaviour {
                    thread.shutdown(shutdown_type);
    
                    break 'executor;
                } else if thread.should_stop() {
                    thread.dispose();
    
                    break 'executor;
                }
            }
        }
    }
}