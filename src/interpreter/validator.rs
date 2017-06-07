use elements::{Module, Opcode, BlockType, FunctionType, ValueType, External, Type};
use interpreter::Error;
use interpreter::runner::{DEFAULT_MEMORY_INDEX, DEFAULT_TABLE_INDEX};
use interpreter::imports::ModuleImports;
use interpreter::module::ItemIndex;
use interpreter::stack::StackWithLimit;
use interpreter::variable::VariableType;

/// Function validation context.
pub struct FunctionValidationContext<'a> {
	/// Wasm module.
	module: &'a Module,
	/// Module imports.
	imports: &'a ModuleImports,
	/// Local variables.
	locals: &'a [ValueType],
	/// Value stack.
	value_stack: StackWithLimit<ValueType>,
	/// Frame stack.
	frame_stack: StackWithLimit<(BlockType, usize)>,
	/// Function return type. None if validating expression.
	return_type: Option<BlockType>,
}

/// Function validator.
pub struct Validator;

/// Instruction outcome.
#[derive(Debug, Clone)]
pub enum InstructionOutcome {
	/// Continue with next instruction.
	RunNextInstruction,
	/// Unreachable instruction reached.
	Unreachable,
}

impl Validator {
	pub fn validate_block(context: &mut FunctionValidationContext, block_type: BlockType, body: &[Opcode], end_instr: Opcode) -> Result<InstructionOutcome, Error> {
		if body.is_empty() || body[body.len() - 1] != end_instr {
			return Err(Error::Validation("Every block must end with end/else instruction".into()));
		}

		context.push_label(block_type)?;
		for opcode in body {
			match Validator::validate_instruction(context, opcode)? {
				InstructionOutcome::RunNextInstruction => (),
				InstructionOutcome::Unreachable => return Ok(InstructionOutcome::Unreachable),
			}
		}
		let return_type = context.pop_label()?;

		match block_type {
			BlockType::NoResult => Ok(InstructionOutcome::RunNextInstruction),
			BlockType::Value(value_type) if Some(value_type) != return_type =>
				Err(Error::Validation(format!("Expected block to return {:?} while it has returned {:?}", value_type, return_type))),
			BlockType::Value(value_type) => context.push_value(value_type).map(|_| InstructionOutcome::RunNextInstruction),
		}
	}

