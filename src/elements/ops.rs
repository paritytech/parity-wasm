use std::io;
use super::{
    Serialize, Deserialize, Error, VarUint7, 
    VarUint1, VarUint32, CountedList, BlockType,
    Uint32, VarUint64, Uint64
};

pub struct Opcodes(Vec<Opcode>);

impl Opcodes {
    pub fn elements(&self) -> &[Opcode] { &self.0 }
}

impl Deserialize for Opcodes {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let mut opcodes = Vec::new();

        loop {
            let opcode = Opcode::deserialize(reader)?;
            let is_terminal = opcode.is_terminal();
            opcodes.push(opcode);
            if is_terminal {
                break;
            }
        }

        Ok(Opcodes(opcodes))
    }
}

pub struct InitExpr(Vec<Opcode>);

// todo: check if kind of opcode sequence is valid as an expression
impl Deserialize for InitExpr {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let mut opcodes = Vec::new();

        loop {
            let opcode = Opcode::deserialize(reader)?;
            let is_terminal = opcode.is_terminal();
            opcodes.push(opcode);
            if is_terminal {
                break;
            }
        }

        Ok(InitExpr(opcodes))
    }
}

pub enum Opcode {
    Unreachable,
    Nop,
    Block(BlockType, Opcodes),
    Loop(BlockType, Opcodes),
    If(BlockType, Opcodes),
    Else,
    End,
    Br(u32),
    BrIf(u32),
    BrTable(Vec<u32>, u32),
    Return,

    Call(u32),
    CallIndirect(u32, bool),

    Drop,
    Select,

    GetLocal(u32),
    SetLocal(u32),
    TeeLocal(u32),
    GetGlobal(u32),
    SetGlobal(u32),

    I32Load(u32, u32),
    I64Load(u32, u32),
    F32Load(u32, u32),
    F64Load(u32, u32),
    I32Load8S(u32, u32),
    I32Load8U(u32, u32),
    I32Load16S(u32, u32),
    I32Load16U(u32, u32),
    I64Load8S(u32, u32),
    I64Load8U(u32, u32),
    I64Load16S(u32, u32),
    I64Load16U(u32, u32),
    I64Load32S(u32, u32),
    I64Load32U(u32, u32),
    I32Store(u32, u32),
    I64Store(u32, u32),
    F32Store(u32, u32),
    F64Store(u32, u32),
    I32Store8(u32, u32),
    I32Store16(u32, u32),
    I64Store8(u32, u32),
    I64Store16(u32, u32),
    I64Store32(u32, u32),
    CurrentMemory(bool),
    GrowMemory(bool),

    I32Const(u32),
    I64Const(u64),
    F32Const(u32),
    F64Const(u64),

    I32Eqz,
    I32Eq,
    I32Ne,
    I32LtS,
    I32LtU,
    I32GtS,
    I32GtU,
    I32LeS,
    I32LeU,
    I32GeS,
    I32GeU,
    
    I64Eqz,
    I64Eq,
    I64Ne,
    I64LtS,
    I64LtU,
    I64GtS,
    I64GtU,
    I64LeS,
    I64LeU,
    I64GeS,
    I64GeU,

    F32Eq,
    F32Ne,
    F32Lt,
    F32Gt,
    F32Le,
    F32Ge,

    F64Eq,
    F64Ne,
    F64Lt,
    F64Gt,
    F64Le,
    F64Ge,

    I32Clz,
    I32Ctz,
    I32Popcnt,
    I32Add,
    I32Sub,
    I32Mul,
    I32DivS,
    I32DivU,
    I32RemS,
    I32RemU,
    I32And,
    I32Or,
    I32Xor,
    I32Shl,
    I32ShlS,
    I32ShrU,
    I32Rotl,
    I32Rotr,

