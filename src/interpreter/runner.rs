use std::mem;
use std::ops;
use std::u32;
use std::fmt::Display;
use elements::{Opcode, BlockType, FunctionType};
use interpreter::Error;
use interpreter::module::{ModuleInstance, ModuleInstanceInterface, CallerContext, ItemIndex};
use interpreter::stack::StackWithLimit;
use interpreter::value::{RuntimeValue, TryInto, WrapInto, TryTruncateInto, ExtendInto, TransmuteInto,
	ArithmeticOps, Integer, Float, LittleEndianConvert};
use interpreter::variable::VariableInstance;

const DEFAULT_MEMORY_INDEX: u32 = 0;
const DEFAULT_TABLE_INDEX: u32 = 0;

pub struct Interpreter;

/// Function execution context.
pub struct FunctionContext<'a> {
	/// Module instance.
	module: &'a ModuleInstance,
	/// Function return type.
	return_type: BlockType,
	/// Local variables.
	locals: Vec<VariableInstance>,
	/// Values stack.
	value_stack: StackWithLimit<RuntimeValue>,
	/// Blocks frames stack.
	frame_stack: StackWithLimit<BlockFrame>,
	/// Current instruction position.
	position: usize,
}

#[derive(Debug, Clone)]
pub enum InstructionOutcome {
	/// Continue with current instruction.
	RunInstruction,
	/// Continue with next instruction.
	RunNextInstruction,
	/// Branch to given frame.
	Branch(usize),
	/// End current frame.
	End,
	/// Return from current function block.
	Return,
}

#[derive(Debug, Clone)]
pub struct BlockFrame {
	// A label for reference from branch instructions.
	branch_position: usize,
	// A label for reference from end instructions.
	end_position: usize,
	// A limit integer value, which is an index into the value stack indicating where to reset it to on a branch to that label.
	value_limit: usize,
	// A signature, which is a block signature type indicating the number and types of result values of the region.
	signature: BlockType,
}

impl Interpreter {
	pub fn run_function(context: &mut FunctionContext, body: &[Opcode]) -> Result<Option<RuntimeValue>, Error> {
		Interpreter::execute_block(context, body)?;
		match context.return_type {
			BlockType::Value(_) => Ok(Some(context.value_stack_mut().pop()?)),
			BlockType::NoResult => Ok(None),
		}
	}

