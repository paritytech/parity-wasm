use std::fmt;
use std::vec::Vec;
use std::boxed::Box;
use io;
use super::{
	Serialize, Deserialize, Error,
	Uint8, VarUint32, CountedList, BlockType,
	Uint32, Uint64, CountedListWriter,
	VarInt32, VarInt64,
};

/// List of instructions (usually inside a block section).
#[derive(Debug, Clone, PartialEq)]
pub struct Instructions(Vec<Instruction>);

impl Instructions {
	/// New list of instructions from vector of instructions.
	pub fn new(elements: Vec<Instruction>) -> Self {
		Instructions(elements)
	}

	/// Empty expression with only `Instruction::End` instruction.
	pub fn empty() -> Self {
		Instructions(vec![Instruction::End])
	}

	/// List of individual instructions.
	pub fn elements(&self) -> &[Instruction] { &self.0 }

	/// Individual instructions, mutable.
	pub fn elements_mut(&mut self) -> &mut Vec<Instruction> { &mut self.0 }
}

impl Deserialize for Instructions {
	type Error = Error;

	fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
		let mut instructions = Vec::new();
		let mut block_count = 1usize;

		loop {
			let instruction = Instruction::deserialize(reader)?;
			if instruction.is_terminal() {
				block_count -= 1;
			} else if instruction.is_block() {
				block_count = block_count.checked_add(1).ok_or(Error::Other("too many instructions"))?;
			}

			instructions.push(instruction);
			if block_count == 0 {
				break;
			}
		}

		Ok(Instructions(instructions))
	}
}

/// Initialization expression.
#[derive(Debug, Clone, PartialEq)]
pub struct InitExpr(Vec<Instruction>);

impl InitExpr {
	/// New initialization expression from instruction list.
	///   `code` must end with the `Instruction::End` instruction!
	pub fn new(code: Vec<Instruction>) -> Self {
		InitExpr(code)
	}

	/// Empty expression with only `Instruction::End` instruction
	pub fn empty() -> Self {
		InitExpr(vec![Instruction::End])
	}

	/// List of instructions used in the expression.
	pub fn code(&self) -> &[Instruction] {
		&self.0
	}

	/// List of instructions used in the expression.
	pub fn code_mut(&mut self) -> &mut Vec<Instruction> {
		&mut self.0
	}
}

impl Deserialize for InitExpr {
	type Error = Error;

	fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
		let mut instructions = Vec::new();

		loop {
			let instruction = Instruction::deserialize(reader)?;
			let is_terminal = instruction.is_terminal();
			instructions.push(instruction);
			if is_terminal {
				break;
			}
		}

		Ok(InitExpr(instructions))
	}
}

/// Instruction
#[derive(Clone, Debug, PartialEq)]
#[allow(missing_docs)]
pub enum Instruction {
	Unreachable,
	Nop,
	Block(BlockType),
	Loop(BlockType),
	If(BlockType),
	Else,
	End,
	Br(u32),
	BrIf(u32),
	BrTable(Box<[u32]>, u32),
	Return,

	Call(u32),
	CallIndirect(u32, u8),

	Drop,
	Select,

	GetLocal(u32),
	SetLocal(u32),
	TeeLocal(u32),
	GetGlobal(u32),
	SetGlobal(u32),

	// All store/load instructions operate with 'memory immediates'
	// which represented here as (flag, offset) tuple
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

	CurrentMemory(u8),
	GrowMemory(u8),

	I32Const(i32),
	I64Const(i64),
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
	I32ShrS,
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

	I32WrapI64,
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

impl Instruction {
	/// Is this instruction starts the new block (which should end with terminal instruction).
	pub fn is_block(&self) -> bool {
		match self {
			&Instruction::Block(_) | &Instruction::Loop(_) | &Instruction::If(_) => true,
			_ => false,
		}
	}

	/// Is this instruction determines the termination of instruction sequence
	/// `true` for `Instruction::End`
	pub fn is_terminal(&self) -> bool {
		match self {
			&Instruction::End => true,
			_ => false,
		}
	}
}

#[allow(missing_docs)]
pub mod opcodes {
	pub const UNREACHABLE: u8 = 0x00;
	pub const NOP: u8 = 0x01;
	pub const BLOCK: u8 = 0x02;
	pub const LOOP: u8 = 0x03;
	pub const IF: u8 = 0x04;
	pub const ELSE: u8 = 0x05;
	pub const END: u8 = 0x0b;
	pub const BR: u8 = 0x0c;
	pub const BRIF: u8 = 0x0d;
	pub const BRTABLE: u8 = 0x0e;
	pub const RETURN: u8 = 0x0f;
	pub const CALL: u8 = 0x10;
	pub const CALLINDIRECT: u8 = 0x11;
	pub const DROP: u8 = 0x1a;
	pub const SELECT: u8 = 0x1b;
	pub const GETLOCAL: u8 = 0x20;
	pub const SETLOCAL: u8 = 0x21;
	pub const TEELOCAL: u8 = 0x22;
	pub const GETGLOBAL: u8 = 0x23;
	pub const SETGLOBAL: u8 = 0x24;
	pub const I32LOAD: u8 = 0x28;
	pub const I64LOAD: u8 = 0x29;
	pub const F32LOAD: u8 = 0x2a;
	pub const F64LOAD: u8 = 0x2b;
	pub const I32LOAD8S: u8 = 0x2c;
	pub const I32LOAD8U: u8 = 0x2d;
	pub const I32LOAD16S: u8 = 0x2e;
	pub const I32LOAD16U: u8 = 0x2f;
	pub const I64LOAD8S: u8 = 0x30;
	pub const I64LOAD8U: u8 = 0x31;
	pub const I64LOAD16S: u8 = 0x32;
	pub const I64LOAD16U: u8 = 0x33;
	pub const I64LOAD32S: u8 = 0x34;
	pub const I64LOAD32U: u8 = 0x35;
	pub const I32STORE: u8 = 0x36;
	pub const I64STORE: u8 = 0x37;
	pub const F32STORE: u8 = 0x38;
	pub const F64STORE: u8 = 0x39;
	pub const I32STORE8: u8 = 0x3a;
	pub const I32STORE16: u8 = 0x3b;
	pub const I64STORE8: u8 = 0x3c;
	pub const I64STORE16: u8 = 0x3d;
	pub const I64STORE32: u8 = 0x3e;
	pub const CURRENTMEMORY: u8 = 0x3f;
	pub const GROWMEMORY: u8 = 0x40;
	pub const I32CONST: u8 = 0x41;
	pub const I64CONST: u8 = 0x42;
	pub const F32CONST: u8 = 0x43;
	pub const F64CONST: u8 = 0x44;
	pub const I32EQZ: u8 = 0x45;
	pub const I32EQ: u8 = 0x46;
	pub const I32NE: u8 = 0x47;
	pub const I32LTS: u8 = 0x48;
	pub const I32LTU: u8 = 0x49;
	pub const I32GTS: u8 = 0x4a;
	pub const I32GTU: u8 = 0x4b;
	pub const I32LES: u8 = 0x4c;
	pub const I32LEU: u8 = 0x4d;
	pub const I32GES: u8 = 0x4e;
	pub const I32GEU: u8 = 0x4f;
	pub const I64EQZ: u8 = 0x50;
	pub const I64EQ: u8 = 0x51;
	pub const I64NE: u8 = 0x52;
	pub const I64LTS: u8 = 0x53;
	pub const I64LTU: u8 = 0x54;
	pub const I64GTS: u8 = 0x55;
	pub const I64GTU: u8 = 0x56;
	pub const I64LES: u8 = 0x57;
	pub const I64LEU: u8 = 0x58;
	pub const I64GES: u8 = 0x59;
	pub const I64GEU: u8 = 0x5a;