    I64Clz,
    I64Ctz,
    I64Popcnt,
    I64Add,
    I64Sub,
    I64Mul,
    I64DivS,
    I64DivU,
    I64RemS,
    I64RemU,
    I64And,
    I64Or,
    I64Xor,
    I64Shl,
    I64ShrS,
    I64ShrU,
    I64Rotl,
    I64Rotr,
    F32Abs,
    F32Neg,
    F32Ceil,
    F32Floor,
    F32Trunc,
    F32Nearest,
    F32Sqrt,
    F32Add,
    F32Sub,
    F32Mul,
    F32Div,
    F32Min,
    F32Max,
    F32Copysign,
    F64Abs,
    F64Neg,
    F64Ceil,
    F64Floor,
    F64Trunc,
    F64Nearest,
    F64Sqrt,
    F64Add,
    F64Sub,
    F64Mul,
    F64Div,
    F64Min,
    F64Max,
    F64Copysign,

    I32WarpI64,
    I32TruncSF32,
    I32TruncUF32,
    I32TruncSF64,
    I32TruncUF64,
    I64ExtendSI32,
    I64ExtendUI32,
    I64TruncSF32,
    I64TruncUF32,
    I64TruncSF64,
    I64TruncUF64,
    F32ConvertSI32,
    F32ConvertUI32,
    F32ConvertSI64,
    F32ConvertUI64,
    F32DemoteF64,
    F64ConvertSI32,
    F64ConvertUI32,
    F64ConvertSI64,
    F64ConvertUI64,
    F64PromoteF32,

    I32ReinterpretF32,
    I64ReinterpretF64,
    F32ReinterpretI32,
    F64ReinterpretI64,
}

impl Opcode {
    pub fn is_terminal(&self) -> bool { 
        match self {
            &Opcode::End => true,
            _ => false,
        }
    }
}

impl Deserialize for Opcode {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        use self::Opcode::*;

        let val: u8 = VarUint7::deserialize(reader)?.into();

