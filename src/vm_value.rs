use std::fmt::{Display, self};

use crate::{string::VMStr, virtual_thread::VThread};

// This is customization of IEEE-754 Double Precision
// This format makes some variants of NaN represented as 50-bit wide pointer. 

pub const STR_SIGNATURE: u64 = 0b0_11111111111_0100000000000000000000000000000000000000000000000000;
pub const NAN: u64 = 0b0_11111111111_1000000000000000000000000000000000000000000000000000;

pub enum VMValue {
    ConstStr(VMStr),
    VarStr(VMStr),
    Float(f64),
}

impl VMValue {
    pub fn from(value: u64, thread: VThread) -> VMValue {
        if (value & STR_SIGNATURE) == STR_SIGNATURE && (value & 0x7fffffffffffffff) != NAN {      
            if ((value & 0x8000000000000) >> 51) != 0 {
                VMValue::ConstStr(VMStr::from(value & 0x3ffffffffffff, thread))
            } else {
                VMValue::VarStr(VMStr::from(value & 0x3ffffffffffff, thread))
            }
        } else {
            VMValue::Float(f64::from_bits(value))
        }
    }

    pub fn as_str(&mut self) -> Option<(&mut VMStr, bool)> {
        match self {
            VMValue::ConstStr(vm_str) => Some((vm_str, true)),
            VMValue::VarStr(vm_str) => Some((vm_str, false)),
            _ => None
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            &VMValue::Float(v) => Some(v),
            _ => None
        }
    }
}

impl Display for VMValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            &VMValue::Float(v) => write!(f, "0x{:016X} [float     {:.8}]", v.to_bits(), v),
            VMValue::ConstStr(vm_str) => write!(f, "0x{:016X} [const str {}]", vm_str.ptr() as u64, if vm_str.ptr().is_null() {
                String::from("NULL")
            } else {
                format!("`{}`", vm_str.as_str())
            }),
            VMValue::VarStr(vm_str) => {write!(f, "0x{:016X} [temp  str {}]", vm_str.ptr() as u64, if vm_str.ptr().is_null() {
                String::from("NULL")
            } else {
                format!("`{}`", vm_str.as_str())
            })},
        }
    }
}