	pub const F32EQ: u8 = 0x5b;
	pub const F32NE: u8 = 0x5c;
	pub const F32LT: u8 = 0x5d;
	pub const F32GT: u8 = 0x5e;
	pub const F32LE: u8 = 0x5f;
	pub const F32GE: u8 = 0x60;

	pub const F64EQ: u8 = 0x61;
	pub const F64NE: u8 = 0x62;
	pub const F64LT: u8 = 0x63;
	pub const F64GT: u8 = 0x64;
	pub const F64LE: u8 = 0x65;
	pub const F64GE: u8 = 0x66;

	pub const I32CLZ: u8 = 0x67;
	pub const I32CTZ: u8 = 0x68;
	pub const I32POPCNT: u8 = 0x69;
	pub const I32ADD: u8 = 0x6a;
	pub const I32SUB: u8 = 0x6b;
	pub const I32MUL: u8 = 0x6c;
	pub const I32DIVS: u8 = 0x6d;
	pub const I32DIVU: u8 = 0x6e;
	pub const I32REMS: u8 = 0x6f;
	pub const I32REMU: u8 = 0x70;
	pub const I32AND: u8 = 0x71;
	pub const I32OR: u8 = 0x72;
	pub const I32XOR: u8 = 0x73;
	pub const I32SHL: u8 = 0x74;
	pub const I32SHRS: u8 = 0x75;
	pub const I32SHRU: u8 = 0x76;
	pub const I32ROTL: u8 = 0x77;
	pub const I32ROTR: u8 = 0x78;

	pub const I64CLZ: u8 = 0x79;
	pub const I64CTZ: u8 = 0x7a;
	pub const I64POPCNT: u8 = 0x7b;
	pub const I64ADD: u8 = 0x7c;
	pub const I64SUB: u8 = 0x7d;
	pub const I64MUL: u8 = 0x7e;
	pub const I64DIVS: u8 = 0x7f;
	pub const I64DIVU: u8 = 0x80;
	pub const I64REMS: u8 = 0x81;
	pub const I64REMU: u8 = 0x82;
	pub const I64AND: u8 = 0x83;
	pub const I64OR: u8 = 0x84;
	pub const I64XOR: u8 = 0x85;
	pub const I64SHL: u8 = 0x86;
	pub const I64SHRS: u8 = 0x87;
	pub const I64SHRU: u8 = 0x88;
	pub const I64ROTL: u8 = 0x89;
	pub const I64ROTR: u8 = 0x8a;
	pub const F32ABS: u8 = 0x8b;
	pub const F32NEG: u8 = 0x8c;
	pub const F32CEIL: u8 = 0x8d;
	pub const F32FLOOR: u8 = 0x8e;
	pub const F32TRUNC: u8 = 0x8f;
	pub const F32NEAREST: u8 = 0x90;
	pub const F32SQRT: u8 = 0x91;
	pub const F32ADD: u8 = 0x92;
	pub const F32SUB: u8 = 0x93;
	pub const F32MUL: u8 = 0x94;
	pub const F32DIV: u8 = 0x95;
	pub const F32MIN: u8 = 0x96;
	pub const F32MAX: u8 = 0x97;
	pub const F32COPYSIGN: u8 = 0x98;
	pub const F64ABS: u8 = 0x99;
	pub const F64NEG: u8 = 0x9a;
	pub const F64CEIL: u8 = 0x9b;
	pub const F64FLOOR: u8 = 0x9c;
	pub const F64TRUNC: u8 = 0x9d;
	pub const F64NEAREST: u8 = 0x9e;
	pub const F64SQRT: u8 = 0x9f;
	pub const F64ADD: u8 = 0xa0;
	pub const F64SUB: u8 = 0xa1;
	pub const F64MUL: u8 = 0xa2;
	pub const F64DIV: u8 = 0xa3;
	pub const F64MIN: u8 = 0xa4;
	pub const F64MAX: u8 = 0xa5;
	pub const F64COPYSIGN: u8 = 0xa6;

	pub const I32WRAPI64: u8 = 0xa7;
	pub const I32TRUNCSF32: u8 = 0xa8;
	pub const I32TRUNCUF32: u8 = 0xa9;
	pub const I32TRUNCSF64: u8 = 0xaa;
	pub const I32TRUNCUF64: u8 = 0xab;
	pub const I64EXTENDSI32: u8 = 0xac;
	pub const I64EXTENDUI32: u8 = 0xad;
	pub const I64TRUNCSF32: u8 = 0xae;
	pub const I64TRUNCUF32: u8 = 0xaf;
	pub const I64TRUNCSF64: u8 = 0xb0;
	pub const I64TRUNCUF64: u8 = 0xb1;
	pub const F32CONVERTSI32: u8 = 0xb2;
	pub const F32CONVERTUI32: u8 = 0xb3;
	pub const F32CONVERTSI64: u8 = 0xb4;
	pub const F32CONVERTUI64: u8 = 0xb5;
	pub const F32DEMOTEF64: u8 = 0xb6;
	pub const F64CONVERTSI32: u8 = 0xb7;
	pub const F64CONVERTUI32: u8 = 0xb8;
	pub const F64CONVERTSI64: u8 = 0xb9;
	pub const F64CONVERTUI64: u8 = 0xba;
	pub const F64PROMOTEF32: u8 = 0xbb;

	pub const I32REINTERPRETF32: u8 = 0xbc;
	pub const I64REINTERPRETF64: u8 = 0xbd;
	pub const F32REINTERPRETI32: u8 = 0xbe;
	pub const F64REINTERPRETI64: u8 = 0xbf;
}

impl Deserialize for Instruction {
	type Error = Error;

	fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
		use self::Instruction::*;
		use self::opcodes::*;

		let val: u8 = Uint8::deserialize(reader)?.into();

		Ok(
			match val {
				UNREACHABLE => Unreachable,
				NOP => Nop,
				BLOCK => Block(BlockType::deserialize(reader)?),
				LOOP => Loop(BlockType::deserialize(reader)?),
				IF => If(BlockType::deserialize(reader)?),
				ELSE => Else,
				END => End,

				BR => Br(VarUint32::deserialize(reader)?.into()),
				BRIF => BrIf(VarUint32::deserialize(reader)?.into()),
				BRTABLE => {
					let t1: Vec<u32> = CountedList::<VarUint32>::deserialize(reader)?
						.into_inner()
						.into_iter()
						.map(Into::into)
						.collect();

					BrTable(t1.into_boxed_slice(), VarUint32::deserialize(reader)?.into())
				},
				RETURN => Return,
				CALL => Call(VarUint32::deserialize(reader)?.into()),
				CALLINDIRECT => {
					let signature: u32 = VarUint32::deserialize(reader)?.into();
					let table_ref: u8 = Uint8::deserialize(reader)?.into();
					if table_ref != 0 { return Err(Error::InvalidTableReference(table_ref)); }

					CallIndirect(
						signature,
						table_ref,
					)
				},
				DROP => Drop,
				SELECT => Select,

				GETLOCAL => GetLocal(VarUint32::deserialize(reader)?.into()),
				SETLOCAL => SetLocal(VarUint32::deserialize(reader)?.into()),
				TEELOCAL => TeeLocal(VarUint32::deserialize(reader)?.into()),
				GETGLOBAL => GetGlobal(VarUint32::deserialize(reader)?.into()),
				SETGLOBAL => SetGlobal(VarUint32::deserialize(reader)?.into()),

				I32LOAD => I32Load(
					VarUint32::deserialize(reader)?.into(),
					VarUint32::deserialize(reader)?.into()),

				I64LOAD => I64Load(
					VarUint32::deserialize(reader)?.into(),
					VarUint32::deserialize(reader)?.into()),

				F32LOAD => F32Load(
					VarUint32::deserialize(reader)?.into(),
					VarUint32::deserialize(reader)?.into()),

				F64LOAD => F64Load(
					VarUint32::deserialize(reader)?.into(),
					VarUint32::deserialize(reader)?.into()),

				I32LOAD8S => I32Load8S(
					VarUint32::deserialize(reader)?.into(),
					VarUint32::deserialize(reader)?.into()),

				I32LOAD8U => I32Load8U(
					VarUint32::deserialize(reader)?.into(),
					VarUint32::deserialize(reader)?.into()),

				I32LOAD16S => I32Load16S(
					VarUint32::deserialize(reader)?.into(),
					VarUint32::deserialize(reader)?.into()),

				I32LOAD16U => I32Load16U(
					VarUint32::deserialize(reader)?.into(),
					VarUint32::deserialize(reader)?.into()),

				I64LOAD8S => I64Load8S(
					VarUint32::deserialize(reader)?.into(),
					VarUint32::deserialize(reader)?.into()),

				I64LOAD8U => I64Load8U(
					VarUint32::deserialize(reader)?.into(),
					VarUint32::deserialize(reader)?.into()),

				I64LOAD16S => I64Load16S(
					VarUint32::deserialize(reader)?.into(),
					VarUint32::deserialize(reader)?.into()),

				I64LOAD16U => I64Load16U(
					VarUint32::deserialize(reader)?.into(),
					VarUint32::deserialize(reader)?.into()),

				I64LOAD32S => I64Load32S(
					VarUint32::deserialize(reader)?.into(),
					VarUint32::deserialize(reader)?.into()),

				I64LOAD32U => I64Load32U(
					VarUint32::deserialize(reader)?.into(),
					VarUint32::deserialize(reader)?.into()),

				I32STORE => I32Store(
					VarUint32::deserialize(reader)?.into(),
					VarUint32::deserialize(reader)?.into()),

				I64STORE => I64Store(
					VarUint32::deserialize(reader)?.into(),
					VarUint32::deserialize(reader)?.into()),

				F32STORE => F32Store(
					VarUint32::deserialize(reader)?.into(),
					VarUint32::deserialize(reader)?.into()),

				F64STORE => F64Store(
					VarUint32::deserialize(reader)?.into(),
					VarUint32::deserialize(reader)?.into()),

				I32STORE8 => I32Store8(
					VarUint32::deserialize(reader)?.into(),
					VarUint32::deserialize(reader)?.into()),

				I32STORE16 => I32Store16(
					VarUint32::deserialize(reader)?.into(),
					VarUint32::deserialize(reader)?.into()),

				I64STORE8 => I64Store8(
					VarUint32::deserialize(reader)?.into(),
					VarUint32::deserialize(reader)?.into()),

				I64STORE16 => I64Store16(
					VarUint32::deserialize(reader)?.into(),
					VarUint32::deserialize(reader)?.into()),

				I64STORE32 => I64Store32(
					VarUint32::deserialize(reader)?.into(),
					VarUint32::deserialize(reader)?.into()),


				CURRENTMEMORY => {
					let mem_ref: u8 = Uint8::deserialize(reader)?.into();
					if mem_ref != 0 { return Err(Error::InvalidMemoryReference(mem_ref)); }
					CurrentMemory(mem_ref)
				},
				GROWMEMORY => {
					let mem_ref: u8 = Uint8::deserialize(reader)?.into();
					if mem_ref != 0 { return Err(Error::InvalidMemoryReference(mem_ref)); }
					GrowMemory(mem_ref)
				}

				I32CONST => I32Const(VarInt32::deserialize(reader)?.into()),
				I64CONST => I64Const(VarInt64::deserialize(reader)?.into()),
				F32CONST => F32Const(Uint32::deserialize(reader)?.into()),
				F64CONST => F64Const(Uint64::deserialize(reader)?.into()),
				I32EQZ => I32Eqz,
				I32EQ => I32Eq,
				I32NE => I32Ne,
				I32LTS => I32LtS,
				I32LTU => I32LtU,
				I32GTS => I32GtS,
				I32GTU => I32GtU,
				I32LES => I32LeS,
				I32LEU => I32LeU,
				I32GES => I32GeS,
				I32GEU => I32GeU,

				I64EQZ => I64Eqz,
				I64EQ => I64Eq,
				I64NE => I64Ne,
				I64LTS => I64LtS,
				I64LTU => I64LtU,
				I64GTS => I64GtS,
				I64GTU => I64GtU,
				I64LES => I64LeS,
				I64LEU => I64LeU,
				I64GES => I64GeS,
				I64GEU => I64GeU,

				F32EQ => F32Eq,
				F32NE => F32Ne,
				F32LT => F32Lt,
				F32GT => F32Gt,
				F32LE => F32Le,
				F32GE => F32Ge,

				F64EQ => F64Eq,
				F64NE => F64Ne,
				F64LT => F64Lt,
				F64GT => F64Gt,
				F64LE => F64Le,
				F64GE => F64Ge,

				I32CLZ => I32Clz,
				I32CTZ => I32Ctz,
				I32POPCNT => I32Popcnt,
				I32ADD => I32Add,
				I32SUB => I32Sub,
				I32MUL => I32Mul,
				I32DIVS => I32DivS,
				I32DIVU => I32DivU,
				I32REMS => I32RemS,
				I32REMU => I32RemU,
				I32AND => I32And,
				I32OR => I32Or,
				I32XOR => I32Xor,
				I32SHL => I32Shl,
				I32SHRS => I32ShrS,
				I32SHRU => I32ShrU,
				I32ROTL => I32Rotl,
				I32ROTR => I32Rotr,

				I64CLZ => I64Clz,
				I64CTZ => I64Ctz,
				I64POPCNT => I64Popcnt,
				I64ADD => I64Add,
				I64SUB => I64Sub,
				I64MUL => I64Mul,
				I64DIVS => I64DivS,
				I64DIVU => I64DivU,
				I64REMS => I64RemS,
				I64REMU => I64RemU,
				I64AND => I64And,
				I64OR => I64Or,
				I64XOR => I64Xor,
				I64SHL => I64Shl,
				I64SHRS => I64ShrS,
				I64SHRU => I64ShrU,
				I64ROTL => I64Rotl,
				I64ROTR => I64Rotr,
				F32ABS => F32Abs,
				F32NEG => F32Neg,
				F32CEIL => F32Ceil,
				F32FLOOR => F32Floor,
				F32TRUNC => F32Trunc,
				F32NEAREST => F32Nearest,
				F32SQRT => F32Sqrt,
				F32ADD => F32Add,
				F32SUB => F32Sub,
				F32MUL => F32Mul,
				F32DIV => F32Div,
				F32MIN => F32Min,
				F32MAX => F32Max,
				F32COPYSIGN => F32Copysign,
				F64ABS => F64Abs,
				F64NEG => F64Neg,
				F64CEIL => F64Ceil,
				F64FLOOR => F64Floor,
				F64TRUNC => F64Trunc,
				F64NEAREST => F64Nearest,
				F64SQRT => F64Sqrt,
				F64ADD => F64Add,
				F64SUB => F64Sub,
				F64MUL => F64Mul,
				F64DIV => F64Div,
				F64MIN => F64Min,
				F64MAX => F64Max,
				F64COPYSIGN => F64Copysign,

				I32WRAPI64 => I32WrapI64,
				I32TRUNCSF32 => I32TruncSF32,
				I32TRUNCUF32 => I32TruncUF32,
				I32TRUNCSF64 => I32TruncSF64,
				I32TRUNCUF64 => I32TruncUF64,
				I64EXTENDSI32 => I64ExtendSI32,
				I64EXTENDUI32 => I64ExtendUI32,
				I64TRUNCSF32 => I64TruncSF32,
				I64TRUNCUF32 => I64TruncUF32,
				I64TRUNCSF64 => I64TruncSF64,
				I64TRUNCUF64 => I64TruncUF64,
				F32CONVERTSI32 => F32ConvertSI32,
				F32CONVERTUI32 => F32ConvertUI32,
				F32CONVERTSI64 => F32ConvertSI64,
				F32CONVERTUI64 => F32ConvertUI64,
				F32DEMOTEF64 => F32DemoteF64,
				F64CONVERTSI32 => F64ConvertSI32,
				F64CONVERTUI32 => F64ConvertUI32,
				F64CONVERTSI64 => F64ConvertSI64,
				F64CONVERTUI64 => F64ConvertUI64,
				F64PROMOTEF32 => F64PromoteF32,

				I32REINTERPRETF32 => I32ReinterpretF32,
				I64REINTERPRETF64 => I64ReinterpretF64,
				F32REINTERPRETI32 => F32ReinterpretI32,
				F64REINTERPRETI64 => F64ReinterpretI64,

				_ => { return Err(Error::UnknownOpcode(val)); }
			}
		)
	}
}

