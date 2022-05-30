use crate::{vm_value::{VMValue, STR_SIGNATURE}, virtual_thread::VThread};

pub async fn idiv(thread: &VThread, dest_reg: u8, src_reg: u8, mut dest: VMValue, mut src: VMValue) {
    unsafe {
        if let Some((l_str, l_const)) = dest.as_str() {
            if let Some((r_str, r_const)) = src.as_str() {
                if let Some(l) = l_str.parse() && let Some(r) = r_str.parse() {
                    thread.set_reg(dest_reg, (l / r).floor().to_bits());
                    
                    if !l_const { l_str.drop().await; }
                    if !r_const {
                        thread.set_reg::<u64>(src_reg, STR_SIGNATURE);
                        r_str.drop().await;
                    }
                } else {
                    thread.set_reg(dest_reg, f32::NAN.to_bits());
                    
                    if !r_const {
                        thread.set_reg::<u64>(src_reg, STR_SIGNATURE);
                        r_str.drop().await;
                    }
                }
            } else {
                let r = src.as_f64().unwrap_unchecked();
    
                if let Some(l) = l_str.parse() {
                    thread.set_reg(dest_reg, (l / r).floor().to_bits());
    
                    if !l_const { l_str.drop().await; }
                } else {
                    thread.set_reg(dest_reg, f32::NAN.to_bits());
                }
            }
        } else {
            let l = dest.as_f64().unwrap_unchecked();
    
            if let Some((r_str, r_const)) = src.as_str() {
                if let Some(r) = r_str.parse() {
                    thread.set_reg(dest_reg, (l / r).floor().to_bits());
    
                    if !r_const {
                        thread.set_reg::<u64>(src_reg, STR_SIGNATURE);
                        r_str.drop().await;
                    }
                } else {
                    thread.set_reg(dest_reg, f32::NAN.to_bits());
    
                    if !r_const {
                        thread.set_reg::<u64>(src_reg, STR_SIGNATURE);
                        r_str.drop().await;
                    }
                }
            } else {
                let r = src.as_f64().unwrap_unchecked();
    
                thread.set_reg(dest_reg, (l / r).floor().to_bits());
            }
        }
    }
}