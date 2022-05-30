macro_rules! js_impl {
    ($name: ident, $operator: tt) => {
        pub async fn $name(thread: &crate::virtual_thread::VThread, dest_reg: u8, src_reg: u8, mut dest: crate::vm_value::VMValue, mut src: crate::vm_value::VMValue) {
            unsafe {
                if let Some((l_str, l_const)) = dest.as_str() {
                    if let Some((r_str, r_const)) = src.as_str() {
                        if let Some(l) = l_str.parse() && let Some(r) = r_str.parse() {
                            thread.set_reg(dest_reg, (l $operator r).to_bits());
                            
                            if !l_const { l_str.drop().await; }
                            if !r_const {
                                thread.set_reg::<u64>(src_reg, crate::vm_value::STR_SIGNATURE);
                                r_str.drop().await;
                            }
                        } else {
                            thread.set_reg(dest_reg, f32::NAN.to_bits());
                            
                            if !r_const {
                                thread.set_reg::<u64>(src_reg, crate::vm_value::STR_SIGNATURE);
                                r_str.drop().await;
                            }
                        }
                    } else {
                        let r = src.as_f64().unwrap_unchecked();
            
                        if let Some(l) = l_str.parse() {
                            thread.set_reg(dest_reg, (l $operator r).to_bits());
            
                            if !l_const { l_str.drop().await; }
                        } else {
                            thread.set_reg(dest_reg, f32::NAN.to_bits());
                        }
                    }
                } else {
                    let l = dest.as_f64().unwrap_unchecked();
            
                    if let Some((r_str, r_const)) = src.as_str() {
                        if let Some(r) = r_str.parse() {
                            thread.set_reg(dest_reg, (l $operator r).to_bits());
            
                            if !r_const {
                                thread.set_reg::<u64>(src_reg, crate::vm_value::STR_SIGNATURE);
                                r_str.drop().await;
                            }
                        } else {
                            thread.set_reg(dest_reg, f32::NAN.to_bits());
            
                            if !r_const {
                                thread.set_reg::<u64>(src_reg, crate::vm_value::STR_SIGNATURE);
                                r_str.drop().await;
                            }
                        }
                    } else {
                        let r = src.as_f64().unwrap_unchecked();
            
                        thread.set_reg(dest_reg, (l $operator r).to_bits());
                    }
                }
            }
        }
    };
}

macro_rules! js_impl_cmp {
    ($name: ident, $operator: tt) => {
        pub async fn $name(thread: &crate::virtual_thread::VThread, dest_reg: u8, src_reg: u8, mut v0: crate::vm_value::VMValue, mut v1: crate::vm_value::VMValue) -> bool {
            unsafe {
                if let Some((l_str, l_const)) = v0.as_str() {
                    if let Some((r_str, r_const)) = v1.as_str() {
                        if let Some(l) = l_str.parse() && let Some(r) = r_str.parse() {
                            if !r_const {
                                thread.set_reg::<u64>(src_reg, crate::vm_value::STR_SIGNATURE);
                                r_str.drop().await;
                            }
                            if !l_const {
                                thread.set_reg::<u64>(dest_reg, crate::vm_value::STR_SIGNATURE);
                                l_str.drop().await;
                            }

                            l $operator r
                        } else {
                            if !r_const {
                                thread.set_reg::<u64>(src_reg, crate::vm_value::STR_SIGNATURE);
                                r_str.drop().await;
                            }
                            
                            false
                        }
                    } else {
                        let r = v1.as_f64().unwrap_unchecked();
            
                        if let Some(l) = l_str.parse() {
                            if !l_const {
                                thread.set_reg::<u64>(dest_reg, crate::vm_value::STR_SIGNATURE);
                                l_str.drop().await;
                            }

                            l $operator r
                        } else {
                            false
                        }
                    }
                } else {
                    let l = v0.as_f64().unwrap_unchecked();
            
                    if let Some((r_str, r_const)) = v1.as_str() {
                        if let Some(r) = r_str.parse() {
                            if !r_const {
                                thread.set_reg::<u64>(src_reg, crate::vm_value::STR_SIGNATURE);
                                r_str.drop().await;
                            }

                            l $operator r
                        } else {
                            if !r_const {
                                thread.set_reg::<u64>(src_reg, crate::vm_value::STR_SIGNATURE);
                                r_str.drop().await;
                            }

                            false
                        }
                    } else {
                        let r = v1.as_f64().unwrap_unchecked();
            
                        l $operator r
                    }
                }
            }
        }
    };
}

pub(crate) use js_impl;
pub(crate) use js_impl_cmp;