macro_rules! op {
	($writer: expr, $byte: expr) => ({
		let b: u8 = $byte;
		$writer.write(&[b])?;
	});
	($writer: expr, $byte: expr, $s: block) => ({
		op!($writer, $byte);
		$s;
	});
}

impl Serialize for Instruction {
	type Error = Error;

	fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
		use self::Instruction::*;
		use self::opcodes::*;

		match self {
			Unreachable => op!(writer, UNREACHABLE),
			Nop => op!(writer, NOP),
			Block(block_type) => op!(writer, BLOCK, {
			   block_type.serialize(writer)?;
			}),
			Loop(block_type) => op!(writer, LOOP, {
			   block_type.serialize(writer)?;
			}),
			If(block_type) => op!(writer, IF, {
			   block_type.serialize(writer)?;
			}),
			Else => op!(writer, ELSE),
			End => op!(writer, END),
			Br(idx) => op!(writer, BR, {
				VarUint32::from(idx).serialize(writer)?;
			}),
			BrIf(idx) => op!(writer, BRIF, {
				VarUint32::from(idx).serialize(writer)?;
			}),
			BrTable(table, default) => op!(writer, BRTABLE, {
				let list_writer = CountedListWriter::<VarUint32, _>(
					table.len(),
					table.into_iter().map(|x| VarUint32::from(*x)),
				);
				list_writer.serialize(writer)?;
				VarUint32::from(default).serialize(writer)?;
			}),
			Return => op!(writer, RETURN),
			Call(index) => op!(writer, CALL, {
				VarUint32::from(index).serialize(writer)?;
			}),
			CallIndirect(index, reserved) => op!(writer, CALLINDIRECT, {
				VarUint32::from(index).serialize(writer)?;
				Uint8::from(reserved).serialize(writer)?;
			}),
			Drop => op!(writer, DROP),
			Select => op!(writer, SELECT),
			GetLocal(index) => op!(writer, GETLOCAL, {
				VarUint32::from(index).serialize(writer)?;
			}),
			SetLocal(index) => op!(writer, SETLOCAL, {
				VarUint32::from(index).serialize(writer)?;
			}),
			TeeLocal(index) => op!(writer, TEELOCAL, {
				VarUint32::from(index).serialize(writer)?;
			}),
			GetGlobal(index) => op!(writer, GETGLOBAL, {
				VarUint32::from(index).serialize(writer)?;
			}),
			SetGlobal(index) => op!(writer, SETGLOBAL, {
				VarUint32::from(index).serialize(writer)?;
			}),
			I32Load(flags, offset) => op!(writer, I32LOAD, {
				VarUint32::from(flags).serialize(writer)?;
				VarUint32::from(offset).serialize(writer)?;
			}),
			I64Load(flags, offset) => op!(writer, I64LOAD, {
				VarUint32::from(flags).serialize(writer)?;
				VarUint32::from(offset).serialize(writer)?;
			}),
			F32Load(flags, offset) => op!(writer, F32LOAD, {
				VarUint32::from(flags).serialize(writer)?;
				VarUint32::from(offset).serialize(writer)?;
			}),
			F64Load(flags, offset) => op!(writer, F64LOAD, {
				VarUint32::from(flags).serialize(writer)?;
				VarUint32::from(offset).serialize(writer)?;
			}),
			I32Load8S(flags, offset) => op!(writer, I32LOAD8S, {
				VarUint32::from(flags).serialize(writer)?;
				VarUint32::from(offset).serialize(writer)?;
			}),
			I32Load8U(flags, offset) => op!(writer, I32LOAD8U, {
				VarUint32::from(flags).serialize(writer)?;
				VarUint32::from(offset).serialize(writer)?;
			}),
			I32Load16S(flags, offset) => op!(writer, I32LOAD16S, {
				VarUint32::from(flags).serialize(writer)?;
				VarUint32::from(offset).serialize(writer)?;
			}),
			I32Load16U(flags, offset) => op!(writer, I32LOAD16U, {
				VarUint32::from(flags).serialize(writer)?;
				VarUint32::from(offset).serialize(writer)?;
			}),
			I64Load8S(flags, offset) => op!(writer, I64LOAD8S, {
				VarUint32::from(flags).serialize(writer)?;
				VarUint32::from(offset).serialize(writer)?;
			}),
			I64Load8U(flags, offset) => op!(writer, I64LOAD8U, {
				VarUint32::from(flags).serialize(writer)?;
				VarUint32::from(offset).serialize(writer)?;
			}),
			I64Load16S(flags, offset) => op!(writer, I64LOAD16S, {
				VarUint32::from(flags).serialize(writer)?;
				VarUint32::from(offset).serialize(writer)?;
			}),
			I64Load16U(flags, offset) => op!(writer, I64LOAD16U, {
				VarUint32::from(flags).serialize(writer)?;
				VarUint32::from(offset).serialize(writer)?;
			}),
			I64Load32S(flags, offset) => op!(writer, I64LOAD32S, {
				VarUint32::from(flags).serialize(writer)?;
				VarUint32::from(offset).serialize(writer)?;
			}),
			I64Load32U(flags, offset) => op!(writer, I64LOAD32U, {
				VarUint32::from(flags).serialize(writer)?;
				VarUint32::from(offset).serialize(writer)?;
			}),
			I32Store(flags, offset) => op!(writer, I32STORE, {
				VarUint32::from(flags).serialize(writer)?;
				VarUint32::from(offset).serialize(writer)?;
			}),
			I64Store(flags, offset) => op!(writer, I64STORE, {
				VarUint32::from(flags).serialize(writer)?;
				VarUint32::from(offset).serialize(writer)?;
			}),
			F32Store(flags, offset) => op!(writer, F32STORE, {
				VarUint32::from(flags).serialize(writer)?;
				VarUint32::from(offset).serialize(writer)?;
			}),
			F64Store(flags, offset) => op!(writer, F64STORE, {
				VarUint32::from(flags).serialize(writer)?;
				VarUint32::from(offset).serialize(writer)?;
			}),
			I32Store8(flags, offset) => op!(writer, I32STORE8, {
				VarUint32::from(flags).serialize(writer)?;
				VarUint32::from(offset).serialize(writer)?;
			}),
			I32Store16(flags, offset) => op!(writer, I32STORE16, {
				VarUint32::from(flags).serialize(writer)?;
				VarUint32::from(offset).serialize(writer)?;
			}),
			I64Store8(flags, offset) => op!(writer, I64STORE8, {
				VarUint32::from(flags).serialize(writer)?;
				VarUint32::from(offset).serialize(writer)?;
			}),
			I64Store16(flags, offset) => op!(writer, I64STORE16, {
				VarUint32::from(flags).serialize(writer)?;
				VarUint32::from(offset).serialize(writer)?;
			}),
			I64Store32(flags, offset) => op!(writer, I64STORE32, {
				VarUint32::from(flags).serialize(writer)?;
				VarUint32::from(offset).serialize(writer)?;
			}),
			CurrentMemory(flag) => op!(writer, CURRENTMEMORY, {
				Uint8::from(flag).serialize(writer)?;
			}),
			GrowMemory(flag) => op!(writer, GROWMEMORY, {
				Uint8::from(flag).serialize(writer)?;
			}),
			I32Const(def) => op!(writer, I32CONST, {
				VarInt32::from(def).serialize(writer)?;
			}),
			I64Const(def) => op!(writer, I64CONST, {
				VarInt64::from(def).serialize(writer)?;
			}),
			F32Const(def) => op!(writer, F32CONST, {
				Uint32::from(def).serialize(writer)?;
			}),
			F64Const(def) => op!(writer, F64CONST, {
				Uint64::from(def).serialize(writer)?;
			}),
			I32Eqz => op!(writer, I32EQZ),
			I32Eq => op!(writer, I32EQ),
			I32Ne => op!(writer, I32NE),
			I32LtS => op!(writer, I32LTS),
			I32LtU => op!(writer, I32LTU),
			I32GtS => op!(writer, I32GTS),
			I32GtU => op!(writer, I32GTU),
			I32LeS => op!(writer, I32LES),
			I32LeU => op!(writer, I32LEU),
			I32GeS => op!(writer, I32GES),
			I32GeU => op!(writer, I32GEU),

			I64Eqz => op!(writer, I64EQZ),
			I64Eq => op!(writer, I64EQ),
			I64Ne => op!(writer, I64NE),
			I64LtS => op!(writer, I64LTS),
			I64LtU => op!(writer, I64LTU),
			I64GtS => op!(writer, I64GTS),
			I64GtU => op!(writer, I64GTU),
			I64LeS => op!(writer, I64LES),
			I64LeU => op!(writer, I64LEU),
			I64GeS => op!(writer, I64GES),
			I64GeU => op!(writer, I64GEU),

			F32Eq => op!(writer, F32EQ),
			F32Ne => op!(writer, F32NE),
			F32Lt => op!(writer, F32LT),
			F32Gt => op!(writer, F32GT),
			F32Le => op!(writer, F32LE),
			F32Ge => op!(writer, F32GE),

			F64Eq => op!(writer, F64EQ),
			F64Ne => op!(writer, F64NE),
			F64Lt => op!(writer, F64LT),
			F64Gt => op!(writer, F64GT),
			F64Le => op!(writer, F64LE),
			F64Ge => op!(writer, F64GE),

			I32Clz => op!(writer, I32CLZ),
			I32Ctz => op!(writer, I32CTZ),
			I32Popcnt => op!(writer, I32POPCNT),
			I32Add => op!(writer, I32ADD),
			I32Sub => op!(writer, I32SUB),
			I32Mul => op!(writer, I32MUL),
			I32DivS => op!(writer, I32DIVS),
			I32DivU => op!(writer, I32DIVU),
			I32RemS => op!(writer, I32REMS),
			I32RemU => op!(writer, I32REMU),
			I32And => op!(writer, I32AND),
			I32Or => op!(writer, I32OR),
			I32Xor => op!(writer, I32XOR),
			I32Shl => op!(writer, I32SHL),
			I32ShrS => op!(writer, I32SHRS),
			I32ShrU => op!(writer, I32SHRU),
			I32Rotl => op!(writer, I32ROTL),
			I32Rotr => op!(writer, I32ROTR),

			I64Clz => op!(writer, I64CLZ),
			I64Ctz => op!(writer, I64CTZ),
			I64Popcnt => op!(writer, I64POPCNT),
			I64Add => op!(writer, I64ADD),
			I64Sub => op!(writer, I64SUB),
			I64Mul => op!(writer, I64MUL),
			I64DivS => op!(writer, I64DIVS),
			I64DivU => op!(writer, I64DIVU),
			I64RemS => op!(writer, I64REMS),
			I64RemU => op!(writer, I64REMU),
			I64And => op!(writer, I64AND),
			I64Or => op!(writer, I64OR),
			I64Xor => op!(writer, I64XOR),
			I64Shl => op!(writer, I64SHL),
			I64ShrS => op!(writer, I64SHRS),
			I64ShrU => op!(writer, I64SHRU),
			I64Rotl => op!(writer, I64ROTL),
			I64Rotr => op!(writer, I64ROTR),
			F32Abs => op!(writer, F32ABS),
			F32Neg => op!(writer, F32NEG),
			F32Ceil => op!(writer, F32CEIL),
			F32Floor => op!(writer, F32FLOOR),
			F32Trunc => op!(writer, F32TRUNC),
			F32Nearest => op!(writer, F32NEAREST),
			F32Sqrt => op!(writer, F32SQRT),
			F32Add => op!(writer, F32ADD),
			F32Sub => op!(writer, F32SUB),
			F32Mul => op!(writer, F32MUL),
			F32Div => op!(writer, F32DIV),
			F32Min => op!(writer, F32MIN),
			F32Max => op!(writer, F32MAX),
			F32Copysign => op!(writer, F32COPYSIGN),
			F64Abs => op!(writer, F64ABS),
			F64Neg => op!(writer, F64NEG),
			F64Ceil => op!(writer, F64CEIL),
			F64Floor => op!(writer, F64FLOOR),
			F64Trunc => op!(writer, F64TRUNC),
			F64Nearest => op!(writer, F64NEAREST),
			F64Sqrt => op!(writer, F64SQRT),
			F64Add => op!(writer, F64ADD),
			F64Sub => op!(writer, F64SUB),
			F64Mul => op!(writer, F64MUL),
			F64Div => op!(writer, F64DIV),
			F64Min => op!(writer, F64MIN),
			F64Max => op!(writer, F64MAX),
			F64Copysign => op!(writer, F64COPYSIGN),

			I32WrapI64 => op!(writer, I32WRAPI64),
			I32TruncSF32 => op!(writer, I32TRUNCSF32),
			I32TruncUF32 => op!(writer, I32TRUNCUF32),
			I32TruncSF64 => op!(writer, I32TRUNCSF64),
			I32TruncUF64 => op!(writer, I32TRUNCUF64),
			I64ExtendSI32 => op!(writer, I64EXTENDSI32),
			I64ExtendUI32 => op!(writer, I64EXTENDUI32),
			I64TruncSF32 => op!(writer, I64TRUNCSF32),
			I64TruncUF32 => op!(writer, I64TRUNCUF32),
			I64TruncSF64 => op!(writer, I64TRUNCSF64),
			I64TruncUF64 => op!(writer, I64TRUNCUF64),
			F32ConvertSI32 => op!(writer, F32CONVERTSI32),
			F32ConvertUI32 => op!(writer, F32CONVERTUI32),
			F32ConvertSI64 => op!(writer, F32CONVERTSI64),
			F32ConvertUI64 => op!(writer, F32CONVERTUI64),
			F32DemoteF64 => op!(writer, F32DEMOTEF64),
			F64ConvertSI32 => op!(writer, F64CONVERTSI32),
			F64ConvertUI32 => op!(writer, F64CONVERTUI32),
			F64ConvertSI64 => op!(writer, F64CONVERTSI64),
			F64ConvertUI64 => op!(writer, F64CONVERTUI64),
			F64PromoteF32 => op!(writer, F64PROMOTEF32),

			I32ReinterpretF32 => op!(writer, I32REINTERPRETF32),
			I64ReinterpretF64 => op!(writer, I64REINTERPRETF64),
			F32ReinterpretI32 => op!(writer, F32REINTERPRETI32),
			F64ReinterpretI64 => op!(writer, F64REINTERPRETI64),
		}

