use std::mem;
use std::ops;
use std::{u32, usize};
use std::sync::Arc;
use std::fmt::{self, Display};
use std::iter::repeat;
use std::collections::{HashMap, VecDeque};
use elements::{Opcode, BlockType, Local};
use interpreter::Error;
use interpreter::module::{ModuleInstanceInterface, CallerContext, ItemIndex, InternalFunctionReference, FunctionSignature};
use interpreter::value::{
	RuntimeValue, TryInto, WrapInto, TryTruncateInto, ExtendInto,
	ArithmeticOps, Integer, Float, LittleEndianConvert, TransmuteInto,
};
use interpreter::variable::VariableInstance;
use common::{DEFAULT_MEMORY_INDEX, DEFAULT_TABLE_INDEX, BlockFrame, BlockFrameType};
use common::stack::StackWithLimit;

/// Function interpreter.
pub struct Interpreter;

/// Function execution context.
pub struct FunctionContext<'a> {
	/// Is context initialized.
	pub is_initialized: bool,
	/// Internal function reference.
	pub function: InternalFunctionReference<'a>,
	/// Execution-local external modules.
	pub externals: &'a HashMap<String, Arc<ModuleInstanceInterface + 'a>>,
	/// Function return type.
	pub return_type: BlockType,
	/// Local variables.
	pub locals: Vec<VariableInstance>,
	/// Values stack.
	pub value_stack: StackWithLimit<RuntimeValue>,
	/// Blocks frames stack.
	pub frame_stack: StackWithLimit<BlockFrame>,
	/// Current instruction position.
	pub position: usize,
}

/// Interpreter action to execute after executing instruction.
#[derive(Debug)]
pub enum InstructionOutcome<'a> {
	/// Continue with next instruction.
	RunNextInstruction,
	/// Branch to given frame.
	Branch(usize),
	/// Execute function call.
	ExecuteCall(InternalFunctionReference<'a>),
	/// End current frame.
	End,
	/// Return from current function block.
	Return,
}

/// Function run result.
enum RunResult<'a> {
	/// Function has returned (optional) value.
	Return(Option<RuntimeValue>),
	/// Function is calling other function.
	NestedCall(FunctionContext<'a>),
}

impl Interpreter {
	pub fn run_function(function_context: FunctionContext) -> Result<Option<RuntimeValue>, Error> {
		let mut function_stack = VecDeque::new();
		function_stack.push_back(function_context);

		loop {
			let mut function_context = function_stack.pop_back().expect("on loop entry - not empty; on loop continue - checking for emptiness; qed");
			let function_ref = function_context.function.clone();
			let function_return = {
				let function_body = function_ref.module.function_body(function_ref.internal_index)?;

				match function_body {
					Some(function_body) => {
						if !function_context.is_initialized() {
							let return_type = function_context.return_type;
							function_context.initialize(function_body.locals)?;
							function_context.push_frame(function_body.labels, BlockFrameType::Function, return_type)?;
						}

						Interpreter::do_run_function(&mut function_context, function_body.body, function_body.labels)?
					},
					None => {
						// move locals back to the stack
						let locals_to_move: Vec<_> = function_context.locals.drain(..).collect();
						for local in locals_to_move {
							function_context.value_stack_mut().push(local.get())?;
						}

						let nested_context = CallerContext::nested(&mut function_context);
						RunResult::Return(function_ref.module.call_internal_function(nested_context, function_ref.internal_index)?)
					},
				}
			};

			match function_return {
				RunResult::Return(return_value) => {
					match function_stack.back_mut() {
						Some(caller_context) => if let Some(return_value) = return_value {
							caller_context.value_stack_mut().push(return_value)?;
						},
						None => return Ok(return_value),
					}
				},
				RunResult::NestedCall(nested_context) => {
					function_stack.push_back(function_context);
					function_stack.push_back(nested_context);
				},
			}
		}
	}