	fn run_instruction(context: &mut FunctionContext, opcode: &Opcode) -> Result<InstructionOutcome, Error> {
		match opcode {
			&Opcode::Unreachable => Interpreter::run_unreachable(context),
			&Opcode::Nop => Interpreter::run_nop(context),
			&Opcode::Block(block_type, ref ops) => Interpreter::run_block(context, block_type, ops.elements()),
			&Opcode::Loop(block_type, ref ops) => Interpreter::run_loop(context, block_type, ops.elements()),
			&Opcode::If(block_type, ref ops) => Interpreter::run_if(context, block_type, ops.elements()),
			&Opcode::Else => Interpreter::run_else(context),
			&Opcode::End => Interpreter::run_end(context),
			&Opcode::Br(idx) => Interpreter::run_br(context, idx),
			&Opcode::BrIf(idx) => Interpreter::run_br_if(context, idx),
			&Opcode::BrTable(ref table, default) => Interpreter::run_br_table(context, table, default),
			&Opcode::Return => Interpreter::run_return(context),

			&Opcode::Call(index) => Interpreter::run_call(context, index),
			&Opcode::CallIndirect(index, _reserved) => Interpreter::run_call_indirect(context, index),

			&Opcode::Drop => Interpreter::run_drop(context),
			&Opcode::Select => Interpreter::run_select(context),

			&Opcode::GetLocal(index) => Interpreter::run_get_local(context, index),
			&Opcode::SetLocal(index) => Interpreter::run_set_local(context, index),
			&Opcode::TeeLocal(index) => Interpreter::run_tee_local(context, index),
			&Opcode::GetGlobal(index) => Interpreter::run_get_global(context, index),
			&Opcode::SetGlobal(index) => Interpreter::run_set_global(context, index),

			&Opcode::I32Load(offset, align) => Interpreter::run_load::<i32>(context, offset, align),
			&Opcode::I64Load(offset, align) => Interpreter::run_load::<i64>(context, offset, align),
			&Opcode::F32Load(offset, align) => Interpreter::run_load::<f32>(context, offset, align),
			&Opcode::F64Load(offset, align) => Interpreter::run_load::<f64>(context, offset, align),
			&Opcode::I32Load8S(offset, align) => Interpreter::run_load_extend::<i8, i32>(context, offset, align),
			&Opcode::I32Load8U(offset, align) => Interpreter::run_load_extend::<u8, i32>(context, offset, align),
			&Opcode::I32Load16S(offset, align) => Interpreter::run_load_extend::<i16, i32>(context, offset, align),
			&Opcode::I32Load16U(offset, align) => Interpreter::run_load_extend::<u16, i32>(context, offset, align),
			&Opcode::I64Load8S(offset, align) => Interpreter::run_load_extend::<i8, i64>(context, offset, align),
			&Opcode::I64Load8U(offset, align) => Interpreter::run_load_extend::<u8, i64>(context, offset, align),
			&Opcode::I64Load16S(offset, align) => Interpreter::run_load_extend::<i16, i64>(context, offset, align),
			&Opcode::I64Load16U(offset, align) => Interpreter::run_load_extend::<u16, i64>(context, offset, align),
			&Opcode::I64Load32S(offset, align) => Interpreter::run_load_extend::<i32, i64>(context, offset, align),
			&Opcode::I64Load32U(offset, align) => Interpreter::run_load_extend::<u32, i64>(context, offset, align),

			&Opcode::I32Store(offset, align) => Interpreter::run_store::<i32>(context, offset, align),
			&Opcode::I64Store(offset, align) => Interpreter::run_store::<i64>(context, offset, align),
			&Opcode::F32Store(offset, align) => Interpreter::run_store::<f32>(context, offset, align),
			&Opcode::F64Store(offset, align) => Interpreter::run_store::<f64>(context, offset, align),
			&Opcode::I32Store8(offset, align) => Interpreter::run_store_wrap::<i32, i8>(context, offset, align),
			&Opcode::I32Store16(offset, align) => Interpreter::run_store_wrap::<i32, i16>(context, offset, align),
			&Opcode::I64Store8(offset, align) => Interpreter::run_store_wrap::<i64, i8>(context, offset, align),
			&Opcode::I64Store16(offset, align) => Interpreter::run_store_wrap::<i64, i16>(context, offset, align),
			&Opcode::I64Store32(offset, align) => Interpreter::run_store_wrap::<i64, i32>(context, offset, align),

			&Opcode::CurrentMemory(_) => Interpreter::run_current_memory(context),
			&Opcode::GrowMemory(_) => Interpreter::run_grow_memory(context),

			&Opcode::I32Const(val) => Interpreter::run_const(context, val.into()),
			&Opcode::I64Const(val) => Interpreter::run_const(context, val.into()),
			&Opcode::F32Const(val) => Interpreter::run_const(context, RuntimeValue::decode_f32(val)),
			&Opcode::F64Const(val) => Interpreter::run_const(context, RuntimeValue::decode_f64(val)),

			&Opcode::I32Eqz => Interpreter::run_eqz::<i32>(context),
			&Opcode::I32Eq => Interpreter::run_eq::<i32>(context),
			&Opcode::I32Ne => Interpreter::run_ne::<i32>(context),
			&Opcode::I32LtS => Interpreter::run_lt::<i32>(context),
			&Opcode::I32LtU => Interpreter::run_lt::<u32>(context),
			&Opcode::I32GtS => Interpreter::run_gt::<i32>(context),
			&Opcode::I32GtU => Interpreter::run_gt::<u32>(context),
			&Opcode::I32LeS => Interpreter::run_lte::<i32>(context),
			&Opcode::I32LeU => Interpreter::run_lte::<u32>(context),
			&Opcode::I32GeS => Interpreter::run_gte::<i32>(context),
			&Opcode::I32GeU => Interpreter::run_gte::<u32>(context),

			&Opcode::I64Eqz => Interpreter::run_eqz::<i64>(context),
			&Opcode::I64Eq => Interpreter::run_eq::<i64>(context),
			&Opcode::I64Ne => Interpreter::run_ne::<i64>(context),
			&Opcode::I64LtS => Interpreter::run_lt::<i64>(context),
			&Opcode::I64LtU => Interpreter::run_lt::<u64>(context),
			&Opcode::I64GtS => Interpreter::run_gt::<i64>(context),
			&Opcode::I64GtU => Interpreter::run_gt::<u64>(context),
			&Opcode::I64LeS => Interpreter::run_lte::<i64>(context),
			&Opcode::I64LeU => Interpreter::run_lte::<u64>(context),
			&Opcode::I64GeS => Interpreter::run_gte::<i64>(context),
			&Opcode::I64GeU => Interpreter::run_gte::<u64>(context),

			&Opcode::F32Eq => Interpreter::run_eq::<f32>(context),
			&Opcode::F32Ne => Interpreter::run_ne::<f32>(context),
			&Opcode::F32Lt => Interpreter::run_lt::<f32>(context),
			&Opcode::F32Gt => Interpreter::run_gt::<f32>(context),
			&Opcode::F32Le => Interpreter::run_lte::<f32>(context),
			&Opcode::F32Ge => Interpreter::run_gte::<f32>(context),

			&Opcode::F64Eq => Interpreter::run_eq::<f64>(context),
			&Opcode::F64Ne => Interpreter::run_ne::<f64>(context),
			&Opcode::F64Lt => Interpreter::run_lt::<f64>(context),
			&Opcode::F64Gt => Interpreter::run_gt::<f64>(context),
			&Opcode::F64Le => Interpreter::run_lte::<f64>(context),
			&Opcode::F64Ge => Interpreter::run_gte::<f64>(context),

			&Opcode::I32Clz => Interpreter::run_clz::<i32>(context),
			&Opcode::I32Ctz => Interpreter::run_ctz::<i32>(context),
			&Opcode::I32Popcnt => Interpreter::run_popcnt::<i32>(context),
			&Opcode::I32Add => Interpreter::run_add::<i32>(context),
			&Opcode::I32Sub => Interpreter::run_sub::<i32>(context),
			&Opcode::I32Mul => Interpreter::run_mul::<i32>(context),
			&Opcode::I32DivS => Interpreter::run_div::<i32, i32>(context),
			&Opcode::I32DivU => Interpreter::run_div::<i32, u32>(context),
			&Opcode::I32RemS => Interpreter::run_rem::<i32, i32>(context),
			&Opcode::I32RemU => Interpreter::run_rem::<i32, u32>(context),
			&Opcode::I32And => Interpreter::run_and::<i32>(context),
			&Opcode::I32Or => Interpreter::run_or::<i32>(context),
			&Opcode::I32Xor => Interpreter::run_xor::<i32>(context),
			&Opcode::I32Shl => Interpreter::run_shl::<i32>(context),
			&Opcode::I32ShrS => Interpreter::run_shr::<i32, i32>(context),
			&Opcode::I32ShrU => Interpreter::run_shr::<i32, u32>(context),
			&Opcode::I32Rotl => Interpreter::run_rotl::<i32>(context),
			&Opcode::I32Rotr => Interpreter::run_rotr::<i32>(context),

			&Opcode::I64Clz => Interpreter::run_clz::<i64>(context),
			&Opcode::I64Ctz => Interpreter::run_ctz::<i64>(context),
			&Opcode::I64Popcnt => Interpreter::run_popcnt::<i64>(context),
			&Opcode::I64Add => Interpreter::run_add::<i64>(context),
			&Opcode::I64Sub => Interpreter::run_sub::<i64>(context),
			&Opcode::I64Mul => Interpreter::run_mul::<i64>(context),
			&Opcode::I64DivS => Interpreter::run_div::<i64, i64>(context),
			&Opcode::I64DivU => Interpreter::run_div::<i64, u64>(context),
			&Opcode::I64RemS => Interpreter::run_rem::<i64, i64>(context),
			&Opcode::I64RemU => Interpreter::run_rem::<i64, u64>(context),
			&Opcode::I64And => Interpreter::run_and::<i64>(context),
			&Opcode::I64Or => Interpreter::run_or::<i64>(context),
			&Opcode::I64Xor => Interpreter::run_xor::<i64>(context),
			&Opcode::I64Shl => Interpreter::run_shl::<i64>(context),
			&Opcode::I64ShrS => Interpreter::run_shr::<i64, i64>(context),
			&Opcode::I64ShrU => Interpreter::run_shr::<i64, u64>(context),
			&Opcode::I64Rotl => Interpreter::run_rotl::<i64>(context),
			&Opcode::I64Rotr => Interpreter::run_rotr::<i64>(context),

			&Opcode::F32Abs => Interpreter::run_abs::<f32>(context),
			&Opcode::F32Neg => Interpreter::run_neg::<f32>(context),
			&Opcode::F32Ceil => Interpreter::run_ceil::<f32>(context),
			&Opcode::F32Floor => Interpreter::run_floor::<f32>(context),
			&Opcode::F32Trunc => Interpreter::run_trunc::<f32>(context),
			&Opcode::F32Nearest => Interpreter::run_nearest::<f32>(context),
			&Opcode::F32Sqrt => Interpreter::run_sqrt::<f32>(context),
			&Opcode::F32Add => Interpreter::run_add::<f32>(context),
			&Opcode::F32Sub => Interpreter::run_sub::<f32>(context),
			&Opcode::F32Mul => Interpreter::run_mul::<f32>(context),
			&Opcode::F32Div => Interpreter::run_div::<f32, f32>(context),
			&Opcode::F32Min => Interpreter::run_min::<f32>(context),
			&Opcode::F32Max => Interpreter::run_max::<f32>(context),
			&Opcode::F32Copysign => Interpreter::run_copysign::<f32>(context),

			&Opcode::F64Abs => Interpreter::run_abs::<f64>(context),
			&Opcode::F64Neg => Interpreter::run_neg::<f64>(context),
			&Opcode::F64Ceil => Interpreter::run_ceil::<f64>(context),
			&Opcode::F64Floor => Interpreter::run_floor::<f64>(context),
			&Opcode::F64Trunc => Interpreter::run_trunc::<f64>(context),
			&Opcode::F64Nearest => Interpreter::run_nearest::<f64>(context),
			&Opcode::F64Sqrt => Interpreter::run_sqrt::<f64>(context),
			&Opcode::F64Add => Interpreter::run_add::<f64>(context),
			&Opcode::F64Sub => Interpreter::run_sub::<f64>(context),
			&Opcode::F64Mul => Interpreter::run_mul::<f64>(context),
			&Opcode::F64Div => Interpreter::run_div::<f64, f64>(context),
			&Opcode::F64Min => Interpreter::run_min::<f64>(context),
			&Opcode::F64Max => Interpreter::run_max::<f64>(context),
			&Opcode::F64Copysign => Interpreter::run_copysign::<f64>(context),

			&Opcode::I32WarpI64 => Interpreter::run_wrap::<i64, i32>(context),
			&Opcode::I32TruncSF32 => Interpreter::run_trunc_to_int::<f32, i32, i32>(context),
			&Opcode::I32TruncUF32 => Interpreter::run_trunc_to_int::<f32, u32, i32>(context),
			&Opcode::I32TruncSF64 => Interpreter::run_trunc_to_int::<f64, i32, i32>(context),
			&Opcode::I32TruncUF64 => Interpreter::run_trunc_to_int::<f64, u32, i32>(context),
			&Opcode::I64ExtendSI32 => Interpreter::run_extend::<i32, i64, i64>(context),
			&Opcode::I64ExtendUI32 => Interpreter::run_extend::<u32, u64, i64>(context),
			&Opcode::I64TruncSF32 => Interpreter::run_trunc_to_int::<f32, i64, i64>(context),
			&Opcode::I64TruncUF32 => Interpreter::run_trunc_to_int::<f32, u64, i64>(context),
			&Opcode::I64TruncSF64 => Interpreter::run_trunc_to_int::<f64, i64, i64>(context),
			&Opcode::I64TruncUF64 => Interpreter::run_trunc_to_int::<f64, u64, i64>(context),
			&Opcode::F32ConvertSI32 => Interpreter::run_extend::<i32, f32, f32>(context),
			&Opcode::F32ConvertUI32 => Interpreter::run_extend::<u32, f32, f32>(context),
			&Opcode::F32ConvertSI64 => Interpreter::run_wrap::<i64, f32>(context),
			&Opcode::F32ConvertUI64 => Interpreter::run_wrap::<u64, f32>(context),
			&Opcode::F32DemoteF64 => Interpreter::run_wrap::<f64, f32>(context),
			&Opcode::F64ConvertSI32 => Interpreter::run_extend::<i32, f64, f64>(context),
			&Opcode::F64ConvertUI32 => Interpreter::run_extend::<u32, f64, f64>(context),
			&Opcode::F64ConvertSI64 => Interpreter::run_extend::<i64, f64, f64>(context),
			&Opcode::F64ConvertUI64 => Interpreter::run_extend::<u64, f64, f64>(context),
			&Opcode::F64PromoteF32 => Interpreter::run_extend::<f32, f64, f64>(context),

			&Opcode::I32ReinterpretF32 => Interpreter::run_reinterpret::<f32, i32>(context),
			&Opcode::I64ReinterpretF64 => Interpreter::run_reinterpret::<f64, i64>(context),
			&Opcode::F32ReinterpretI32 => Interpreter::run_reinterpret::<i32, f32>(context),
			&Opcode::F64ReinterpretI64 => Interpreter::run_reinterpret::<i64, f64>(context),
		}
	}