		Ok(())
	}
}

macro_rules! fmt_op {
	($f: expr, $mnemonic: expr) => ({
		write!($f, "{}", $mnemonic)
	});
	($f: expr, $mnemonic: expr, $immediate: expr) => ({
		write!($f, "{} {}", $mnemonic, $immediate)
	});
	($f: expr, $mnemonic: expr, $immediate1: expr, $immediate2: expr) => ({
		write!($f, "{} {} {}", $mnemonic, $immediate1, $immediate2)
	});
}

impl fmt::Display for Instruction {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		use self::Instruction::*;
		use super::BlockType;

		match *self {
			Unreachable => fmt_op!(f, "unreachable"),
			Nop => fmt_op!(f, "nop"),
			Block(BlockType::NoResult) => fmt_op!(f, "block"),
			Block(BlockType::Value(value_type)) => fmt_op!(f, "block", value_type),
			Loop(BlockType::NoResult) => fmt_op!(f, "loop"),
			Loop(BlockType::Value(value_type)) => fmt_op!(f, "loop", value_type),
			If(BlockType::NoResult) => fmt_op!(f, "if"),
			If(BlockType::Value(value_type)) => fmt_op!(f, "if", value_type),
			Else => fmt_op!(f, "else"),
			End => fmt_op!(f, "end"),
			Br(idx) => fmt_op!(f, "br",  idx),
			BrIf(idx) => fmt_op!(f, "br_if",  idx),
			BrTable(_, default) => fmt_op!(f, "br_table", default),
			Return => fmt_op!(f, "return"),
			Call(index) => fmt_op!(f, "call", index),
			CallIndirect(index, _) =>  fmt_op!(f, "call_indirect", index),
			Drop => fmt_op!(f, "drop"),
			Select => fmt_op!(f, "select"),
			GetLocal(index) => fmt_op!(f, "get_local", index),
			SetLocal(index) => fmt_op!(f, "set_local", index),
			TeeLocal(index) => fmt_op!(f, "tee_local", index),
			GetGlobal(index) => fmt_op!(f, "get_global", index),
			SetGlobal(index) => fmt_op!(f, "set_global", index),

			I32Load(_, 0) => write!(f, "i32.load"),
			I32Load(_, offset) => write!(f, "i32.load offset={}", offset),

			I64Load(_, 0) => write!(f, "i64.load"),
			I64Load(_, offset) => write!(f, "i64.load offset={}", offset),

			F32Load(_, 0) => write!(f, "f32.load"),
			F32Load(_, offset) => write!(f, "f32.load offset={}", offset),

			F64Load(_, 0) => write!(f, "f64.load"),
			F64Load(_, offset) => write!(f, "f64.load offset={}", offset),

			I32Load8S(_, 0) => write!(f, "i32.load8_s"),
			I32Load8S(_, offset) => write!(f, "i32.load8_s offset={}", offset),

			I32Load8U(_, 0) => write!(f, "i32.load8_u"),
			I32Load8U(_, offset) => write!(f, "i32.load8_u offset={}", offset),

			I32Load16S(_, 0) => write!(f, "i32.load16_s"),
			I32Load16S(_, offset) => write!(f, "i32.load16_s offset={}", offset),

			I32Load16U(_, 0) => write!(f, "i32.load16_u"),
			I32Load16U(_, offset) => write!(f, "i32.load16_u offset={}", offset),

			I64Load8S(_, 0) => write!(f, "i64.load8_s"),
			I64Load8S(_, offset) => write!(f, "i64.load8_s offset={}", offset),

			I64Load8U(_, 0) => write!(f, "i64.load8_u"),
			I64Load8U(_, offset) => write!(f, "i64.load8_u offset={}", offset),

			I64Load16S(_, 0) => write!(f, "i64.load16_s"),
			I64Load16S(_, offset) => write!(f, "i64.load16_s offset={}", offset),

			I64Load16U(_, 0) => write!(f, "i64.load16_u"),
			I64Load16U(_, offset) => write!(f, "i64.load16_u offset={}", offset),

			I64Load32S(_, 0) => write!(f, "i64.load32_s"),
			I64Load32S(_, offset) => write!(f, "i64.load32_s offset={}", offset),

			I64Load32U(_, 0) => write!(f, "i64.load32_u"),
			I64Load32U(_, offset) => write!(f, "i64.load32_u offset={}", offset),

			I32Store(_, 0) => write!(f, "i32.store"),
			I32Store(_, offset) => write!(f, "i32.store offset={}", offset),

			I64Store(_, 0) => write!(f, "i64.store"),
			I64Store(_, offset) => write!(f, "i64.store offset={}", offset),

			F32Store(_, 0) => write!(f, "f32.store"),
			F32Store(_, offset) => write!(f, "f32.store offset={}", offset),

			F64Store(_, 0) => write!(f, "f64.store"),
			F64Store(_, offset) => write!(f, "f64.store offset={}", offset),

			I32Store8(_, 0) => write!(f, "i32.store8"),
			I32Store8(_, offset) => write!(f, "i32.store8 offset={}", offset),

			I32Store16(_, 0) => write!(f, "i32.store16"),
			I32Store16(_, offset) => write!(f, "i32.store16 offset={}", offset),

			I64Store8(_, 0) => write!(f, "i64.store8"),
			I64Store8(_, offset) => write!(f, "i64.store8 offset={}", offset),

			I64Store16(_, 0) => write!(f, "i64.store16"),
			I64Store16(_, offset) => write!(f, "i64.store16 offset={}", offset),

			I64Store32(_, 0) => write!(f, "i64.store32"),
			I64Store32(_, offset) => write!(f, "i64.store32 offset={}", offset),

			CurrentMemory(_) => fmt_op!(f, "current_memory"),
			GrowMemory(_) => fmt_op!(f, "grow_memory"),

			I32Const(def) => fmt_op!(f, "i32.const", def),
			I64Const(def) => fmt_op!(f, "i64.const", def),
			F32Const(def) => fmt_op!(f, "f32.const", def),
			F64Const(def) => fmt_op!(f, "f64.const", def),

			I32Eq => write!(f, "i32.eq"),
			I32Eqz => write!(f, "i32.eqz"),
			I32Ne => write!(f, "i32.ne"),
			I32LtS => write!(f, "i32.lt_s"),
			I32LtU => write!(f, "i32.lt_u"),
			I32GtS => write!(f, "i32.gt_s"),
			I32GtU => write!(f, "i32.gt_u"),
			I32LeS => write!(f, "i32.le_s"),
			I32LeU => write!(f, "i32.le_u"),
			I32GeS => write!(f, "i32.ge_s"),
			I32GeU => write!(f, "i32.ge_u"),

			I64Eq => write!(f, "i64.eq"),
			I64Eqz => write!(f, "i64.eqz"),
			I64Ne => write!(f, "i64.ne"),
			I64LtS => write!(f, "i64.lt_s"),
			I64LtU => write!(f, "i64.lt_u"),
			I64GtS => write!(f, "i64.gt_s"),
			I64GtU => write!(f, "i64.gt_u"),
			I64LeS => write!(f, "i64.le_s"),
			I64LeU => write!(f, "i64.le_u"),
			I64GeS => write!(f, "i64.ge_s"),
			I64GeU => write!(f, "i64.ge_u"),

			F32Eq => write!(f, "f32.eq"),
			F32Ne => write!(f, "f32.ne"),
			F32Lt => write!(f, "f32.lt"),
			F32Gt => write!(f, "f32.gt"),
			F32Le => write!(f, "f32.le"),
			F32Ge => write!(f, "f32.ge"),

			F64Eq => write!(f, "f64.eq"),
			F64Ne => write!(f, "f64.ne"),
			F64Lt => write!(f, "f64.lt"),
			F64Gt => write!(f, "f64.gt"),
			F64Le => write!(f, "f64.le"),
			F64Ge => write!(f, "f64.ge"),

			I32Clz => write!(f, "i32.clz"),
			I32Ctz => write!(f, "i32.ctz"),
			I32Popcnt => write!(f, "i32.popcnt"),
			I32Add => write!(f, "i32.add"),
			I32Sub => write!(f, "i32.sub"),
			I32Mul => write!(f, "i32.mul"),
			I32DivS => write!(f, "i32.div_s"),
			I32DivU => write!(f, "i32.div_u"),
			I32RemS => write!(f, "i32.rem_s"),
			I32RemU => write!(f, "i32.rem_u"),
			I32And => write!(f, "i32.and"),
			I32Or => write!(f, "i32.or"),
			I32Xor => write!(f, "i32.xor"),
			I32Shl => write!(f, "i32.shl"),
			I32ShrS => write!(f, "i32.shr_s"),
			I32ShrU => write!(f, "i32.shr_u"),
			I32Rotl => write!(f, "i32.rotl"),
			I32Rotr => write!(f, "i32.rotr"),

			I64Clz => write!(f, "i64.clz"),
			I64Ctz => write!(f, "i64.ctz"),
			I64Popcnt => write!(f, "i64.popcnt"),
			I64Add => write!(f, "i64.add"),
			I64Sub => write!(f, "i64.sub"),
			I64Mul => write!(f, "i64.mul"),
			I64DivS => write!(f, "i64.div_s"),
			I64DivU => write!(f, "i64.div_u"),
			I64RemS => write!(f, "i64.rem_s"),
			I64RemU => write!(f, "i64.rem_u"),
			I64And => write!(f, "i64.and"),
			I64Or => write!(f, "i64.or"),
			I64Xor => write!(f, "i64.xor"),
			I64Shl => write!(f, "i64.shl"),
			I64ShrS => write!(f, "i64.shr_s"),
			I64ShrU => write!(f, "i64.shr_u"),
			I64Rotl => write!(f, "i64.rotl"),
			I64Rotr => write!(f, "i64.rotr"),

			F32Abs => write!(f, "f32.abs"),
			F32Neg => write!(f, "f32.neg"),
			F32Ceil => write!(f, "f32.ceil"),
			F32Floor => write!(f, "f32.floor"),
			F32Trunc => write!(f, "f32.trunc"),
			F32Nearest => write!(f, "f32.nearest"),
			F32Sqrt => write!(f, "f32.sqrt"),
			F32Add => write!(f, "f32.add"),
			F32Sub => write!(f, "f32.sub"),
			F32Mul => write!(f, "f32.mul"),
			F32Div => write!(f, "f32.div"),
			F32Min => write!(f, "f32.min"),
			F32Max => write!(f, "f32.max"),
			F32Copysign => write!(f, "f32.copysign"),

			F64Abs => write!(f, "f64.abs"),
			F64Neg => write!(f, "f64.neg"),
			F64Ceil => write!(f, "f64.ceil"),
			F64Floor => write!(f, "f64.floor"),
			F64Trunc => write!(f, "f64.trunc"),
			F64Nearest => write!(f, "f64.nearest"),
			F64Sqrt => write!(f, "f64.sqrt"),
			F64Add => write!(f, "f64.add"),
			F64Sub => write!(f, "f64.sub"),
			F64Mul => write!(f, "f64.mul"),
			F64Div => write!(f, "f64.div"),
			F64Min => write!(f, "f64.min"),
			F64Max => write!(f, "f64.max"),
			F64Copysign => write!(f, "f64.copysign"),

			I32WrapI64 => write!(f, "i32.wrap/i64"),
			I32TruncSF32 => write!(f, "i32.trunc_s/f32"),
			I32TruncUF32 => write!(f, "i32.trunc_u/f32"),
			I32TruncSF64 => write!(f, "i32.trunc_s/f64"),
			I32TruncUF64 => write!(f, "i32.trunc_u/f64"),

			I64ExtendSI32 => write!(f, "i64.extend_s/i32"),
			I64ExtendUI32 => write!(f, "i64.extend_u/i32"),

			I64TruncSF32 => write!(f, "i64.trunc_s/f32"),
			I64TruncUF32 => write!(f, "i64.trunc_u/f32"),
			I64TruncSF64 => write!(f, "i64.trunc_s/f64"),
			I64TruncUF64 => write!(f, "i64.trunc_u/f64"),

			F32ConvertSI32 => write!(f, "f32.convert_s/i32"),
			F32ConvertUI32 => write!(f, "f32.convert_u/i32"),
			F32ConvertSI64 => write!(f, "f32.convert_s/i64"),
			F32ConvertUI64 => write!(f, "f32.convert_u/i64"),
			F32DemoteF64 => write!(f, "f32.demote/f64"),

			F64ConvertSI32 => write!(f, "f64.convert_s/i32"),
			F64ConvertUI32 => write!(f, "f64.convert_u/i32"),
			F64ConvertSI64 => write!(f, "f64.convert_s/i64"),
			F64ConvertUI64 => write!(f, "f64.convert_u/i64"),
			F64PromoteF32 => write!(f, "f64.promote/f32"),

			I32ReinterpretF32 => write!(f, "i32.reinterpret/f32"),
			I64ReinterpretF64 => write!(f, "i64.reinterpret/f64"),
			F32ReinterpretI32 => write!(f, "f32.reinterpret/i32"),
			F64ReinterpretI64 => write!(f, "f64.reinterpret/i64"),
		}
	}
}

