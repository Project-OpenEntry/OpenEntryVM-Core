#[derive(Clone, Copy)]
pub union Register {
    pub r64: u64,
    pub r32: u32,
    pub r16: u16,
    pub r8: u8
}

impl Default for Register {
    fn default() -> Self {
        Register { r64: 0 }
    }
}