	fn run_unreachable(_context: &mut FunctionContext) -> Result<InstructionOutcome, Error> {
		Err(Error::Trap("programmatic".into()))
	}

	fn run_nop(_context: &mut FunctionContext) -> Result<InstructionOutcome, Error> {
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn run_block(context: &mut FunctionContext, block_type: BlockType, body: &[Opcode]) -> Result<InstructionOutcome, Error> {
		let frame_position = context.position + 1;
		context.push_frame(frame_position, frame_position, block_type.clone())?;
		Interpreter::execute_block(context, body)
	}

	fn run_loop(context: &mut FunctionContext, block_type: BlockType, body: &[Opcode]) -> Result<InstructionOutcome, Error> {
		let frame_position = context.position;
		context.push_frame(frame_position, frame_position + 1, block_type.clone())?;
		Interpreter::execute_block(context,  body)
	}

	fn run_if(context: &mut FunctionContext, block_type: BlockType, body: &[Opcode]) -> Result<InstructionOutcome, Error> {
		let body_len = body.len();
		let else_index = body.iter().position(|op| *op == Opcode::Else).unwrap_or(body_len - 1);
		let (begin_index, end_index) = if context.value_stack_mut().pop_as()? {
			(0, else_index + 1)
		} else {
			(else_index + 1, body_len)
		};

		if begin_index != end_index {
			let frame_position = context.position + 1;
			context.push_frame(frame_position, frame_position, block_type.clone())?;
			Interpreter::execute_block(context, &body[begin_index..end_index])
		} else {
			Ok(InstructionOutcome::RunNextInstruction)
		}
	}

	fn run_else(_context: &mut FunctionContext) -> Result<InstructionOutcome, Error> {
		Ok(InstructionOutcome::End)
	}

	fn run_end(_context: &mut FunctionContext) -> Result<InstructionOutcome, Error> {
		Ok(InstructionOutcome::End)
	}

	fn run_br(_context: &mut FunctionContext, label_idx: u32) -> Result<InstructionOutcome, Error> {
		Ok(InstructionOutcome::Branch(label_idx as usize))
	}

	fn run_br_if(context: &mut FunctionContext, label_idx: u32) -> Result<InstructionOutcome, Error> {
		if context.value_stack_mut().pop_as()? {
			Ok(InstructionOutcome::Branch(label_idx as usize))
		} else {
			Ok(InstructionOutcome::RunNextInstruction)
		}
	}

	fn run_br_table(context: &mut FunctionContext, table: &Vec<u32>, default: u32) -> Result<InstructionOutcome, Error> {
		let index: u32 = context.value_stack_mut().pop_as()?;
		Ok(InstructionOutcome::Branch(table.get(index as usize).cloned().unwrap_or(default) as usize))
	}

	fn run_return(_context: &mut FunctionContext) -> Result<InstructionOutcome, Error> {
		Ok(InstructionOutcome::Return)
	}

	fn run_call(context: &mut FunctionContext, func_idx: u32) -> Result<InstructionOutcome, Error> {
		context.call_function(func_idx)
			.and_then(|r| r.map(|r| context.value_stack_mut().push(r)).unwrap_or(Ok(())))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_call_indirect(context: &mut FunctionContext, type_idx: u32) -> Result<InstructionOutcome, Error> {
		let table_func_idx: u32 = context.value_stack_mut().pop_as()?;
		context.call_function_indirect(DEFAULT_TABLE_INDEX, type_idx, table_func_idx)
			.and_then(|r| r.map(|r| context.value_stack_mut().push(r)).unwrap_or(Ok(())))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_drop(context: &mut FunctionContext) -> Result<InstructionOutcome, Error> {
		context
			.value_stack_mut()
			.pop()
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_select(context: &mut FunctionContext) -> Result<InstructionOutcome, Error> {
		context
			.value_stack_mut()
			.pop_triple()
			.and_then(|(left, mid, right)|
				match (left, mid, right.try_into()) {
					(left, mid, Ok(condition)) => Ok((left, mid, condition)),
					_ => Err(Error::Stack("expected to get int value from stack".into()))
				}
			)
			.map(|(left, mid, condition)| if condition { left } else { mid })
			.map(|val| context.value_stack_mut().push(val))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_get_local(context: &mut FunctionContext, index: u32) -> Result<InstructionOutcome, Error> {
		context.get_local(index as usize)
			.map(|value| context.value_stack_mut().push(value))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_set_local(context: &mut FunctionContext, index: u32) -> Result<InstructionOutcome, Error> {
		let arg = context.value_stack_mut().pop()?;
		context.set_local(index as usize, arg)
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_tee_local(context: &mut FunctionContext, index: u32) -> Result<InstructionOutcome, Error> {
		let arg = context.value_stack().top()?.clone();
		context.set_local(index as usize, arg)
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_get_global(context: &mut FunctionContext, index: u32) -> Result<InstructionOutcome, Error> {
		context.module()
			.global(ItemIndex::IndexSpace(index))
			.and_then(|g| context.value_stack_mut().push(g.get()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_set_global(context: &mut FunctionContext, index: u32) -> Result<InstructionOutcome, Error> {
		context
			.value_stack_mut()
			.pop()
			.and_then(|v| context.module().global(ItemIndex::IndexSpace(index)).and_then(|g| g.set(v)))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_load<T>(context: &mut FunctionContext, offset: u32, _align: u32) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<T>, T: LittleEndianConvert {
		let address = effective_address(offset, context.value_stack_mut().pop_as()?)?;
		context.module()
			.memory(ItemIndex::IndexSpace(DEFAULT_MEMORY_INDEX))
			.and_then(|m| m.get(address, mem::size_of::<T>()))
			.and_then(|b| T::from_little_endian(b))
			.and_then(|n| context.value_stack_mut().push(n.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_load_extend<T, U>(context: &mut FunctionContext, offset: u32, _align: u32) -> Result<InstructionOutcome, Error>
		where T: ExtendInto<U>, RuntimeValue: From<U>, T: LittleEndianConvert {
		let address = effective_address(offset, context.value_stack_mut().pop_as()?)?;
		let stack_value: U = context.module()
			.memory(ItemIndex::IndexSpace(DEFAULT_MEMORY_INDEX))
			.and_then(|m| m.get(address, mem::size_of::<T>()))
			.and_then(|b| T::from_little_endian(b))
			.map(|v| v.extend_into())?;
		context
			.value_stack_mut()
			.push(stack_value.into())
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_store<T>(context: &mut FunctionContext, offset: u32, _align: u32) -> Result<InstructionOutcome, Error>
		where RuntimeValue: TryInto<T, Error>, T: LittleEndianConvert {
		let stack_value = context
			.value_stack_mut()
			.pop_as::<T>()
			.map(|n| n.into_little_endian())?;
		let address = effective_address(offset, context.value_stack_mut().pop_as::<u32>()?)?;
		context.module()
			.memory(ItemIndex::IndexSpace(DEFAULT_MEMORY_INDEX))
			.and_then(|m| m.set(address, &stack_value))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_store_wrap<T, U>(context: &mut FunctionContext, offset: u32, _align: u32) -> Result<InstructionOutcome, Error>
		where RuntimeValue: TryInto<T, Error>, T: WrapInto<U>, U: LittleEndianConvert {
		let stack_value: T = context.value_stack_mut().pop().and_then(|v| v.try_into())?;
		let stack_value = stack_value.wrap_into().into_little_endian();
		let address = effective_address(offset, context.value_stack_mut().pop_as::<u32>()?)?;
		context.module()
			.memory(ItemIndex::IndexSpace(DEFAULT_MEMORY_INDEX))
			.and_then(|m| m.set(address, &stack_value))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_current_memory(context: &mut FunctionContext) -> Result<InstructionOutcome, Error> {
		context.module()
			.memory(ItemIndex::IndexSpace(DEFAULT_MEMORY_INDEX))
			.map(|m| m.size())
			.and_then(|s| context.value_stack_mut().push(RuntimeValue::I64(s as i64)))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_grow_memory(context: &mut FunctionContext) -> Result<InstructionOutcome, Error> {
		let pages: u32 = context.value_stack_mut().pop_as()?;
		context.module()
			.memory(ItemIndex::IndexSpace(DEFAULT_MEMORY_INDEX))
			.and_then(|m| m.grow(pages))
			.and_then(|m| context.value_stack_mut().push(RuntimeValue::I32(m as i32)))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_const(context: &mut FunctionContext, val: RuntimeValue) -> Result<InstructionOutcome, Error> {
		context
			.value_stack_mut()
			.push(val)
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_eqz<T>(context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: TryInto<T, Error>, T: PartialEq<T> + Default {
		context
			.value_stack_mut()
			.pop_as::<T>()
			.map(|v| RuntimeValue::I32(if v == Default::default() { 1 } else { 0 }))
			.and_then(|v| context.value_stack_mut().push(v))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_eq<T>(context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: TryInto<T, Error>, T: PartialEq<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| RuntimeValue::I32(if left == right { 1 } else { 0 }))
			.and_then(|v| context.value_stack_mut().push(v))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_ne<T>(context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: TryInto<T, Error>, T: PartialEq<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| RuntimeValue::I32(if left != right { 1 } else { 0 }))
			.and_then(|v| context.value_stack_mut().push(v))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_lt<T>(context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: TryInto<T, Error>, T: PartialOrd<T> + Display {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| RuntimeValue::I32(if left < right { 1 } else { 0 }))
			.and_then(|v| context.value_stack_mut().push(v))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_gt<T>(context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: TryInto<T, Error>, T: PartialOrd<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| RuntimeValue::I32(if left > right { 1 } else { 0 }))
			.and_then(|v| context.value_stack_mut().push(v))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_lte<T>(context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: TryInto<T, Error>, T: PartialOrd<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| RuntimeValue::I32(if left <= right { 1 } else { 0 }))
			.and_then(|v| context.value_stack_mut().push(v))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_gte<T>(context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: TryInto<T, Error>, T: PartialOrd<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| RuntimeValue::I32(if left >= right { 1 } else { 0 }))
			.and_then(|v| context.value_stack_mut().push(v))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_clz<T>(context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: Integer<T> {
		context
			.value_stack_mut()
			.pop_as::<T>()
			.map(|v| v.leading_zeros())
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_ctz<T>(context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: Integer<T> {
		context
			.value_stack_mut()
			.pop_as::<T>()
			.map(|v| v.trailing_zeros())
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_popcnt<T>(context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: Integer<T> {
		context
			.value_stack_mut()
			.pop_as::<T>()
			.map(|v| v.count_ones())
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_add<T>(context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: ArithmeticOps<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| left.add(right))
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_sub<T>(context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: ArithmeticOps<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| left.sub(right))
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_mul<T>(context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: ArithmeticOps<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| left.mul(right))
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_div<T, U>(context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: TransmuteInto<U> + Display, U: ArithmeticOps<U> + TransmuteInto<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| (left.transmute_into(), right.transmute_into()))
			.map(|(left, right)| left.div(right))
			.map(|v| v.transmute_into())
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_rem<T, U>(context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: TransmuteInto<U>, U: Integer<U> + TransmuteInto<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| (left.transmute_into(), right.transmute_into()))
			.map(|(left, right)| left.rem(right))
			.map(|v| v.transmute_into())
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_and<T>(context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<<T as ops::BitAnd>::Output> + TryInto<T, Error>, T: ops::BitAnd<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| left.bitand(right))
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_or<T>(context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<<T as ops::BitOr>::Output> + TryInto<T, Error>, T: ops::BitOr<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| left.bitor(right))
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_xor<T>(context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<<T as ops::BitXor>::Output> + TryInto<T, Error>, T: ops::BitXor<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| left.bitxor(right))
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_shl<T>(context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<<T as ops::Shl<T>>::Output> + TryInto<T, Error>, T: ops::Shl<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| left.shl(right))
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_shr<T, U>(context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: TransmuteInto<U>, U: ops::Shr<U>, <U as ops::Shr<U>>::Output: TransmuteInto<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| (left.transmute_into(), right.transmute_into()))
			.map(|(left, right)| left.shr(right))
			.map(|v| v.transmute_into())
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_rotl<T>(context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: Integer<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| left.rotl(right))
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_rotr<T>(context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: Integer<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| left.rotr(right))
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_abs<T>(context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: Float<T> {
		context
			.value_stack_mut()
			.pop_as::<T>()
			.map(|v| v.abs())
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_neg<T>(context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<<T as ops::Neg>::Output> + TryInto<T, Error>, T: ops::Neg {
		context
			.value_stack_mut()
			.pop_as::<T>()
			.map(|v| v.neg())
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_ceil<T>(context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: Float<T> {
		context
			.value_stack_mut()
			.pop_as::<T>()
			.map(|v| v.ceil())
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_floor<T>(context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: Float<T> {
		context
			.value_stack_mut()
			.pop_as::<T>()
			.map(|v| v.floor())
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_trunc<T>(context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: Float<T> {
		context
			.value_stack_mut()
			.pop_as::<T>()
			.map(|v| v.trunc())
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_nearest<T>(context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: Float<T> {
		context
			.value_stack_mut()
			.pop_as::<T>()
			.map(|v| v.round())
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_sqrt<T>(context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: Float<T> {
		context
			.value_stack_mut()
			.pop_as::<T>()
			.map(|v| v.sqrt())
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_min<T>(context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: Float<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| left.min(right))
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_max<T>(context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: Float<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| left.max(right))
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_copysign<T>(context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: Float<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| left.copysign(right))
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_wrap<T, U>(context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<U> + TryInto<T, Error>, T: WrapInto<U> {
		context
			.value_stack_mut()
			.pop_as::<T>()
			.map(|v| v.wrap_into())
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_trunc_to_int<T, U, V>(context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<V> + TryInto<T, Error>, T: TryTruncateInto<U, Error>, U: TransmuteInto<V>,  {
		context
			.value_stack_mut()
			.pop_as::<T>()
			.and_then(|v| v.try_truncate_into())
			.map(|v| v.transmute_into())
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_extend<T, U, V>(context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<V> + TryInto<T, Error>, T: ExtendInto<U>, U: TransmuteInto<V> {
		context
			.value_stack_mut()
			.pop_as::<T>()
			.map(|v| v.extend_into())
			.map(|v| v.transmute_into())
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_reinterpret<T, U>(context: &mut FunctionContext) -> Result<InstructionOutcome, Error>
		where RuntimeValue: From<U>, RuntimeValue: TryInto<T, Error>, T: TransmuteInto<U> {
		context
			.value_stack_mut()
			.pop_as::<T>()
			.map(TransmuteInto::transmute_into)
			.and_then(|val| context.value_stack_mut().push(val.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn execute_block(context: &mut FunctionContext, body: &[Opcode]) -> Result<InstructionOutcome, Error> {
		debug_assert!(!context.frame_stack.is_empty());

		// run instructions
		context.position = 0;
		loop {
			let instruction = &body[context.position];

			//println!("=== RUNNING {:?}", instruction); // TODO: trace
			match Interpreter::run_instruction(context, instruction)? {
				InstructionOutcome::RunInstruction => (),
				InstructionOutcome::RunNextInstruction => context.position += 1,
				InstructionOutcome::Branch(index) => {
					if index != 0 {
						context.discard_frame()?;
						return Ok(InstructionOutcome::Branch(index - 1));
					} else {
						context.pop_frame(true)?;
						return Ok(InstructionOutcome::RunInstruction);
					}
				},
				InstructionOutcome::End => {
					context.pop_frame(false)?;
					return Ok(InstructionOutcome::RunInstruction);
				},
				InstructionOutcome::Return => return Ok(InstructionOutcome::Return),
			}
		}
	}
}

impl<'a> FunctionContext<'a> {
	pub fn new(module: &'a ModuleInstance, value_stack_limit: usize, frame_stack_limit: usize, function: &FunctionType, body: &[Opcode], args: Vec<VariableInstance>) -> Result<Self, Error> {
		let mut context = FunctionContext {
			module: module,
			return_type: function.return_type().map(|vt| BlockType::Value(vt)).unwrap_or(BlockType::NoResult),
			value_stack: StackWithLimit::with_limit(value_stack_limit),
			frame_stack: StackWithLimit::with_limit(frame_stack_limit),
			locals: args,
			position: 0,
		};
		context.push_frame(body.len() - 1, body.len() - 1, match function.return_type() {
			Some(value_type) => BlockType::Value(value_type),
			None => BlockType::NoResult,
		})?;
		Ok(context)
	}

	pub fn module(&self) -> &ModuleInstance {
		self.module
	}

	pub fn call_function(&mut self, index: u32) -> Result<Option<RuntimeValue>, Error> {
		self.module.call_function(CallerContext::nested(self), ItemIndex::IndexSpace(index))
	}

	pub fn call_function_indirect(&mut self, table_index: u32, type_index: u32, func_index: u32) -> Result<Option<RuntimeValue>, Error> {
		self.module.call_function_indirect(CallerContext::nested(self), ItemIndex::IndexSpace(table_index), type_index, func_index)
	}

	pub fn set_local(&mut self, index: usize, value: RuntimeValue) -> Result<InstructionOutcome, Error> {
		self.locals.get_mut(index)
			.ok_or(Error::Local(format!("expected to have local with index {}", index)))
			.and_then(|l| l.set(value))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	pub fn get_local(&mut self, index: usize) -> Result<RuntimeValue, Error> {
		self.locals.get(index)
			.ok_or(Error::Local(format!("expected to have local with index {}", index)))
			.map(|l| l.get())
	}

	pub fn value_stack(&self) -> &StackWithLimit<RuntimeValue> {
		&self.value_stack
	}

	pub fn value_stack_mut(&mut self) -> &mut StackWithLimit<RuntimeValue> {
		&mut self.value_stack
	}

	pub fn frame_stack(&self) -> &StackWithLimit<BlockFrame> {
		&self.frame_stack
	}

	pub fn push_frame(&mut self, branch_position: usize, end_position: usize, signature: BlockType) -> Result<(), Error> {
		self.frame_stack.push(BlockFrame {
			branch_position: branch_position,
			end_position: end_position,
			value_limit: self.value_stack.len(),
			signature: signature,
		})
	}

	pub fn discard_frame(&mut self) -> Result<(), Error> {
		self.frame_stack.pop()
			.map(|_| ())
	}

	pub fn pop_frame(&mut self, is_branch: bool) -> Result<(), Error> {
		let frame = self.frame_stack.pop()?;
		if frame.value_limit > self.value_stack.len() {
			return Err(Error::Stack("invalid stack len".into()));
		}

		let frame_value = match frame.signature {
			BlockType::Value(_) => Some(self.value_stack.pop()?),
			BlockType::NoResult => None,
		};
		self.value_stack.resize(frame.value_limit, RuntimeValue::I32(0));
		self.position = if is_branch { frame.branch_position } else { frame.end_position };
		if let Some(frame_value) = frame_value {
			self.value_stack.push(frame_value)?;
		}

		Ok(())
	}
}

impl BlockFrame {
	pub fn invalid() -> Self {
		BlockFrame {
			branch_position: usize::max_value(),
			end_position: usize::max_value(),
			value_limit: usize::max_value(),
			signature: BlockType::NoResult,
		}
	}
}

fn effective_address(address: u32, offset: u32) -> Result<u32, Error> {
	match offset.checked_add(address) {
		None => Err(Error::Memory(format!("invalid memory access: {} + {}", offset, address))),
		Some(address) => Ok(address),
	}
}
