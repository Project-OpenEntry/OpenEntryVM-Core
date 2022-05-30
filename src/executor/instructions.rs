use crate::{op_codes::{OpCodes, OpLayout}, vm_value::{VMValue, STR_SIGNATURE}, virtual_thread::VThread, thread_counter::ShutdownType, js_impl, vm_intrinsics, utils::handle_lock};

use super::executor::{ExecutorBehaviour, Lock};

pub async fn run<const DROP: bool>(thread: VThread, lock: Lock) -> (Lock, ExecutorBehaviour) {
    let ip = thread.get_reg::<u64>(0) as usize;
    let op = thread.get_mem::<u8>(ip);

    match op {
        x if x == OpCodes::MOV | OpLayout::R_R => {
            let dest = thread.get_mem::<u8>(ip + 1);
            let src = thread.get_mem::<u8>(ip + 2);
            
            thread.set_reg(dest, thread.get_reg::<u64>(src));
        }
        x if x == OpCodes::MOV | OpLayout::R_RO => {
            let dest = thread.get_mem::<u8>(ip + 1);
            let base = thread.get_reg::<u64>(thread.get_mem::<u8>(ip + 2)) as usize;
            let offset = thread.get_mem::<u32>(ip + 4) as usize;

            thread.set_reg(dest, thread.get_mem_absolute::<u64>(base + offset * 8));
        }
        x if x == OpCodes::MOV | OpLayout::RO_R => {
            let base = thread.get_reg::<u64>(thread.get_mem::<u8>(ip + 1)) as usize;
            let dest = thread.get_mem::<u8>(ip + 2);
            let offset = thread.get_mem::<u32>(ip + 4) as usize;

            thread.set_mem_absolute(base + offset * 8, thread.get_reg::<u64>(dest));
        }
        x if x == OpCodes::MOV | OpLayout::R_I => {
            let dest = thread.get_mem::<u8>(ip + 1);
            let immediate = thread.get_mem::<u64>(ip + 8);

            thread.set_reg(dest, immediate);

            thread.inc_inst(8); // Fat instruction
        }
        x if x == OpCodes::MOV | OpLayout::RO_I => {
            let base = thread.get_reg::<u64>(thread.get_mem::<u8>(ip + 1)) as usize;
            let offset = thread.get_mem::<u32>(ip + 4) as usize;
            let immediate = thread.get_mem::<u64>(ip + 8);

            thread.set_mem_absolute(base + offset * 8, immediate);

            thread.inc_inst(8); // Fat instruction
        }
        OpCodes::LSTR => {
            let reg = thread.get_mem::<u8>(ip + 1);
            let base = thread.get_reg::<u64>(thread.get_mem::<u8>(ip + 2));
            let offset = thread.get_mem::<u32>(ip + 4) as u64;

            thread.set_reg(reg, (base + offset * 8) | STR_SIGNATURE | 0x8000000000000);
        }
        OpCodes::ADD => {
            let dest_reg = thread.get_mem::<u8>(ip + 1);
            let src_reg = thread.get_mem::<u8>(ip + 2);
            let dest = VMValue::from(thread.get_reg::<u64>(dest_reg), thread.clone());
            let src = VMValue::from(thread.get_reg::<u64>(src_reg), thread.clone());

            js_impl::add(&thread, dest_reg, src_reg, dest, src).await;
        }
        OpCodes::SUB => {
            let dest_reg = thread.get_mem::<u8>(ip + 1);
            let src_reg = thread.get_mem::<u8>(ip + 2);
            let dest = VMValue::from(thread.get_reg::<u64>(dest_reg), thread.clone());
            let src = VMValue::from(thread.get_reg::<u64>(src_reg), thread.clone());

            js_impl::sub(&thread, dest_reg, src_reg, dest, src).await;
        }
        OpCodes::MUL => {
            let dest_reg = thread.get_mem::<u8>(ip + 1);
            let src_reg = thread.get_mem::<u8>(ip + 2);
            let dest = VMValue::from(thread.get_reg::<u64>(dest_reg), thread.clone());
            let src = VMValue::from(thread.get_reg::<u64>(src_reg), thread.clone());

            js_impl::mul(&thread, dest_reg, src_reg, dest, src).await;
        }
        OpCodes::DIV => {
            let dest_reg = thread.get_mem::<u8>(ip + 1);
            let src_reg = thread.get_mem::<u8>(ip + 2);
            let dest = VMValue::from(thread.get_reg::<u64>(dest_reg), thread.clone());
            let src = VMValue::from(thread.get_reg::<u64>(src_reg), thread.clone());

            js_impl::div(&thread, dest_reg, src_reg, dest, src).await;
        }
        OpCodes::IDIV => {
            let dest_reg = thread.get_mem::<u8>(ip + 1);
            let src_reg = thread.get_mem::<u8>(ip + 2);
            let dest = VMValue::from(thread.get_reg::<u64>(dest_reg), thread.clone());
            let src = VMValue::from(thread.get_reg::<u64>(src_reg), thread.clone());

            js_impl::idiv(&thread, dest_reg, src_reg, dest, src).await;
        }
        OpCodes::REM => {
            let dest_reg = thread.get_mem::<u8>(ip + 1);
            let src_reg = thread.get_mem::<u8>(ip + 2);
            let dest = VMValue::from(thread.get_reg::<u64>(dest_reg), thread.clone());
            let src = VMValue::from(thread.get_reg::<u64>(src_reg), thread.clone());
            
            js_impl::rem(&thread, dest_reg, src_reg, dest, src).await;
        }
        OpCodes::CALL => {
            let addr = thread.get_mem::<u32>(ip + 4);

            thread.push(thread.get_reg::<u64>(6));
            thread.push(thread.get_reg::<u64>(7));
            thread.push(thread.get_reg::<u64>(8));
            thread.push(thread.get_reg::<u64>(9));
            thread.push(thread.get_reg::<u64>(10));
            thread.push(thread.get_reg::<u64>(11));
            thread.push(thread.get_reg::<u64>(12));
            thread.push(thread.get_reg::<u64>(13));
            thread.push(thread.get_reg::<u64>(14));
            thread.push(thread.get_reg::<u64>(15));
            thread.push(ip as u64 + 8);

            thread.set_reg(0, thread.get_reg::<u64>(1) + addr as u64 * 8);
            thread.set_reg(2, thread.get_reg::<u64>(4));

            return (handle_lock::<DROP>(lock), ExecutorBehaviour::None);
        }
        OpCodes::JT => {
            let addr = thread.get_mem::<u32>(ip + 4);

            if thread.get_flag::<0>() {
                thread.set_reg(0, addr);
            }

            return (handle_lock::<DROP>(lock), ExecutorBehaviour::None);
        }
        OpCodes::JMP => {
            let addr = thread.get_mem::<u32>(ip + 4);
            
            thread.set_reg(0, addr);
            
            return (handle_lock::<DROP>(lock), ExecutorBehaviour::None);
        }
        OpCodes::CMP => {
            let v0_reg = thread.get_mem::<u8>(ip + 1);
            let v1_reg = thread.get_mem::<u8>(ip + 2);
            let v0 = VMValue::from(thread.get_reg::<u64>(v0_reg), thread.clone());
            let v1 = VMValue::from(thread.get_reg::<u64>(v1_reg), thread.clone());
            let cmp_type = thread.get_mem::<u8>(ip + 3);

            thread.set_flag::<0>(match cmp_type {
                0 => js_impl::eq(&thread, v0_reg, v1_reg, v0, v1).await,
                1 => !js_impl::eq(&thread, v0_reg, v1_reg, v0, v1).await,
                2 => js_impl::lt(&thread, v0_reg, v1_reg, v0, v1).await,
                3 => js_impl::lte(&thread, v0_reg, v1_reg, v0, v1).await,
                4 => js_impl::gt(&thread, v0_reg, v1_reg, v0, v1).await,
                5 => js_impl::gte(&thread, v0_reg, v1_reg, v0, v1).await,
                _ => panic!("Unsupported Operation")
            });
        }
        OpCodes::PUSHR => {
            let reg = thread.get_mem::<u8>(ip + 1);

            thread.push(thread.get_reg::<u64>(reg));
        }
        OpCodes::PUSHI => {
            let immediate = thread.get_mem::<u64>(ip + 8);
            
            thread.push(immediate);

            thread.inc_inst(8); // Fat instruction
        }
        OpCodes::POP => {
            let reg = thread.get_mem::<u8>(ip + 1);

            thread.set_reg(reg, thread.pop());
        }
        OpCodes::INT => {
            let id = thread.get_mem::<u8>(ip + 1);

            thread.inc_inst(8); // Pre-increase for moving `thread` variable

            return vm_intrinsics::call::<DROP>(thread, id, lock).await;
        }
        OpCodes::ENV => {
            let extension_id = thread.get_mem::<u32>(ip) & 0x00FFFFFF;
            let function_id = thread.get_mem::<u32>(ip + 4);

            thread.inc_inst(8); // Pre-increase for moving `thread` variable

            return thread.get_extension(extension_id).function_call::<DROP>(thread, lock, function_id);
        }
        OpCodes::ENVJ => {
            let extension_id = thread.get_mem::<u32>(ip) & 0x00FFFFFF;
            let interrupt_id = thread.get_mem::<u32>(ip + 4);

            thread.inc_inst(8); // Pre-increase for moving `thread` variable

            return thread.get_extension(extension_id).interrupt_call::<DROP>(thread, lock, interrupt_id);
        }
        OpCodes::SPAWN => {
            let addr = thread.get_mem::<u32>(ip + 4);

            thread.spawn(thread.get_reg::<u64>(1) + addr as u64 * 8).await;
        }
        OpCodes::RET => {
            let fp = thread.get_reg::<u64>(2);
            
            let addr = thread.get_mem_absolute::<u64>(fp as usize + 8);

            thread.set_reg::<u64>(6, fp + 11 * 8);
            thread.set_reg::<u64>(7, fp + 10 * 8);
            thread.set_reg::<u64>(8, fp + 9 * 8);
            thread.set_reg::<u64>(9, fp + 8 * 8);
            thread.set_reg::<u64>(10, fp + 7 * 8);
            thread.set_reg::<u64>(11, fp + 6 * 8);
            thread.set_reg::<u64>(12, fp + 5 * 8);
            thread.set_reg::<u64>(13, fp + 4 * 8);
            thread.set_reg::<u64>(14, fp + 3 * 8);
            thread.set_reg::<u64>(15, fp + 2 * 8);

            thread.set_reg(0, addr);
            thread.set_reg(2, thread.get_reg::<u64>(4) + 88);
            thread.set_reg(4, thread.get_reg::<u64>(2));

            return (handle_lock::<DROP>(lock), ExecutorBehaviour::None);
        }
        OpCodes::SUB32 => {
            let reg = thread.get_mem::<u8>(ip + 1);
            let amount = thread.get_mem::<u32>(ip + 4);

            thread.sub32(reg, amount);
        }
        OpCodes::ADD32 => {
            let reg = thread.get_mem::<u8>(ip + 1);
            let amount = thread.get_mem::<u32>(ip + 4);

            thread.add32(reg, amount);
        }
        OpCodes::END => {
            return (handle_lock::<DROP>(lock), ExecutorBehaviour::Shutdown(ShutdownType::Gracefully));
        }
        OpCodes::DROP => {
            let reg = thread.get_mem::<u8>(ip + 1);
            let data = VMValue::from(thread.get_reg::<u64>(reg), thread.clone());

            if let VMValue::VarStr(vmstr) = data {
                vmstr.drop().await;
            } else {
                panic!("Invalid Operation");
            }

            thread.set_reg::<u64>(reg, STR_SIGNATURE);
        }
        _ => panic!("Unsupported Instruction")
    }

    thread.inc_inst(8);

    (handle_lock::<DROP>(lock), ExecutorBehaviour::None)
}