	fn do_run_function<'a>(function_context: &mut FunctionContext<'a>, function_body: &[Opcode], function_labels: &HashMap<usize, usize>) -> Result<RunResult<'a>, Error> {
		loop {
			let instruction = &function_body[function_context.position];

			debug!(target: "interpreter", "running {:?}", instruction);
			match Interpreter::run_instruction(function_context, function_labels, instruction)? {
				InstructionOutcome::RunNextInstruction => function_context.position += 1,
				InstructionOutcome::Branch(mut index) => {
					// discard index - 1 blocks
					while index >= 1 {
						function_context.discard_frame()?;
						index -= 1;
					}

					function_context.pop_frame(true)?;
					if function_context.frame_stack().is_empty() {
						break;
					}
				},
				InstructionOutcome::ExecuteCall(func_ref) => {
					function_context.position += 1;
					return Ok(RunResult::NestedCall(function_context.nested(func_ref)?));
				},
				InstructionOutcome::End => {
					if function_context.frame_stack().is_empty() {
						break;
					}
				},
				InstructionOutcome::Return => break,
			}
		}

		Ok(RunResult::Return(match function_context.return_type {
			BlockType::Value(_) => Some(function_context.value_stack_mut().pop()?),
			BlockType::NoResult => None,
		}))
	}

	fn run_instruction<'a>(context: &mut FunctionContext<'a>, labels: &HashMap<usize, usize>, opcode: &Opcode) -> Result<InstructionOutcome<'a>, Error> {
		match opcode {
			&Opcode::Unreachable => Interpreter::run_unreachable(context),
			&Opcode::Nop => Interpreter::run_nop(context),
			&Opcode::Block(block_type) => Interpreter::run_block(context, labels, block_type),
			&Opcode::Loop(block_type) => Interpreter::run_loop(context, labels, block_type),
			&Opcode::If(block_type) => Interpreter::run_if(context, labels, block_type),
			&Opcode::Else => Interpreter::run_else(context, labels),
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

			&Opcode::I32Load(align, offset) => Interpreter::run_load::<i32>(context, align, offset),
			&Opcode::I64Load(align, offset) => Interpreter::run_load::<i64>(context, align, offset),
			&Opcode::F32Load(align, offset) => Interpreter::run_load::<f32>(context, align, offset),
			&Opcode::F64Load(align, offset) => Interpreter::run_load::<f64>(context, align, offset),
			&Opcode::I32Load8S(align, offset) => Interpreter::run_load_extend::<i8, i32>(context, align, offset),
			&Opcode::I32Load8U(align, offset) => Interpreter::run_load_extend::<u8, i32>(context, align, offset),
			&Opcode::I32Load16S(align, offset) => Interpreter::run_load_extend::<i16, i32>(context, align, offset),
			&Opcode::I32Load16U(align, offset) => Interpreter::run_load_extend::<u16, i32>(context, align, offset),
			&Opcode::I64Load8S(align, offset) => Interpreter::run_load_extend::<i8, i64>(context, align, offset),
			&Opcode::I64Load8U(align, offset) => Interpreter::run_load_extend::<u8, i64>(context, align, offset),
			&Opcode::I64Load16S(align, offset) => Interpreter::run_load_extend::<i16, i64>(context, align, offset),
			&Opcode::I64Load16U(align, offset) => Interpreter::run_load_extend::<u16, i64>(context, align, offset),
			&Opcode::I64Load32S(align, offset) => Interpreter::run_load_extend::<i32, i64>(context, align, offset),
			&Opcode::I64Load32U(align, offset) => Interpreter::run_load_extend::<u32, i64>(context, align, offset),

			&Opcode::I32Store(align, offset) => Interpreter::run_store::<i32>(context, align, offset),
			&Opcode::I64Store(align, offset) => Interpreter::run_store::<i64>(context, align, offset),
			&Opcode::F32Store(align, offset) => Interpreter::run_store::<f32>(context, align, offset),
			&Opcode::F64Store(align, offset) => Interpreter::run_store::<f64>(context, align, offset),
			&Opcode::I32Store8(align, offset) => Interpreter::run_store_wrap::<i32, i8>(context, align, offset),
			&Opcode::I32Store16(align, offset) => Interpreter::run_store_wrap::<i32, i16>(context, align, offset),
			&Opcode::I64Store8(align, offset) => Interpreter::run_store_wrap::<i64, i8>(context, align, offset),
			&Opcode::I64Store16(align, offset) => Interpreter::run_store_wrap::<i64, i16>(context, align, offset),
			&Opcode::I64Store32(align, offset) => Interpreter::run_store_wrap::<i64, i32>(context, align, offset),

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
			&Opcode::I32Shl => Interpreter::run_shl::<i32>(context, 0x1F),
			&Opcode::I32ShrS => Interpreter::run_shr::<i32, i32>(context, 0x1F),
			&Opcode::I32ShrU => Interpreter::run_shr::<i32, u32>(context, 0x1F),
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
			&Opcode::I64Shl => Interpreter::run_shl::<i64>(context, 0x3F),
			&Opcode::I64ShrS => Interpreter::run_shr::<i64, i64>(context, 0x3F),
			&Opcode::I64ShrU => Interpreter::run_shr::<i64, u64>(context, 0x3F),
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

	fn run_unreachable<'a>(_context: &mut FunctionContext) -> Result<InstructionOutcome<'a>, Error> {
		Err(Error::Trap("programmatic".into()))
	}

	fn run_nop<'a>(_context: &mut FunctionContext) -> Result<InstructionOutcome<'a>, Error> {
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn run_block<'a>(context: &mut FunctionContext<'a>, labels: &HashMap<usize, usize>, block_type: BlockType) -> Result<InstructionOutcome<'a>, Error> {
		context.push_frame(labels, BlockFrameType::Block, block_type)?;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn run_loop<'a>(context: &mut FunctionContext<'a>, labels: &HashMap<usize, usize>, block_type: BlockType) -> Result<InstructionOutcome<'a>, Error> {
		context.push_frame(labels, BlockFrameType::Loop, block_type)?;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn run_if<'a>(context: &mut FunctionContext<'a>, labels: &HashMap<usize, usize>, block_type: BlockType) -> Result<InstructionOutcome<'a>, Error> {
		let branch = context.value_stack_mut().pop_as()?;
		let block_frame_type = if branch { BlockFrameType::IfTrue } else {
			let else_pos = labels[&context.position];
			if !labels.contains_key(&else_pos) {
				context.position = else_pos;
				return Ok(InstructionOutcome::RunNextInstruction);
			}

			context.position = else_pos;
			BlockFrameType::IfFalse
		};
		context.push_frame(labels, block_frame_type, block_type).map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_else<'a>(context: &mut FunctionContext, labels: &HashMap<usize, usize>) -> Result<InstructionOutcome<'a>, Error> {
		let end_pos = labels[&context.position];
		context.pop_frame(false)?;
		context.position = end_pos;
		Ok(InstructionOutcome::RunNextInstruction)
	}

	fn run_end<'a>(context: &mut FunctionContext) -> Result<InstructionOutcome<'a>, Error> {
		context.pop_frame(false)?;
		Ok(InstructionOutcome::End)
	}

	fn run_br<'a>(_context: &mut FunctionContext, label_idx: u32) -> Result<InstructionOutcome<'a>, Error> {
		Ok(InstructionOutcome::Branch(label_idx as usize))
	}

	fn run_br_if<'a>(context: &mut FunctionContext, label_idx: u32) -> Result<InstructionOutcome<'a>, Error> {
		if context.value_stack_mut().pop_as()? {
			Ok(InstructionOutcome::Branch(label_idx as usize))
		} else {
			Ok(InstructionOutcome::RunNextInstruction)
		}
	}

	fn run_br_table<'a>(context: &mut FunctionContext, table: &Vec<u32>, default: u32) -> Result<InstructionOutcome<'a>, Error> {
		let index: u32 = context.value_stack_mut().pop_as()?;
		Ok(InstructionOutcome::Branch(table.get(index as usize).cloned().unwrap_or(default) as usize))
	}

	fn run_return<'a>(_context: &mut FunctionContext) -> Result<InstructionOutcome<'a>, Error> {
		Ok(InstructionOutcome::Return)
	}

	fn run_call<'a>(context: &mut FunctionContext<'a>, func_idx: u32) -> Result<InstructionOutcome<'a>, Error> {
		Ok(InstructionOutcome::ExecuteCall(context.module().function_reference(ItemIndex::IndexSpace(func_idx), Some(context.externals))?))
	}

	fn run_call_indirect<'a>(context: &mut FunctionContext<'a>, type_idx: u32) -> Result<InstructionOutcome<'a>, Error> {
		let table_func_idx: u32 = context.value_stack_mut().pop_as()?;
		let function_reference = context.module().function_reference_indirect(DEFAULT_TABLE_INDEX, type_idx, table_func_idx, Some(context.externals))?;
		{
			let required_function_type = context.module().function_type_by_index(type_idx)?;
			let actual_function_type = function_reference.module.function_type(ItemIndex::Internal(function_reference.internal_index))?;
			if required_function_type != actual_function_type {
				return Err(Error::Function(format!("expected function with signature ({:?}) -> {:?} when got with ({:?}) -> {:?}",
					required_function_type.params(), required_function_type.return_type(),
					actual_function_type.params(), actual_function_type.return_type())));
			}
		}
		Ok(InstructionOutcome::ExecuteCall(function_reference))
	}

	fn run_drop<'a>(context: &mut FunctionContext) -> Result<InstructionOutcome<'a>, Error> {
		context
			.value_stack_mut()
			.pop()
			.map_err(Into::into)
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_select<'a>(context: &mut FunctionContext) -> Result<InstructionOutcome<'a>, Error> {
		context
			.value_stack_mut()
			.pop_triple()
			.and_then(|(left, mid, right)| {
				let right: Result<_, Error> = right.try_into();
				match (left, mid, right) {
					(left, mid, Ok(condition)) => Ok((left, mid, condition)),
					_ => Err(Error::Stack("expected to get int value from stack".into()))
				}
			})
			.map(|(left, mid, condition)| if condition { left } else { mid })
			.map(|val| context.value_stack_mut().push(val))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_get_local<'a>(context: &mut FunctionContext, index: u32) -> Result<InstructionOutcome<'a>, Error> {
		context.get_local(index as usize)
			.map(|value| context.value_stack_mut().push(value))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_set_local<'a>(context: &mut FunctionContext, index: u32) -> Result<InstructionOutcome<'a>, Error> {
		let arg = context.value_stack_mut().pop()?;
		context.set_local(index as usize, arg)
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_tee_local<'a>(context: &mut FunctionContext, index: u32) -> Result<InstructionOutcome<'a>, Error> {
		let arg = context.value_stack().top()?.clone();
		context.set_local(index as usize, arg)
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_get_global<'a>(context: &mut FunctionContext, index: u32) -> Result<InstructionOutcome<'a>, Error> {
		context.module()
			.global(ItemIndex::IndexSpace(index), None, Some(context.externals))
			.and_then(|g| context.value_stack_mut().push(g.get()).map_err(Into::into))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_set_global<'a>(context: &mut FunctionContext, index: u32) -> Result<InstructionOutcome<'a>, Error> {
		context
			.value_stack_mut()
			.pop()
			.map_err(Into::into)
			.and_then(|v| context.module().global(ItemIndex::IndexSpace(index), None, Some(context.externals)).and_then(|g| g.set(v)))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_load<'a, T>(context: &mut FunctionContext, _align: u32, offset: u32) -> Result<InstructionOutcome<'a>, Error>
		where RuntimeValue: From<T>, T: LittleEndianConvert {
		let address = effective_address(offset, context.value_stack_mut().pop_as()?)?;
		context.module()
			.memory(ItemIndex::IndexSpace(DEFAULT_MEMORY_INDEX))
			.and_then(|m| m.get(address, mem::size_of::<T>()))
			.and_then(|b| T::from_little_endian(b))
			.and_then(|n| context.value_stack_mut().push(n.into()).map_err(Into::into))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_load_extend<'a, T, U>(context: &mut FunctionContext, _align: u32, offset: u32) -> Result<InstructionOutcome<'a>, Error>
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
			.map_err(Into::into)
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_store<'a, T>(context: &mut FunctionContext, _align: u32, offset: u32) -> Result<InstructionOutcome<'a>, Error>
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

	fn run_store_wrap<'a, T, U>(
		context: &mut FunctionContext,
		_align: u32,
		offset: u32,
	) -> Result<InstructionOutcome<'a>, Error>
	where
		RuntimeValue: TryInto<T, Error>,
		T: WrapInto<U>,
		U: LittleEndianConvert,
	{
		let stack_value: T = context
			.value_stack_mut()
			.pop()
			.map_err(Into::into)
			.and_then(|v| v.try_into())?;
		let stack_value = stack_value.wrap_into().into_little_endian();
		let address = effective_address(offset, context.value_stack_mut().pop_as::<u32>()?)?;
		context.module()
			.memory(ItemIndex::IndexSpace(DEFAULT_MEMORY_INDEX))
			.and_then(|m| m.set(address, &stack_value))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_current_memory<'a>(context: &mut FunctionContext) -> Result<InstructionOutcome<'a>, Error> {
		context
			.module()
			.memory(ItemIndex::IndexSpace(DEFAULT_MEMORY_INDEX))
			.map(|m| m.size())
			.and_then(|s| {
				context
					.value_stack_mut()
					.push(RuntimeValue::I32(s as i32))
					.map_err(Into::into)
			})
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_grow_memory<'a>(context: &mut FunctionContext) -> Result<InstructionOutcome<'a>, Error> {
		let pages: u32 = context.value_stack_mut().pop_as()?;
		context
			.module()
			.memory(ItemIndex::IndexSpace(DEFAULT_MEMORY_INDEX))
			.and_then(|m| m.grow(pages))
			.and_then(|m| {
				context
					.value_stack_mut()
					.push(RuntimeValue::I32(m as i32))
					.map_err(Into::into)
			})
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_const<'a>(context: &mut FunctionContext, val: RuntimeValue) -> Result<InstructionOutcome<'a>, Error> {
		context
			.value_stack_mut()
			.push(val)
			.map_err(Into::into)
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_eqz<'a, T>(context: &mut FunctionContext) -> Result<InstructionOutcome<'a>, Error>
		where RuntimeValue: TryInto<T, Error>, T: PartialEq<T> + Default {
		context
			.value_stack_mut()
			.pop_as::<T>()
			.map(|v| RuntimeValue::I32(if v == Default::default() { 1 } else { 0 }))
			.and_then(|v| context.value_stack_mut().push(v).map_err(Into::into))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_eq<'a, T>(context: &mut FunctionContext) -> Result<InstructionOutcome<'a>, Error>
		where RuntimeValue: TryInto<T, Error>, T: PartialEq<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| RuntimeValue::I32(if left == right { 1 } else { 0 }))
			.and_then(|v| context.value_stack_mut().push(v).map_err(Into::into))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_ne<'a, T>(context: &mut FunctionContext) -> Result<InstructionOutcome<'a>, Error>
		where RuntimeValue: TryInto<T, Error>, T: PartialEq<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| RuntimeValue::I32(if left != right { 1 } else { 0 }))
			.and_then(|v| context.value_stack_mut().push(v).map_err(Into::into))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_lt<'a, T>(context: &mut FunctionContext) -> Result<InstructionOutcome<'a>, Error>
		where RuntimeValue: TryInto<T, Error>, T: PartialOrd<T> + Display {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| RuntimeValue::I32(if left < right { 1 } else { 0 }))
			.and_then(|v| context.value_stack_mut().push(v).map_err(Into::into))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_gt<'a, T>(context: &mut FunctionContext) -> Result<InstructionOutcome<'a>, Error>
		where RuntimeValue: TryInto<T, Error>, T: PartialOrd<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| RuntimeValue::I32(if left > right { 1 } else { 0 }))
			.and_then(|v| context.value_stack_mut().push(v).map_err(Into::into))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_lte<'a, T>(context: &mut FunctionContext) -> Result<InstructionOutcome<'a>, Error>
		where RuntimeValue: TryInto<T, Error>, T: PartialOrd<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| RuntimeValue::I32(if left <= right { 1 } else { 0 }))
			.and_then(|v| context.value_stack_mut().push(v).map_err(Into::into))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_gte<'a, T>(context: &mut FunctionContext) -> Result<InstructionOutcome<'a>, Error>
		where RuntimeValue: TryInto<T, Error>, T: PartialOrd<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| RuntimeValue::I32(if left >= right { 1 } else { 0 }))
			.and_then(|v| context.value_stack_mut().push(v).map_err(Into::into))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_clz<'a, T>(context: &mut FunctionContext) -> Result<InstructionOutcome<'a>, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: Integer<T> {
		context
			.value_stack_mut()
			.pop_as::<T>()
			.map(|v| v.leading_zeros())
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_ctz<'a, T>(context: &mut FunctionContext) -> Result<InstructionOutcome<'a>, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: Integer<T> {
		context
			.value_stack_mut()
			.pop_as::<T>()
			.map(|v| v.trailing_zeros())
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_popcnt<'a, T>(context: &mut FunctionContext) -> Result<InstructionOutcome<'a>, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: Integer<T> {
		context
			.value_stack_mut()
			.pop_as::<T>()
			.map(|v| v.count_ones())
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_add<'a, T>(context: &mut FunctionContext) -> Result<InstructionOutcome<'a>, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: ArithmeticOps<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| left.add(right))
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_sub<'a, T>(context: &mut FunctionContext) -> Result<InstructionOutcome<'a>, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: ArithmeticOps<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| left.sub(right))
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_mul<'a, T>(context: &mut FunctionContext) -> Result<InstructionOutcome<'a>, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: ArithmeticOps<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| left.mul(right))
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_div<'a, T, U>(context: &mut FunctionContext) -> Result<InstructionOutcome<'a>, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: TransmuteInto<U> + Display, U: ArithmeticOps<U> + TransmuteInto<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| (left.transmute_into(), right.transmute_into()))
			.map(|(left, right)| left.div(right))?
			.map(|v| v.transmute_into())
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_rem<'a, T, U>(context: &mut FunctionContext) -> Result<InstructionOutcome<'a>, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: TransmuteInto<U>, U: Integer<U> + TransmuteInto<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| (left.transmute_into(), right.transmute_into()))
			.map(|(left, right)| left.rem(right))?
			.map(|v| v.transmute_into())
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_and<'a, T>(context: &mut FunctionContext) -> Result<InstructionOutcome<'a>, Error>
		where RuntimeValue: From<<T as ops::BitAnd>::Output> + TryInto<T, Error>, T: ops::BitAnd<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| left.bitand(right))
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_or<'a, T>(context: &mut FunctionContext) -> Result<InstructionOutcome<'a>, Error>
		where RuntimeValue: From<<T as ops::BitOr>::Output> + TryInto<T, Error>, T: ops::BitOr<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| left.bitor(right))
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_xor<'a, T>(context: &mut FunctionContext) -> Result<InstructionOutcome<'a>, Error>
		where RuntimeValue: From<<T as ops::BitXor>::Output> + TryInto<T, Error>, T: ops::BitXor<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| left.bitxor(right))
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_shl<'a, T>(context: &mut FunctionContext, mask: T) -> Result<InstructionOutcome<'a>, Error>
		where RuntimeValue: From<<T as ops::Shl<T>>::Output> + TryInto<T, Error>, T: ops::Shl<T> + ops::BitAnd<T, Output=T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| left.shl(right & mask))
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_shr<'a, T, U>(context: &mut FunctionContext, mask: U) -> Result<InstructionOutcome<'a>, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: TransmuteInto<U>, U: ops::Shr<U> + ops::BitAnd<U, Output=U>, <U as ops::Shr<U>>::Output: TransmuteInto<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| (left.transmute_into(), right.transmute_into()))
			.map(|(left, right)| left.shr(right & mask))
			.map(|v| v.transmute_into())
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_rotl<'a, T>(context: &mut FunctionContext) -> Result<InstructionOutcome<'a>, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: Integer<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| left.rotl(right))
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_rotr<'a, T>(context: &mut FunctionContext) -> Result<InstructionOutcome<'a>, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: Integer<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| left.rotr(right))
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_abs<'a, T>(context: &mut FunctionContext) -> Result<InstructionOutcome<'a>, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: Float<T> {
		context
			.value_stack_mut()
			.pop_as::<T>()
			.map(|v| v.abs())
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_neg<'a, T>(context: &mut FunctionContext) -> Result<InstructionOutcome<'a>, Error>
		where RuntimeValue: From<<T as ops::Neg>::Output> + TryInto<T, Error>, T: ops::Neg {
		context
			.value_stack_mut()
			.pop_as::<T>()
			.map(|v| v.neg())
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_ceil<'a, T>(context: &mut FunctionContext) -> Result<InstructionOutcome<'a>, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: Float<T> {
		context
			.value_stack_mut()
			.pop_as::<T>()
			.map(|v| v.ceil())
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_floor<'a, T>(context: &mut FunctionContext) -> Result<InstructionOutcome<'a>, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: Float<T> {
		context
			.value_stack_mut()
			.pop_as::<T>()
			.map(|v| v.floor())
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_trunc<'a, T>(context: &mut FunctionContext) -> Result<InstructionOutcome<'a>, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: Float<T> {
		context
			.value_stack_mut()
			.pop_as::<T>()
			.map(|v| v.trunc())
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_nearest<'a, T>(context: &mut FunctionContext) -> Result<InstructionOutcome<'a>, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: Float<T> {
		context
			.value_stack_mut()
			.pop_as::<T>()
			.map(|v| v.nearest())
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_sqrt<'a, T>(context: &mut FunctionContext) -> Result<InstructionOutcome<'a>, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: Float<T> {
		context
			.value_stack_mut()
			.pop_as::<T>()
			.map(|v| v.sqrt())
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_min<'a, T>(context: &mut FunctionContext) -> Result<InstructionOutcome<'a>, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: Float<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| left.min(right))
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_max<'a, T>(context: &mut FunctionContext) -> Result<InstructionOutcome<'a>, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: Float<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| left.max(right))
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_copysign<'a, T>(context: &mut FunctionContext) -> Result<InstructionOutcome<'a>, Error>
		where RuntimeValue: From<T> + TryInto<T, Error>, T: Float<T> {
		context
			.value_stack_mut()
			.pop_pair_as::<T>()
			.map(|(left, right)| left.copysign(right))
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_wrap<'a, T, U>(context: &mut FunctionContext) -> Result<InstructionOutcome<'a>, Error>
		where RuntimeValue: From<U> + TryInto<T, Error>, T: WrapInto<U> {
		context
			.value_stack_mut()
			.pop_as::<T>()
			.map(|v| v.wrap_into())
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_trunc_to_int<'a, T, U, V>(context: &mut FunctionContext) -> Result<InstructionOutcome<'a>, Error>
		where RuntimeValue: From<V> + TryInto<T, Error>, T: TryTruncateInto<U, Error>, U: TransmuteInto<V>,  {
		context
			.value_stack_mut()
			.pop_as::<T>()
			.and_then(|v| v.try_truncate_into())
			.map(|v| v.transmute_into())
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_extend<'a, T, U, V>(
		context: &mut FunctionContext,
	) -> Result<InstructionOutcome<'a>, Error>
	where
		RuntimeValue: From<V> + TryInto<T, Error>,
		T: ExtendInto<U>,
		U: TransmuteInto<V>,
	{
		context
			.value_stack_mut()
			.pop_as::<T>()
			.map_err(Error::into)
			.map(|v| v.extend_into())
			.map(|v| v.transmute_into())
			.map(|v| context.value_stack_mut().push(v.into()))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}

	fn run_reinterpret<'a, T, U>(context: &mut FunctionContext) -> Result<InstructionOutcome<'a>, Error>
		where RuntimeValue: From<U>, RuntimeValue: TryInto<T, Error>, T: TransmuteInto<U> {
		context
			.value_stack_mut()
			.pop_as::<T>()
			.map(TransmuteInto::transmute_into)
			.and_then(|val| context.value_stack_mut().push(val.into()).map_err(Into::into))
			.map(|_| InstructionOutcome::RunNextInstruction)
	}
}

impl<'a> FunctionContext<'a> {
	pub fn new(function: InternalFunctionReference<'a>, externals: &'a HashMap<String, Arc<ModuleInstanceInterface + 'a>>, value_stack_limit: usize, frame_stack_limit: usize, function_type: &FunctionSignature, args: Vec<VariableInstance>) -> Self {
		FunctionContext {
			is_initialized: false,
			function: function,
			externals: externals,
			return_type: function_type.return_type().map(|vt| BlockType::Value(vt)).unwrap_or(BlockType::NoResult),
			value_stack: StackWithLimit::with_limit(value_stack_limit),
			frame_stack: StackWithLimit::with_limit(frame_stack_limit),
			locals: args,
			position: 0,
		}
	}

	pub fn nested(&mut self, function: InternalFunctionReference<'a>) -> Result<Self, Error> {
		let (function_locals, function_return_type) = {
			let function_type = function.module.function_type(ItemIndex::Internal(function.internal_index))?;
			let function_return_type = function_type.return_type().map(|vt| BlockType::Value(vt)).unwrap_or(BlockType::NoResult);
			let function_locals = prepare_function_args(&function_type, &mut self.value_stack)?;
			(function_locals, function_return_type)
		};

		Ok(FunctionContext {
			is_initialized: false,
			function: function,
			externals: self.externals,
			return_type: function_return_type,
			value_stack: StackWithLimit::with_limit(self.value_stack.limit() - self.value_stack.len()),
			frame_stack: StackWithLimit::with_limit(self.frame_stack.limit() - self.frame_stack.len()),
			locals: function_locals,
			position: 0,
		})
	}

	pub fn is_initialized(&self) -> bool {
		self.is_initialized
	}

	pub fn initialize(&mut self, locals: &[Local]) -> Result<(), Error> {
		debug_assert!(!self.is_initialized);
		self.is_initialized = true;

		let locals = locals.iter()
			.flat_map(|l| repeat(l.value_type().into()).take(l.count() as usize))
			.map(|vt| VariableInstance::new(true, vt, RuntimeValue::default(vt)))
			.collect::<Result<Vec<_>, _>>()?;
		self.locals.extend(locals);
		Ok(())
	}

	pub fn module(&self) -> &Arc<ModuleInstanceInterface + 'a> {
		&self.function.module
	}

	pub fn externals(&self) -> &HashMap<String, Arc<ModuleInstanceInterface + 'a>> {
		&self.externals
	}

	pub fn set_local(&mut self, index: usize, value: RuntimeValue) -> Result<InstructionOutcome<'a>, Error> {
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

	pub fn frame_stack_mut(&mut self) -> &mut StackWithLimit<BlockFrame> {
		&mut self.frame_stack
	}

	pub fn push_frame(&mut self, labels: &HashMap<usize, usize>, frame_type: BlockFrameType, block_type: BlockType) -> Result<(), Error> {
		let begin_position = self.position;
		let branch_position = match frame_type {
			BlockFrameType::Function => usize::MAX,
			BlockFrameType::Loop => begin_position,
			BlockFrameType::IfTrue => {
				let else_pos = labels[&begin_position];
				1usize + match labels.get(&else_pos) {
					Some(end_pos) => *end_pos,
					None => else_pos,
				}
			},
			_ => labels[&begin_position] + 1,
		};
		let end_position = match frame_type {
			BlockFrameType::Function => usize::MAX,
			_ => labels[&begin_position] + 1,
		};
		Ok(self.frame_stack.push(BlockFrame {
			frame_type: frame_type,
			block_type: block_type,
			begin_position: begin_position,
			branch_position: branch_position,
			end_position: end_position,
			value_stack_len: self.value_stack.len(),
		})?)
	}

	pub fn discard_frame(&mut self) -> Result<(), Error> {
		Ok(self.frame_stack.pop().map(|_| ())?)
	}

	pub fn pop_frame(&mut self, is_branch: bool) -> Result<(), Error> {
		let frame = self.frame_stack.pop()?;
		if frame.value_stack_len > self.value_stack.len() {
			return Err(Error::Stack("invalid stack len".into()));
		}

		let frame_value = match frame.block_type {
			BlockType::Value(_) if frame.frame_type != BlockFrameType::Loop || !is_branch => Some(self.value_stack.pop()?),
			_ => None,
		};
		self.value_stack.resize(frame.value_stack_len, RuntimeValue::I32(0));
		self.position = if is_branch { frame.branch_position } else { frame.end_position };
		if let Some(frame_value) = frame_value {
			self.value_stack.push(frame_value)?;
		}

		Ok(())
	}
}

impl<'a> fmt::Debug for FunctionContext<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "FunctionContext")
	}
}

fn effective_address(address: u32, offset: u32) -> Result<u32, Error> {
	match offset.checked_add(address) {
		None => Err(Error::Memory(format!("invalid memory access: {} + {}", offset, address))),
		Some(address) => Ok(address),
	}
}

pub fn prepare_function_args(function_type: &FunctionSignature, caller_stack: &mut StackWithLimit<RuntimeValue>) -> Result<Vec<VariableInstance>, Error> {
	let mut args = function_type.params().iter().rev().map(|param_type| {
		let param_value = caller_stack.pop()?;
		let actual_type = param_value.variable_type();
		let expected_type = (*param_type).into();
		if actual_type != Some(expected_type) {
			return Err(Error::Function(format!("invalid parameter type {:?} when expected {:?}", actual_type, expected_type)));
		}

		VariableInstance::new(true, expected_type, param_value)
	}).collect::<Result<Vec<_>, _>>()?;
	args.reverse();
	Ok(args)
}
