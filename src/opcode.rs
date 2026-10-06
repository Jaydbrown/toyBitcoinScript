#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Opcode {
    Op0 = 0x00,
    Op1Negate = 0x4f,
    Op1 = 0x51,
    Op2 = 0x52,
    Op3 = 0x53,
    OpVerify = 0x69,
    OpReturn = 0x6a,
    OpToAltStack = 0x6b,
    OpFromAltStack = 0x6c,
    OpDrop = 0x75,
    OpDup = 0x76,
    OpSwap = 0x78,
    OpEqual = 0x87,
    OpEqualVerify = 0x88,
    Op1Add = 0x8b,
    Op1Sub = 0x8c,
    OpAdd = 0x93,
    OpSub = 0x94,
    OpHash160 = 0xa9,
    OpCheckSig = 0xac,
}

impl TryFrom<u8> for Opcode {
    type Error = &'static str;

    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        match byte {
            0x00 => Ok(Opcode::Op0),
            0x4f => Ok(Opcode::Op1Negate),
            0x51 => Ok(Opcode::Op1),
            0x52 => Ok(Opcode::Op2),
            0x53 => Ok(Opcode::Op3),

            0x69 => Ok(Opcode::OpVerify),
            0x6a => Ok(Opcode::OpReturn),

            0x6b => Ok(Opcode::OpToAltStack),
            0x6c => Ok(Opcode::OpFromAltStack),
            0x75 => Ok(Opcode::OpDrop),
            0x76 => Ok(Opcode::OpDup),
            0x78 => Ok(Opcode::OpSwap),

            0x87 => Ok(Opcode::OpEqual),
            0x88 => Ok(Opcode::OpEqualVerify),

            0x8b => Ok(Opcode::Op1Add),
            0x8c => Ok(Opcode::Op1Sub),
            0x93 => Ok(Opcode::OpAdd),
            0x94 => Ok(Opcode::OpSub),

            0xa9 => Ok(Opcode::OpHash160),
            0xac => Ok(Opcode::OpCheckSig),

            _ => Err("Unknown or unsupported opcode"),
        }
    }
}