impl Serialize for Instructions {
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

#[test]
fn ifelse() {
	// see if-else.wast/if-else.wasm
	let instruction_list = super::deserialize_buffer::<Instructions>(&[0x04, 0x7F, 0x41, 0x05, 0x05, 0x41, 0x07, 0x0B, 0x0B])
		.expect("valid hex of if instruction");
	let instructions = instruction_list.elements();
	match &instructions[0] {
		&Instruction::If(_) => (),
		_ => panic!("Should be deserialized as if instruction"),
	}
	let before_else = instructions.iter().skip(1)
		.take_while(|op| match **op { Instruction::Else => false, _ => true }).count();
	let after_else = instructions.iter().skip(1)
		.skip_while(|op| match **op { Instruction::Else => false, _ => true })
		.take_while(|op| match **op { Instruction::End => false, _ => true })
		.count()
		- 1; // minus Instruction::Else itself
	assert_eq!(before_else, after_else);
}

#[test]
fn display() {
	let instruction = Instruction::GetLocal(0);
	assert_eq!("get_local 0", format!("{}", instruction));

	let instruction = Instruction::F64Store(0, 24);
	assert_eq!("f64.store offset=24", format!("{}", instruction));

	let instruction = Instruction::I64Store(0, 0);
	assert_eq!("i64.store", format!("{}", instruction));
}

#[test]
fn size_off() {
	assert!(::std::mem::size_of::<Instruction>() <= 24);
}