	pub fn validate_instruction(context: &mut FunctionValidationContext, opcode: &Opcode) -> Result<InstructionOutcome, Error> {
		match opcode {
			&Opcode::Unreachable => Ok(InstructionOutcome::Unreachable),
			&Opcode::Nop => Ok(InstructionOutcome::RunNextInstruction),
			&Opcode::Block(block_type, ref ops) => Validator::validate_block(context, block_type, ops.elements(), Opcode::End),
			&Opcode::Loop(block_type, ref ops) => Validator::validate_loop(context, block_type, ops.elements()),
			&Opcode::If(block_type, ref ops) => Validator::validate_if(context, block_type, ops.elements()),
			&Opcode::Else => Ok(InstructionOutcome::RunNextInstruction),
			&Opcode::End => Ok(InstructionOutcome::RunNextInstruction),
			&Opcode::Br(idx) => Validator::validate_br(context, idx),
			&Opcode::BrIf(idx) => Validator::validate_br_if(context, idx),
			&Opcode::BrTable(ref table, default) => Validator::validate_br_table(context, table, default),
			&Opcode::Return => Validator::validate_return(context),

			&Opcode::Call(index) => Validator::validate_call(context, index),
			&Opcode::CallIndirect(index, _reserved) => Validator::validate_call_indirect(context, index),

			&Opcode::Drop => Validator::validate_drop(context),
			&Opcode::Select => Validator::validate_select(context),

			&Opcode::GetLocal(index) => Validator::validate_get_local(context, index),
			&Opcode::SetLocal(index) => Validator::validate_set_local(context, index),
			&Opcode::TeeLocal(index) => Validator::validate_tee_local(context, index),
			&Opcode::GetGlobal(index) => Validator::validate_get_global(context, index),
			&Opcode::SetGlobal(index) => Validator::validate_set_global(context, index),

			&Opcode::I32Load(align, _) => Validator::validate_load(context, align, 4, ValueType::I32),
			&Opcode::I64Load(align, _) => Validator::validate_load(context, align, 8, ValueType::I64),
			&Opcode::F32Load(align, _) => Validator::validate_load(context, align, 4, ValueType::F32),
			&Opcode::F64Load(align, _) => Validator::validate_load(context, align, 8, ValueType::F64),
			&Opcode::I32Load8S(align, _) => Validator::validate_load(context, align, 1, ValueType::I32),
			&Opcode::I32Load8U(align, _) => Validator::validate_load(context, align, 1, ValueType::I32),
			&Opcode::I32Load16S(align, _) => Validator::validate_load(context, align, 2, ValueType::I32),
			&Opcode::I32Load16U(align, _) => Validator::validate_load(context, align, 2, ValueType::I32),
			&Opcode::I64Load8S(align, _) => Validator::validate_load(context, align, 1, ValueType::I64),
			&Opcode::I64Load8U(align, _) => Validator::validate_load(context, align, 1, ValueType::I64),
			&Opcode::I64Load16S(align, _) => Validator::validate_load(context, align, 2, ValueType::I64),
			&Opcode::I64Load16U(align, _) => Validator::validate_load(context, align, 2, ValueType::I64),
			&Opcode::I64Load32S(align, _) => Validator::validate_load(context, align, 4, ValueType::I64),
			&Opcode::I64Load32U(align, _) => Validator::validate_load(context, align, 4, ValueType::I64),

			&Opcode::I32Store(align, _) => Validator::validate_store(context, align, 4, ValueType::I32),
			&Opcode::I64Store(align, _) => Validator::validate_store(context, align, 8, ValueType::I64),
			&Opcode::F32Store(align, _) => Validator::validate_store(context, align, 4, ValueType::F32),
			&Opcode::F64Store(align, _) => Validator::validate_store(context, align, 8, ValueType::F64),
			&Opcode::I32Store8(align, _) => Validator::validate_store(context, align, 1, ValueType::I32),
			&Opcode::I32Store16(align, _) => Validator::validate_store(context, align, 2, ValueType::I32),
			&Opcode::I64Store8(align, _) => Validator::validate_store(context, align, 1, ValueType::I64),
			&Opcode::I64Store16(align, _) => Validator::validate_store(context, align, 2, ValueType::I64),
			&Opcode::I64Store32(align, _) => Validator::validate_store(context, align, 4, ValueType::I64),

			&Opcode::CurrentMemory(_) => Validator::validate_current_memory(context),
			&Opcode::GrowMemory(_) => Validator::validate_grow_memory(context),

			&Opcode::I32Const(_) => Validator::validate_const(context, ValueType::I32),
			&Opcode::I64Const(_) => Validator::validate_const(context, ValueType::I64),
			&Opcode::F32Const(_) => Validator::validate_const(context, ValueType::F32),
			&Opcode::F64Const(_) => Validator::validate_const(context, ValueType::F64),

			&Opcode::I32Eqz => Validator::validate_testop(context, ValueType::I32),
			&Opcode::I32Eq => Validator::validate_relop(context, ValueType::I32),
			&Opcode::I32Ne => Validator::validate_relop(context, ValueType::I32),
			&Opcode::I32LtS => Validator::validate_relop(context, ValueType::I32),
			&Opcode::I32LtU => Validator::validate_relop(context, ValueType::I32),
			&Opcode::I32GtS => Validator::validate_relop(context, ValueType::I32),
			&Opcode::I32GtU => Validator::validate_relop(context, ValueType::I32),
			&Opcode::I32LeS => Validator::validate_relop(context, ValueType::I32),
			&Opcode::I32LeU => Validator::validate_relop(context, ValueType::I32),
			&Opcode::I32GeS => Validator::validate_relop(context, ValueType::I32),
			&Opcode::I32GeU => Validator::validate_relop(context, ValueType::I32),

			&Opcode::I64Eqz => Validator::validate_testop(context, ValueType::I64),
			&Opcode::I64Eq => Validator::validate_relop(context, ValueType::I64),
			&Opcode::I64Ne => Validator::validate_relop(context, ValueType::I64),
			&Opcode::I64LtS => Validator::validate_relop(context, ValueType::I64),
			&Opcode::I64LtU => Validator::validate_relop(context, ValueType::I64),
			&Opcode::I64GtS => Validator::validate_relop(context, ValueType::I64),
			&Opcode::I64GtU => Validator::validate_relop(context, ValueType::I64),
			&Opcode::I64LeS => Validator::validate_relop(context, ValueType::I64),
			&Opcode::I64LeU => Validator::validate_relop(context, ValueType::I64),
			&Opcode::I64GeS => Validator::validate_relop(context, ValueType::I64),
			&Opcode::I64GeU => Validator::validate_relop(context, ValueType::I64),

			&Opcode::F32Eq => Validator::validate_relop(context, ValueType::F32),
			&Opcode::F32Ne => Validator::validate_relop(context, ValueType::F32),
			&Opcode::F32Lt => Validator::validate_relop(context, ValueType::F32),
			&Opcode::F32Gt => Validator::validate_relop(context, ValueType::F32),
			&Opcode::F32Le => Validator::validate_relop(context, ValueType::F32),
			&Opcode::F32Ge => Validator::validate_relop(context, ValueType::F32),

			&Opcode::F64Eq => Validator::validate_relop(context, ValueType::F64),
			&Opcode::F64Ne => Validator::validate_relop(context, ValueType::F64),
			&Opcode::F64Lt => Validator::validate_relop(context, ValueType::F64),
			&Opcode::F64Gt => Validator::validate_relop(context, ValueType::F64),
			&Opcode::F64Le => Validator::validate_relop(context, ValueType::F64),
			&Opcode::F64Ge => Validator::validate_relop(context, ValueType::F64),

			&Opcode::I32Clz => Validator::validate_unop(context, ValueType::I32),
			&Opcode::I32Ctz => Validator::validate_unop(context, ValueType::I32),
			&Opcode::I32Popcnt => Validator::validate_unop(context, ValueType::I32),
			&Opcode::I32Add => Validator::validate_binop(context, ValueType::I32),
			&Opcode::I32Sub => Validator::validate_binop(context, ValueType::I32),
			&Opcode::I32Mul => Validator::validate_binop(context, ValueType::I32),
			&Opcode::I32DivS => Validator::validate_binop(context, ValueType::I32),
			&Opcode::I32DivU => Validator::validate_binop(context, ValueType::I32),
			&Opcode::I32RemS => Validator::validate_binop(context, ValueType::I32),
			&Opcode::I32RemU => Validator::validate_binop(context, ValueType::I32),
			&Opcode::I32And => Validator::validate_binop(context, ValueType::I32),
			&Opcode::I32Or => Validator::validate_binop(context, ValueType::I32),
			&Opcode::I32Xor => Validator::validate_binop(context, ValueType::I32),
			&Opcode::I32Shl => Validator::validate_binop(context, ValueType::I32),
			&Opcode::I32ShrS => Validator::validate_binop(context, ValueType::I32),
			&Opcode::I32ShrU => Validator::validate_binop(context, ValueType::I32),
			&Opcode::I32Rotl => Validator::validate_binop(context, ValueType::I32),
			&Opcode::I32Rotr => Validator::validate_binop(context, ValueType::I32),

			&Opcode::I64Clz => Validator::validate_unop(context, ValueType::I64),
			&Opcode::I64Ctz => Validator::validate_unop(context, ValueType::I64),
			&Opcode::I64Popcnt => Validator::validate_unop(context, ValueType::I64),
			&Opcode::I64Add => Validator::validate_binop(context, ValueType::I64),
			&Opcode::I64Sub => Validator::validate_binop(context, ValueType::I64),
			&Opcode::I64Mul => Validator::validate_binop(context, ValueType::I64),
			&Opcode::I64DivS => Validator::validate_binop(context, ValueType::I64),
			&Opcode::I64DivU => Validator::validate_binop(context, ValueType::I64),
			&Opcode::I64RemS => Validator::validate_binop(context, ValueType::I64),
			&Opcode::I64RemU => Validator::validate_binop(context, ValueType::I64),
			&Opcode::I64And => Validator::validate_binop(context, ValueType::I64),
			&Opcode::I64Or => Validator::validate_binop(context, ValueType::I64),
			&Opcode::I64Xor => Validator::validate_binop(context, ValueType::I64),
			&Opcode::I64Shl => Validator::validate_binop(context, ValueType::I64),
			&Opcode::I64ShrS => Validator::validate_binop(context, ValueType::I64),
			&Opcode::I64ShrU => Validator::validate_binop(context, ValueType::I64),
			&Opcode::I64Rotl => Validator::validate_binop(context, ValueType::I64),
			&Opcode::I64Rotr => Validator::validate_binop(context, ValueType::I64),

			&Opcode::F32Abs => Validator::validate_unop(context, ValueType::F32),
			&Opcode::F32Neg => Validator::validate_unop(context, ValueType::F32),
			&Opcode::F32Ceil => Validator::validate_unop(context, ValueType::F32),
			&Opcode::F32Floor => Validator::validate_unop(context, ValueType::F32),
			&Opcode::F32Trunc => Validator::validate_unop(context, ValueType::F32),
			&Opcode::F32Nearest => Validator::validate_unop(context, ValueType::F32),
			&Opcode::F32Sqrt => Validator::validate_unop(context, ValueType::F32),
			&Opcode::F32Add => Validator::validate_binop(context, ValueType::F32),
			&Opcode::F32Sub => Validator::validate_binop(context, ValueType::F32),
			&Opcode::F32Mul => Validator::validate_binop(context, ValueType::F32),
			&Opcode::F32Div => Validator::validate_binop(context, ValueType::F32),
			&Opcode::F32Min => Validator::validate_binop(context, ValueType::F32),
			&Opcode::F32Max => Validator::validate_binop(context, ValueType::F32),
			&Opcode::F32Copysign => Validator::validate_binop(context, ValueType::F32),

			&Opcode::F64Abs => Validator::validate_unop(context, ValueType::F64),
			&Opcode::F64Neg => Validator::validate_unop(context, ValueType::F64),
			&Opcode::F64Ceil => Validator::validate_unop(context, ValueType::F64),
			&Opcode::F64Floor => Validator::validate_unop(context, ValueType::F64),
			&Opcode::F64Trunc => Validator::validate_unop(context, ValueType::F64),
			&Opcode::F64Nearest => Validator::validate_unop(context, ValueType::F64),
			&Opcode::F64Sqrt => Validator::validate_unop(context, ValueType::F64),
			&Opcode::F64Add => Validator::validate_binop(context, ValueType::F64),
			&Opcode::F64Sub => Validator::validate_binop(context, ValueType::F64),
			&Opcode::F64Mul => Validator::validate_binop(context, ValueType::F64),
			&Opcode::F64Div => Validator::validate_binop(context, ValueType::F64),
			&Opcode::F64Min => Validator::validate_binop(context, ValueType::F64),
			&Opcode::F64Max => Validator::validate_binop(context, ValueType::F64),
			&Opcode::F64Copysign => Validator::validate_binop(context, ValueType::F64),

			&Opcode::I32WarpI64 => Validator::validate_cvtop(context, ValueType::I64, ValueType::I32),
			&Opcode::I32TruncSF32 => Validator::validate_cvtop(context, ValueType::F32, ValueType::I32),
			&Opcode::I32TruncUF32 => Validator::validate_cvtop(context, ValueType::F32, ValueType::I32),
			&Opcode::I32TruncSF64 => Validator::validate_cvtop(context, ValueType::F64, ValueType::I32),
			&Opcode::I32TruncUF64 => Validator::validate_cvtop(context, ValueType::F64, ValueType::I32),
			&Opcode::I64ExtendSI32 => Validator::validate_cvtop(context, ValueType::I32, ValueType::I64),
			&Opcode::I64ExtendUI32 => Validator::validate_cvtop(context, ValueType::I32, ValueType::I64),
			&Opcode::I64TruncSF32 => Validator::validate_cvtop(context, ValueType::F32, ValueType::I64),
			&Opcode::I64TruncUF32 => Validator::validate_cvtop(context, ValueType::F32, ValueType::I64),
			&Opcode::I64TruncSF64 => Validator::validate_cvtop(context, ValueType::F64, ValueType::I64),
			&Opcode::I64TruncUF64 => Validator::validate_cvtop(context, ValueType::F64, ValueType::I64),
			&Opcode::F32ConvertSI32 => Validator::validate_cvtop(context, ValueType::I32, ValueType::F32),
			&Opcode::F32ConvertUI32 => Validator::validate_cvtop(context, ValueType::I32, ValueType::F32),
			&Opcode::F32ConvertSI64 => Validator::validate_cvtop(context, ValueType::I64, ValueType::F32),
			&Opcode::F32ConvertUI64 => Validator::validate_cvtop(context, ValueType::I64, ValueType::F32),
			&Opcode::F32DemoteF64 => Validator::validate_cvtop(context, ValueType::F64, ValueType::F32),
			&Opcode::F64ConvertSI32 => Validator::validate_cvtop(context, ValueType::I32, ValueType::F64),
			&Opcode::F64ConvertUI32 => Validator::validate_cvtop(context, ValueType::I32, ValueType::F64),
			&Opcode::F64ConvertSI64 => Validator::validate_cvtop(context, ValueType::I64, ValueType::F64),
			&Opcode::F64ConvertUI64 => Validator::validate_cvtop(context, ValueType::I64, ValueType::F64),
			&Opcode::F64PromoteF32 => Validator::validate_cvtop(context, ValueType::F32, ValueType::F64),

			&Opcode::I32ReinterpretF32 => Validator::validate_cvtop(context, ValueType::F32, ValueType::I32),
			&Opcode::I64ReinterpretF64 => Validator::validate_cvtop(context, ValueType::F64, ValueType::I64),
			&Opcode::F32ReinterpretI32 => Validator::validate_cvtop(context, ValueType::I32, ValueType::F32),
			&Opcode::F64ReinterpretI64 => Validator::validate_cvtop(context, ValueType::I64, ValueType::F64),
		}
	}