        Ok(
            match val {
                0x00 => Unreachable,
                0x01 => Nop,
                0x02 => Block(BlockType::deserialize(reader)?, Opcodes::deserialize(reader)?),
                0x03 => Loop(BlockType::deserialize(reader)?, Opcodes::deserialize(reader)?),
                0x04 => If(BlockType::deserialize(reader)?, Opcodes::deserialize(reader)?),
                0x05 => Else,
                0x0b => End,

                0x0c => Br(VarUint32::deserialize(reader)?.into()),
                0x0d => BrIf(VarUint32::deserialize(reader)?.into()),
                0x0e => { 
                    let t1: Vec<u32> = CountedList::<VarUint32>::deserialize(reader)?
                        .into_inner()
                        .into_iter()
                        .map(Into::into)
                        .collect();

                    BrTable(t1, VarUint32::deserialize(reader)?.into())
                },
                0x0f => Return,
                0x10 => Call(VarUint32::deserialize(reader)?.into()),
                0x11 => CallIndirect(
                    VarUint32::deserialize(reader)?.into(), 
                    VarUint1::deserialize(reader)?.into()),
                0x1a => Drop,
                0x1b => Select,

                0x20 => GetLocal(VarUint32::deserialize(reader)?.into()),
                0x21 => SetLocal(VarUint32::deserialize(reader)?.into()),
                0x22 => TeeLocal(VarUint32::deserialize(reader)?.into()),
                0x23 => GetGlobal(VarUint32::deserialize(reader)?.into()),
                0x24 => SetGlobal(VarUint32::deserialize(reader)?.into()),

                0x28 => I32Load(
                    VarUint32::deserialize(reader)?.into(), 
                    VarUint32::deserialize(reader)?.into()),

                0x29 => I64Load(
                    VarUint32::deserialize(reader)?.into(), 
                    VarUint32::deserialize(reader)?.into()),

                0x2a => F32Load(
                    VarUint32::deserialize(reader)?.into(), 
                    VarUint32::deserialize(reader)?.into()),

                0x2b => F64Load(
                    VarUint32::deserialize(reader)?.into(), 
                    VarUint32::deserialize(reader)?.into()),

                0x2c => I32Load8S(
                    VarUint32::deserialize(reader)?.into(), 
                    VarUint32::deserialize(reader)?.into()),

                0x2d => I32Load8U(
                    VarUint32::deserialize(reader)?.into(), 
                    VarUint32::deserialize(reader)?.into()),

                0x2e => I32Load16S(
                    VarUint32::deserialize(reader)?.into(), 
                    VarUint32::deserialize(reader)?.into()),

                0x2f => I32Load16U(
                    VarUint32::deserialize(reader)?.into(), 
                    VarUint32::deserialize(reader)?.into()),

                0x30 => I64Load8S(
                    VarUint32::deserialize(reader)?.into(), 
                    VarUint32::deserialize(reader)?.into()),

                0x31 => I64Load8U(
                    VarUint32::deserialize(reader)?.into(), 
                    VarUint32::deserialize(reader)?.into()),

                0x32 => I64Load16S(
                    VarUint32::deserialize(reader)?.into(), 
                    VarUint32::deserialize(reader)?.into()),

                0x33 => I64Load16U(
                    VarUint32::deserialize(reader)?.into(), 
                    VarUint32::deserialize(reader)?.into()),

                0x34 => I64Load32S(
                    VarUint32::deserialize(reader)?.into(), 
                    VarUint32::deserialize(reader)?.into()),

                0x35 => I64Load32U(
                    VarUint32::deserialize(reader)?.into(), 
                    VarUint32::deserialize(reader)?.into()),

                0x36 => I32Store(
                    VarUint32::deserialize(reader)?.into(), 
                    VarUint32::deserialize(reader)?.into()),

                0x37 => I64Store(
                    VarUint32::deserialize(reader)?.into(), 
                    VarUint32::deserialize(reader)?.into()),

                0x38 => F32Store(
                    VarUint32::deserialize(reader)?.into(), 
                    VarUint32::deserialize(reader)?.into()),

                0x39 => F64Store(
                    VarUint32::deserialize(reader)?.into(), 
                    VarUint32::deserialize(reader)?.into()),

                0x3a => I32Store8(
                    VarUint32::deserialize(reader)?.into(), 
                    VarUint32::deserialize(reader)?.into()),

                0x3b => I32Store16(
                    VarUint32::deserialize(reader)?.into(), 
                    VarUint32::deserialize(reader)?.into()),

                0x3c => I64Store8(
                    VarUint32::deserialize(reader)?.into(), 
                    VarUint32::deserialize(reader)?.into()),

                0x3d => I64Store16(
                    VarUint32::deserialize(reader)?.into(), 
                    VarUint32::deserialize(reader)?.into()),

                0x3e => I64Store32(
                    VarUint32::deserialize(reader)?.into(), 
                    VarUint32::deserialize(reader)?.into()),


                0x3f => CurrentMemory(VarUint1::deserialize(reader)?.into()),
                0x40 => GrowMemory(VarUint1::deserialize(reader)?.into()),

                0x41 => I32Const(VarUint32::deserialize(reader)?.into()),
                0x42 => I64Const(VarUint64::deserialize(reader)?.into()),
                0x43 => F32Const(Uint32::deserialize(reader)?.into()),
                0x44 => F64Const(Uint64::deserialize(reader)?.into()),
                0x45 => I32Eqz,
                0x46 => I32Eq,
                0x47 => I32Ne,
                0x48 => I32LtS,
                0x49 => I32LtU,
                0x4a => I32GtS,
                0x4b => I32GtU,
                0x4c => I32LeS,
                0x4d => I32LeU,
                0x4e => I32GeS,
                0x4f => I32GeU,
                
                0x50 => I64Eqz,
                0x51 => I64Eq,
                0x52 => I64Ne,
                0x53 => I64LtS,
                0x54 => I64LtU,
                0x55 => I64GtS,
                0x56 => I64GtU,
                0x57 => I64LeS,
                0x58 => I64LeU,
                0x59 => I64GeS,
                0x5a => I64GeU,

                0x5b => F32Eq,
                0x5c => F32Ne,
                0x5d => F32Lt,
                0x5e => F32Gt,
                0x5f => F32Le,
                0x60 => F32Ge,

                0x61 => F64Eq,
                0x62 => F64Ne,
                0x63 => F64Lt,
                0x64 => F64Gt,
                0x65 => F64Le,
                0x66 => F64Ge,

                0x67 => I32Clz,
                0x68 => I32Ctz,
                0x69 => I32Popcnt,
                0x6a => I32Add,
                0x6b => I32Sub,
                0x6c => I32Mul,
                0x6d => I32DivS,
                0x6e => I32DivU,
                0x6f => I32RemS,
                0x70 => I32RemU,
                0x71 => I32And,
                0x72 => I32Or,
                0x73 => I32Xor,
                0x74 => I32Shl,
                0x75 => I32ShlS,
                0x76 => I32ShrU,
                0x77 => I32Rotl,
                0x78 => I32Rotr,

                0x79 => I64Clz,
                0x7a => I64Ctz,
                0x7b => I64Popcnt,
                0x7c => I64Add,
                0x7d => I64Sub,
                0x7e => I64Mul,
                0x7f => I64DivS,
                0x80 => I64DivU,
                0x81 => I64RemS,
                0x82 => I64RemU,
                0x83 => I64And,
                0x84 => I64Or,
                0x85 => I64Xor,
                0x86 => I64Shl,
                0x87 => I64ShrS,
                0x88 => I64ShrU,
                0x89 => I64Rotl,
                0x8a => I64Rotr,
                0x8b => F32Abs,
                0x8c => F32Neg,
                0x8d => F32Ceil,
                0x8e => F32Floor,
                0x8f => F32Trunc,
                0x90 => F32Nearest,
                0x91 => F32Sqrt,
                0x92 => F32Add,
                0x93 => F32Sub,
                0x94 => F32Mul,
                0x95 => F32Div,
                0x96 => F32Min,
                0x97 => F32Max,
                0x98 => F32Copysign,
                0x99 => F64Abs,
                0x9a => F64Neg,
                0x9b => F64Ceil,
                0x9c => F64Floor,
                0x9d => F64Trunc,
                0x9e => F64Nearest,
                0x9f => F64Sqrt,
                0xa0 => F64Add,
                0xa1 => F64Sub,
                0xa2 => F64Mul,
                0xa3 => F64Div,
                0xa4 => F64Min,
                0xa5 => F64Max,
                0xa6 => F64Copysign,

                0xa7 => I32WarpI64,
                0xa8 => I32TruncSF32,
                0xa9 => I32TruncUF32,
                0xaa => I32TruncSF64,
                0xab => I32TruncUF64,
                0xac => I64ExtendSI32,
                0xad => I64ExtendUI32,
                0xae => I64TruncSF32,
                0xaf => I64TruncUF32,
                0xb0 => I64TruncSF64,
                0xb1 => I64TruncUF64,
                0xb2 => F32ConvertSI32,
                0xb3 => F32ConvertUI32,
                0xb4 => F32ConvertSI64,
                0xb5 => F32ConvertUI64,
                0xb6 => F32DemoteF64,
                0xb7 => F64ConvertSI32,
                0xb8 => F64ConvertUI32,
                0xb9 => F64ConvertSI64,
                0xba => F64ConvertUI64,
                0xbb => F64PromoteF32,

                0xbc => I32ReinterpretF32,
                0xbd => I64ReinterpretF64,
                0xbe => F32ReinterpretI32,
                0xbf => F64ReinterpretI64,

                _ => { return Err(Error::UnknownOpcode(val)); }
            }
        )
    }
}

macro_rules! op {
    ($writer: expr, $byte: expr) => ({
        let b: u8 = $byte;
        $writer.write_all(&[b])?;
    });
    ($writer: expr, $byte: expr, $s: block) => ({
        op!($writer, $byte);
        $s;
    });
}

impl Serialize for Opcode {
    type Error = Error;
    
    fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
        use self::Opcode::*;

        match self {
            Unreachable => op!(writer, 0x00),
            Nop => op!(writer, 0x01),
            Block(block_type, ops) => op!(writer, 0x02, {
               block_type.serialize(writer)?;
               ops.serialize(writer)?;
            }),            
            End => op!(writer, 0x0b),           
            _ => unreachable!(),
        }

        Ok(())
    }
    
}

impl Serialize for Opcodes {
    type Error = Error;
    
    fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
        for op in self.0.into_iter() {
            op.serialize(writer)?;
        }

        Ok(())
    }
}

impl Serialize for InitExpr {
    type Error = Error;
    
    fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
        for op in self.0.into_iter() {
            op.serialize(writer)?;
        }

        Ok(())
    }
}

