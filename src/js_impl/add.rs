use crate::{vm_value::{VMValue, STR_SIGNATURE}, string::VMStr, virtual_thread::VThread};

pub async fn add(thread: &VThread, dest_reg: u8, src_reg: u8, mut dest: VMValue, mut src: VMValue) {
    unsafe {
        if let Some((l_str, l_const)) = dest.as_str() {
            if let Some((r_str, r_const)) = src.as_str() {
                if let Some(l) = l_str.parse() && let Some(r) = r_str.parse() {
                    thread.set_reg(dest_reg, (l + r).to_bits());

                    if !l_const { l_str.drop().await; }
                    if !r_const {
                        thread.set_reg::<u64>(src_reg, STR_SIGNATURE);
                        r_str.drop().await;
                    }
                } else {
                    if l_const {
                        thread.set_reg(dest_reg, l_str.cloned_push(r_str).await.as_vm_value());
                    } else {
                        l_str.push(r_str).await;
    
                        thread.set_reg(dest_reg, l_str.as_vm_value());
                    }
                    
                    if !r_const {
                        thread.set_reg::<u64>(src_reg, STR_SIGNATURE);
                        r_str.drop().await;
                    }
                }
            } else {
                let r = src.as_f64().unwrap_unchecked();
    
                if let Some(l) = l_str.parse() {
                    thread.set_reg(dest_reg, (l + r).to_bits());
    
                    if !l_const { l_str.drop().await; }
                } else {
                    let stringified = VMStr::from_str(r.to_string(), thread.clone()).await;
    
                    if l_const {
                        thread.set_reg(dest_reg, l_str.cloned_push(&stringified).await.as_vm_value());
                    } else {
                        l_str.push(&stringified).await;
    
                        thread.set_reg(dest_reg, l_str.as_vm_value());
                    }
    
                    stringified.drop().await;
                }
            }
        } else {
            let l = dest.as_f64().unwrap_unchecked();
    
            if let Some((r_str, r_const)) = src.as_str() {
                if let Some(r) = r_str.parse() {
                    thread.set_reg(dest_reg, (l + r).to_bits());
    
                    if !r_const {
                        thread.set_reg::<u64>(src_reg, STR_SIGNATURE);
                        r_str.drop().await;
                    }
                } else {
                    let mut stringified = VMStr::from_str(l.to_string(), thread.clone()).await;
    
                    stringified.push(&r_str).await;
    
                    thread.set_reg(dest_reg, stringified.as_vm_value());
    
                    if !r_const {
                        thread.set_reg::<u64>(src_reg, STR_SIGNATURE);
                        r_str.drop().await;
                    }
                }
            } else {
                let r = src.as_f64().unwrap_unchecked();
    
                thread.set_reg(dest_reg, (l + r).to_bits());
            }
        }
    }
}