	fn validate_const(context: &mut FunctionValidationContext, value_type: ValueType) -> Result<InstructionOutcome, Error> {
		context.push_value(value_type)?;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn validate_unop(context: &mut FunctionValidationContext, value_type: ValueType) -> Result<InstructionOutcome, Error> {
		context.pop_value(value_type)?;
		context.push_value(value_type)?;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn validate_binop(context: &mut FunctionValidationContext, value_type: ValueType) -> Result<InstructionOutcome, Error> {
		context.pop_value(value_type)?;
		context.pop_value(value_type)?;
		context.push_value(value_type)?;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn validate_testop(context: &mut FunctionValidationContext, value_type: ValueType) -> Result<InstructionOutcome, Error> {
		context.pop_value(value_type)?;
		context.push_value(ValueType::I32)?;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn validate_relop(context: &mut FunctionValidationContext, value_type: ValueType) -> Result<InstructionOutcome, Error> {
		context.pop_value(value_type)?;
		context.pop_value(value_type)?;
		context.push_value(ValueType::I32)?;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn validate_cvtop(context: &mut FunctionValidationContext, value_type1: ValueType, value_type2: ValueType) -> Result<InstructionOutcome, Error> {
		context.pop_value(value_type1)?;
		context.push_value(value_type2)?;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn validate_drop(context: &mut FunctionValidationContext) -> Result<InstructionOutcome, Error> {
		context.pop_any_value().map(|_| ())?;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn validate_select(context: &mut FunctionValidationContext) -> Result<InstructionOutcome, Error> {
		context.pop_value(ValueType::I32)?;
		let select_type = context.pop_any_value()?;
		context.pop_value(select_type)?;
		context.push_value(select_type)?;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn validate_get_local(context: &mut FunctionValidationContext, index: u32) -> Result<InstructionOutcome, Error> {
		let local_type = context.require_local(index)?;
		context.push_value(local_type)?;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn validate_set_local(context: &mut FunctionValidationContext, index: u32) -> Result<InstructionOutcome, Error> {
		let local_type = context.require_local(index)?;
		let value_type = context.pop_any_value()?;
		if local_type != value_type {
			return Err(Error::Validation(format!("Trying to update local {} of type {:?} with value of type {:?}", index, local_type, value_type)));
		}
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn validate_tee_local(context: &mut FunctionValidationContext, index: u32) -> Result<InstructionOutcome, Error> {
		let local_type = context.require_local(index)?;
		let value_type = context.tee_any_value()?;
		if local_type != value_type {
			return Err(Error::Validation(format!("Trying to update local {} of type {:?} with value of type {:?}", index, local_type, value_type)));
		}
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn validate_get_global(context: &mut FunctionValidationContext, index: u32) -> Result<InstructionOutcome, Error> {
		let global_type = context.require_global(index, None)?;
		context.push_value(global_type)?;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn validate_set_global(context: &mut FunctionValidationContext, index: u32) -> Result<InstructionOutcome, Error> {
		let global_type = context.require_global(index, Some(true))?;
		let value_type = context.pop_any_value()?;
		if global_type != value_type {
			return Err(Error::Validation(format!("Trying to update global {} of type {:?} with value of type {:?}", index, global_type, value_type)));
		}
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn validate_load(context: &mut FunctionValidationContext, align: u32, max_align: u32, value_type: ValueType) -> Result<InstructionOutcome, Error> {
		if align > max_align {
			return Err(Error::Validation(format!("Too large memory alignment {} (expected at most {})", align, max_align)));
		}

		context.require_memory(DEFAULT_MEMORY_INDEX)?;
		context.push_value(value_type)?;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn validate_store(context: &mut FunctionValidationContext, align: u32, max_align: u32, value_type: ValueType) -> Result<InstructionOutcome, Error> {
		if align > max_align {
			return Err(Error::Validation(format!("Too large memory alignment {} (expected at most {})", align, max_align)));
		}

		context.require_memory(DEFAULT_MEMORY_INDEX)?;
		context.pop_value(value_type)?;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn validate_loop(context: &mut FunctionValidationContext, block_type: BlockType, body: &[Opcode]) -> Result<InstructionOutcome, Error> {
		Validator::validate_block(context, block_type, body, Opcode::End)
	}

	fn validate_if(context: &mut FunctionValidationContext, block_type: BlockType, body: &[Opcode]) -> Result<InstructionOutcome, Error> {
		let body_len = body.len();
		let separator_index = body.iter()
			.position(|op| *op == Opcode::Else)
			.unwrap_or(body_len - 1);
		if separator_index != body_len - 1 {
			Validator::validate_block(context, block_type, &body[..separator_index + 1], Opcode::Else)?;
			Validator::validate_block(context, block_type, &body[separator_index+1..], Opcode::End)
		} else {
			Validator::validate_block(context, block_type, body, Opcode::End)
		}
	}

	fn validate_br(context: &mut FunctionValidationContext, idx: u32) -> Result<InstructionOutcome, Error> {
		context.require_label(idx)?;
		Ok(InstructionOutcome::Unreachable)
	}

	fn validate_br_if(context: &mut FunctionValidationContext, idx: u32) -> Result<InstructionOutcome, Error> {
		context.pop_value(ValueType::I32)?;
		context.require_label(idx)?;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn validate_br_table(context: &mut FunctionValidationContext, table: &Vec<u32>, default: u32) -> Result<InstructionOutcome, Error> {
		let default_block_type = context.require_label(default)?;
		for label in table {
			let label_block_type = context.require_label(*label)?;
			if default_block_type != label_block_type {
				return Err(Error::Validation(format!("Default label in br_table points to block of type {:?}, while other points to {:?}", default_block_type, label_block_type)));
			}
		}

		Ok(InstructionOutcome::Unreachable)
	}

	fn validate_return(context: &mut FunctionValidationContext) -> Result<InstructionOutcome, Error> {
		if let BlockType::Value(value_type) = context.return_type()? {
			context.tee_value(value_type)?;
		}
		Ok(InstructionOutcome::Unreachable)
	}

	fn validate_call(context: &mut FunctionValidationContext, idx: u32) -> Result<InstructionOutcome, Error> {
		let (argument_types, return_type) = context.require_function(idx)?;
		for argument_type in argument_types.iter().rev() {
			context.pop_value(*argument_type)?;
		}
		if let BlockType::Value(value_type) = return_type {
			context.push_value(value_type)?;
		}
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn validate_call_indirect(context: &mut FunctionValidationContext, idx: u32) -> Result<InstructionOutcome, Error> {
		context.require_table(DEFAULT_TABLE_INDEX, VariableType::AnyFunc)?;
		let (argument_types, return_type) = context.require_function_type(idx)?;
		for argument_type in argument_types.iter().rev() {
			context.pop_value(*argument_type)?;
		}
		if let BlockType::Value(value_type) = return_type {
			context.push_value(value_type)?;
		}
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn validate_current_memory(context: &mut FunctionValidationContext) -> Result<InstructionOutcome, Error> {
		context.require_memory(DEFAULT_MEMORY_INDEX)?;
		context.push_value(ValueType::I32)?;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn validate_grow_memory(context: &mut FunctionValidationContext) -> Result<InstructionOutcome, Error> {
		context.require_memory(DEFAULT_MEMORY_INDEX)?;
		context.pop_value(ValueType::I32)?;
		context.push_value(ValueType::I32)?;
		Ok(InstructionOutcome::RunNextInstruction)
	}
}

impl<'a> FunctionValidationContext<'a> {
	pub fn new(module: &'a Module, imports: &'a ModuleImports, locals: &'a [ValueType], value_stack_limit: usize, frame_stack_limit: usize, function: &FunctionType) -> Self {
		FunctionValidationContext {
			module: module,
			imports: imports,
			locals: locals,
			value_stack: StackWithLimit::with_limit(value_stack_limit),
			frame_stack: StackWithLimit::with_limit(frame_stack_limit),
			return_type: Some(function.return_type().map(BlockType::Value).unwrap_or(BlockType::NoResult)),
		}
	}

	pub fn push_value(&mut self, value_type: ValueType) -> Result<(), Error> {
		self.value_stack.push(value_type)
	}

	pub fn pop_value(&mut self, value_type: ValueType) -> Result<(), Error> {
		let stack_value_type = self.value_stack.pop()?;
		if stack_value_type == value_type {
			return Ok(());
		}

		Err(Error::Validation(format!("Expected value of type {:?} on top of stack. Got {:?}", value_type, stack_value_type)))
	}

	pub fn tee_value(&mut self, value_type: ValueType) -> Result<(), Error> {
		let stack_value_type = self.value_stack.top()?;
		if *stack_value_type == value_type {
			return Ok(());
		}

		Err(Error::Validation(format!("Expected value of type {:?} on top of stack. Got {:?}", value_type, stack_value_type)))
	}

	pub fn pop_any_value(&mut self) -> Result<ValueType, Error> {
		self.value_stack.pop()
	}

	pub fn tee_any_value(&mut self) -> Result<ValueType, Error> {
		self.value_stack.top().map(Clone::clone)
	}

	pub fn push_label(&mut self, block_type: BlockType) -> Result<(), Error> {
		self.frame_stack.push((block_type, self.value_stack.len()))
	}

	pub fn pop_label(&mut self) -> Result<Option<ValueType>, Error> {
		let (_, value_stack_len) = self.frame_stack.pop()?;
		let return_type = if self.value_stack.len() > value_stack_len {
			Some(self.value_stack.pop()?)
		} else {
			None
		};

		self.value_stack.resize(value_stack_len, ValueType::I32);
		Ok(return_type)
	}

	pub fn require_label(&self, idx: u32) -> Result<BlockType, Error> {
		self.frame_stack.get(idx as usize).map(|&(block_type, _)| block_type)	
	}

	pub fn return_type(&self) -> Result<BlockType, Error> {
		self.return_type.ok_or(Error::Validation("Trying to return from expression".into()))
	}

	pub fn require_local(&self, idx: u32) -> Result<ValueType, Error> {
		self.locals.get(idx as usize)
			.cloned()
			.ok_or(Error::Validation(format!("Trying to access local with index {} when there are only {} locals", idx, self.locals.len())))
	}

	pub fn require_global(&self, idx: u32, mutability: Option<bool>) -> Result<ValueType, Error> {
		match self.imports.parse_global_index(ItemIndex::IndexSpace(idx)) {
			ItemIndex::IndexSpace(_) => unreachable!("parse_global_index is intended to resolve this"),
			ItemIndex::Internal(internal_idx) => self.module
				.global_section().ok_or(Error::Validation(format!("Trying to access internal global {} in module without global section", internal_idx)))
				.and_then(|s| s.entries().get(internal_idx as usize).ok_or(Error::Validation(format!("Trying to access internal global {} in module with {} globals", internal_idx, s.entries().len()))))
				.and_then(|g| match mutability {
					Some(true) if !g.global_type().is_mutable() => Err(Error::Validation(format!("Expected internal global {} to be mutable", internal_idx))),
					Some(false) if g.global_type().is_mutable() => Err(Error::Validation(format!("Expected internal global {} to be immutable", internal_idx))),
					_ => Ok(g),
				})
				.map(|g| g.global_type().content_type()),
			ItemIndex::External(external_idx) => self.module
				.import_section().ok_or(Error::Validation(format!("Trying to access external global {} in module without import section", external_idx)))
				.and_then(|s| s.entries().get(external_idx as usize).ok_or(Error::Validation(format!("Trying to access external global with index {} in module with {}-entries import section", external_idx, s.entries().len()))))
				.and_then(|e| match e.external() {
					&External::Global(ref g) => {
						match mutability {
							Some(true) if !g.is_mutable() => Err(Error::Validation(format!("Expected external global {} to be mutable", external_idx))),
							Some(false) if g.is_mutable() => Err(Error::Validation(format!("Expected external global {} to be immutable", external_idx))),
							_ => Ok(g.content_type()),
						}
					},
					_ => Err(Error::Validation(format!("Import entry {} expected to import global", external_idx)))
				}),
		}
	}

	pub fn require_memory(&self, idx: u32) -> Result<(), Error> {
		match self.imports.parse_memory_index(ItemIndex::IndexSpace(idx)) {
			ItemIndex::IndexSpace(_) => unreachable!("parse_memory_index is intended to resolve this"),
			ItemIndex::Internal(internal_idx) => self.module
				.memory_section().ok_or(Error::Validation(format!("Trying to access internal memory {} in module without memory section", internal_idx)))
				.and_then(|s| s.entries().get(internal_idx as usize).ok_or(Error::Validation(format!("Trying to access internal memory {} in module with {} memory regions", internal_idx, s.entries().len()))))
				.map(|_| ()),
			ItemIndex::External(external_idx) => self.module
				.import_section().ok_or(Error::Validation(format!("Trying to access external memory {} in module without import section", external_idx)))
				.and_then(|s| s.entries().get(external_idx as usize).ok_or(Error::Validation(format!("Trying to access external memory with index {} in module with {}-entries import section", external_idx, s.entries().len()))))
				.and_then(|e| match e.external() {
					&External::Memory(_) => Ok(()),
					_ => Err(Error::Validation(format!("Import entry {} expected to import memory", external_idx)))
				}),
		}
	}

	pub fn require_table(&self, idx: u32, _variable_type: VariableType) -> Result<(), Error> {
		match self.imports.parse_table_index(ItemIndex::IndexSpace(idx)) {
			ItemIndex::IndexSpace(_) => unreachable!("parse_table_index is intended to resolve this"),
			ItemIndex::Internal(internal_idx) => self.module
				.table_section().ok_or(Error::Validation(format!("Trying to access internal table {} in module without table section", internal_idx)))
				.and_then(|s| s.entries().get(internal_idx as usize).ok_or(Error::Validation(format!("Trying to access internal table {} in module with {} tables", internal_idx, s.entries().len()))))
				.and_then(|_| Ok(())), // TODO: check variable type
			ItemIndex::External(external_idx) => self.module
				.import_section().ok_or(Error::Validation(format!("Trying to access external table {} in module without import section", external_idx)))
				.and_then(|s| s.entries().get(external_idx as usize).ok_or(Error::Validation(format!("Trying to access external table with index {} in module with {}-entries import section", external_idx, s.entries().len()))))
				.and_then(|e| match e.external() {
					&External::Table(_) => Ok(()), // TODO: check variable type
					_ => Err(Error::Validation(format!("Import entry {} expected to import table", external_idx)))
				}),
		}
	}

	pub fn require_function(&self, idx: u32) -> Result<(Vec<ValueType>, BlockType), Error> {
		match self.imports.parse_function_index(ItemIndex::IndexSpace(idx)) {
			ItemIndex::IndexSpace(_) => unreachable!("parse_function_index is intended to resolve this"),
			ItemIndex::Internal(internal_idx) => self.module
				.function_section().ok_or(Error::Validation(format!("Trying to access internal function {} in module without function section", internal_idx)))
				.and_then(|s| s.entries().get(internal_idx as usize).map(|f| f.type_ref()).ok_or(Error::Validation(format!("Trying to access internal function {} in module with {} functions", internal_idx, s.entries().len()))))
				.and_then(|tidx| self.require_function_type(tidx)),
			ItemIndex::External(external_idx) => self.module
				.import_section().ok_or(Error::Validation(format!("Trying to access external function {} in module without import section", external_idx)))
				.and_then(|s| s.entries().get(external_idx as usize).ok_or(Error::Validation(format!("Trying to access external function with index {} in module with {}-entries import section", external_idx, s.entries().len()))))
				.and_then(|e| match e.external() {
					&External::Function(tidx) => Ok(tidx),
					_ => Err(Error::Validation(format!("Import entry {} expected to import function", external_idx)))
				})
				.and_then(|tidx| self.require_function_type(tidx)),
		}
	}

	pub fn require_function_type(&self, idx: u32) -> Result<(Vec<ValueType>, BlockType), Error> {
		self.module
			.type_section().ok_or(Error::Validation(format!("Trying to access internal function {} in module without type section", idx)))
			.and_then(|ts| match ts.types().get(idx as usize) {
				Some(&Type::Function(ref function_type)) => Ok((function_type.params().to_vec(), function_type.return_type().map(BlockType::Value).unwrap_or(BlockType::NoResult))),
				_ => Err(Error::Validation(format!("Trying to access internal function {} with wrong type", idx))),
			})
	}
}
