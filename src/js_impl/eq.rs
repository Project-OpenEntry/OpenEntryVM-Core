use crate::{vm_value::{VMValue, STR_SIGNATURE}, string::VMStr, virtual_thread::VThread};

pub async fn eq(thread: &VThread, dest_reg: u8, src_reg: u8, mut v0: VMValue, mut v1: VMValue) -> bool {
    unsafe {
        if let Some((l_str, l_const)) = v0.as_str() {
            if let Some((r_str, r_const)) = v1.as_str() {
                if let Some(l) = l_str.parse() && let Some(r) = r_str.parse() {
                    if !r_const {
                        thread.set_reg::<u64>(src_reg, STR_SIGNATURE);
                        r_str.drop().await;
                    }
                    if !l_const {
                        thread.set_reg::<u64>(dest_reg, STR_SIGNATURE);
                        l_str.drop().await;
                    }

                    l == r
                } else {
                    let value = VMStr::str_eq(l_str, r_str);

                    if !r_const {
                        thread.set_reg::<u64>(src_reg, STR_SIGNATURE);
                        r_str.drop().await;
                    }

                    value
                }
            } else {
                let r = v1.as_f64().unwrap_unchecked();
    
                if let Some(l) = l_str.parse() {
                    if !l_const {
                        thread.set_reg::<u64>(dest_reg, STR_SIGNATURE);
                        l_str.drop().await;
                    }

                    l == r
                } else {
                    let stringified = VMStr::from_str(r.to_string(), thread.clone()).await;
                    let value = VMStr::str_eq(l_str, &stringified);

                    stringified.drop().await;

                    value
                }
            }
        } else {
            let l = v0.as_f64().unwrap_unchecked();
    
            if let Some((r_str, r_const)) = v1.as_str() {
                if let Some(r) = r_str.parse() {
                    if !r_const {
                        thread.set_reg::<u64>(src_reg, STR_SIGNATURE);
                        r_str.drop().await;
                    }

                    l == r
                } else {
                    let stringified = VMStr::from_str(l.to_string(), thread.clone()).await;
                    let value = VMStr::str_eq(&stringified, r_str);

                    stringified.drop().await;

                    if !r_const {
                        thread.set_reg::<u64>(src_reg, STR_SIGNATURE);
                        r_str.drop().await;
                    }

                    value
                }
            } else {
                let r = v1.as_f64().unwrap_unchecked();
    
                l == r
            }
        }
    }
}