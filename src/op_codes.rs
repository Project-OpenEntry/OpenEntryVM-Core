use crate::utils;

utils::gen_enum!(OpCodes, u8, [
    END   = 0x00,
    MOV   = 0x01,
    LSTR  = 0x02,
    ADD   = 0x03,
    SUB   = 0x04,
    MUL   = 0x05,
    DIV   = 0x06,
    IDIV  = 0x07,
    REM   = 0x08,
    CALL  = 0x09,
    JT    = 0x0A,
    JMP   = 0x0B,
    CMP   = 0x0C,
    PUSHR = 0x0D,
    PUSHI = 0x0E,
    POP   = 0x0F,
    INT   = 0x10,
    ENV   = 0x11,
    ENVJ  = 0x12,
    SPAWN = 0x13,
    RET   = 0x14,
    SUB32 = 0x15,
    ADD32 = 0x16,
    DROP  = 0x17,
]);

utils::gen_enum!(OpLayout, u8, [
    R_R  = 0b10000000,
    R_RO = 0b10010000,
    RO_R = 0b10100000,
    R_I  = 0b10110000,
    RO_I = 0b11000000,
]);