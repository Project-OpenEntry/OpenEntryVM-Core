use crate::utils;

utils::gen_enum!(Intrinsic, u8, [
    Debug = 0x00,
    Sleep = 0x01,
    Restart = 0xFE,
    Throw = 0xFF,
]);