///! Tests from https://github.com/WebAssembly/wabt/tree/8e1f6031e9889ba770c7be4a9b084da5f14456a0/test/interp

use std::sync::Weak;
use builder::module;
use elements::{Module, ValueType, Opcodes, Opcode, BlockType, FunctionType};
use interpreter::Error;
use interpreter::module::{ModuleInstance, ItemIndex};
use interpreter::program::ProgramInstance;
use interpreter::runner::{Interpreter, FunctionContext};
use interpreter::value::{RuntimeValue, TryInto};
use interpreter::variable::{VariableInstance, VariableType};

fn run_function_i32(body: &Opcodes, arg: i32) -> Result<i32, Error> {
	let ftype = FunctionType::new(vec![ValueType::I32], Some(ValueType::I32));
	let module = ModuleInstance::new(Weak::default(), Module::default()).unwrap();
	let mut context = FunctionContext::new(&module, 1024, 1024, &ftype, body.elements(), vec![
			VariableInstance::new(true, VariableType::I32, RuntimeValue::I32(arg)).unwrap(),	// arg
			VariableInstance::new(true, VariableType::I32, RuntimeValue::I32(0)).unwrap(),		// local1
			VariableInstance::new(true, VariableType::I32, RuntimeValue::I32(0)).unwrap(),		// local2
		])?;
	Interpreter::run_function(&mut context, body.elements())
		.map(|v| v.unwrap().try_into().unwrap())
}

/// https://github.com/WebAssembly/wabt/blob/8e1f6031e9889ba770c7be4a9b084da5f14456a0/test/interp/unreachable.txt
#[test]
fn unreachable() {
	let body = Opcodes::new(vec![
		Opcode::Unreachable,	// trap
		Opcode::End]);

	assert_eq!(run_function_i32(&body, 0).unwrap_err(), Error::Trap("programmatic".into()));
}

#[test]
fn nop() {
	let body = Opcodes::new(vec![
		Opcode::Nop,			// nop
		Opcode::I32Const(1),	// [1]
		Opcode::Nop,			// nop
		Opcode::End]);

	assert_eq!(run_function_i32(&body, 0).unwrap(), 1);
}

/// https://github.com/WebAssembly/wabt/blob/8e1f6031e9889ba770c7be4a9b084da5f14456a0/test/interp/expr-block.txt
#[test]
fn expr_block() {
	let body = Opcodes::new(vec![
		Opcode::Block(BlockType::Value(ValueType::I32),	// mark block
			Opcodes::new(vec![
				Opcode::I32Const(10),		// [10]
				Opcode::Drop,
				Opcode::I32Const(1),		// [1]
				Opcode::End,
			])),
		Opcode::End]);

	assert_eq!(run_function_i32(&body, 0).unwrap(), 1);
}

/// https://github.com/WebAssembly/wabt/blob/8e1f6031e9889ba770c7be4a9b084da5f14456a0/test/interp/loop.txt
#[test]
fn loop_test() {
	let body = Opcodes::new(vec![
		Opcode::Loop(BlockType::NoResult,	// loop
			Opcodes::new(vec![
				Opcode::GetLocal(1),		//   [local1]
				Opcode::GetLocal(0),		//   [local1, arg]
				Opcode::I32Add,				//   [arg + local1]
				Opcode::SetLocal(1),		//   [] + local1 = arg + local1
				Opcode::GetLocal(0),		//   [arg]
				Opcode::I32Const(1),		//   [arg, 1]
				Opcode::I32Add,				//   [arg + 1]
				Opcode::SetLocal(0),		//   [] + arg = arg + 1
				Opcode::GetLocal(0),		//   [arg]
				Opcode::I32Const(5),		//   [arg, 5]
				Opcode::I32LtS,				//   [arg < 5]
				Opcode::If(BlockType::NoResult,
					Opcodes::new(vec![
						Opcode::Br(1),		//   break loop
						Opcode::End,
					])),
				Opcode::End])),				// end loop
		Opcode::GetLocal(1),				// [local1]
		Opcode::End]);

	assert_eq!(run_function_i32(&body, 0).unwrap(), 10);
}

/// https://github.com/WebAssembly/wabt/blob/8e1f6031e9889ba770c7be4a9b084da5f14456a0/test/interp/if.txt#L3
#[test]
fn if_1() {
	let body = Opcodes::new(vec![
		Opcode::I32Const(0),				// [0]
		Opcode::SetLocal(0),				// [] + arg = 0
		Opcode::I32Const(1),				// [1]
		Opcode::If(BlockType::NoResult,		// if 1
			Opcodes::new(vec![
				Opcode::GetLocal(0),		//   [arg]
				Opcode::I32Const(1),		//   [arg, 1]
				Opcode::I32Add,				//   [arg + 1]
				Opcode::SetLocal(0),		//   [] + arg = arg + 1
				Opcode::End,				// end if
			])),
		Opcode::I32Const(0),				// [0]
		Opcode::If(BlockType::NoResult,		// if 0
			Opcodes::new(vec![
				Opcode::GetLocal(0),		//   [arg]
				Opcode::I32Const(1),		//   [arg, 1]
				Opcode::I32Add,				//   [arg + 1]
				Opcode::SetLocal(0),		//   [] + arg = arg + 1
				Opcode::End,				// end if
			])),
		Opcode::GetLocal(0),				// [arg]
		Opcode::End]);

	assert_eq!(run_function_i32(&body, 0).unwrap(), 1);
}

/// https://github.com/WebAssembly/wabt/blob/8e1f6031e9889ba770c7be4a9b084da5f14456a0/test/interp/if.txt#L23
#[test]
fn if_2() {
	let body = Opcodes::new(vec![
		Opcode::I32Const(1),				// [1]
		Opcode::If(BlockType::NoResult,		// if 1
			Opcodes::new(vec![
				Opcode::I32Const(1),		//   [1]
				Opcode::SetLocal(0),		//   [] + arg = 1
				Opcode::Else,				// else
				Opcode::I32Const(2),		//   [2]
				Opcode::SetLocal(0),		//   [] + arg = 2
				Opcode::End,				// end if
			])),
		Opcode::I32Const(0),				// [0]
		Opcode::If(BlockType::NoResult,		// if 0
			Opcodes::new(vec![
				Opcode::I32Const(4),		//   [4]
				Opcode::SetLocal(1),		//   [] + local1 = 4
				Opcode::Else,				// else
				Opcode::I32Const(8),		//   [8]
				Opcode::SetLocal(1),		//   [] + local1 = 8
				Opcode::End,				// end if
			])),
		Opcode::GetLocal(0),				// [arg]
		Opcode::GetLocal(1),				// [arg, local1]
		Opcode::I32Add,						// [arg + local1]
		Opcode::End]);

	assert_eq!(run_function_i32(&body, 0).unwrap(), 9);
}

/// https://github.com/WebAssembly/wabt/blob/8e1f6031e9889ba770c7be4a9b084da5f14456a0/test/interp/expr-if.txt
#[test]
fn expr_if() {
	let body = Opcodes::new(vec![
		Opcode::GetLocal(0),							// [arg]
		Opcode::I32Const(0),							// [arg, 0]
		Opcode::I32Eq,									// [arg == 0]
		Opcode::If(BlockType::Value(ValueType::I32),	// if arg == 0
			Opcodes::new(vec![
				Opcode::I32Const(1),					//   [1]
				Opcode::Else,							// else
				Opcode::I32Const(2),					//   [2]
				Opcode::End,							// end if
			])),
		Opcode::End]);

	assert_eq!(run_function_i32(&body, 0).unwrap(), 1);
	assert_eq!(run_function_i32(&body, 1).unwrap(), 2);
}

/// https://github.com/WebAssembly/wabt/blob/8e1f6031e9889ba770c7be4a9b084da5f14456a0/test/interp/br.txt#L4
#[test]
fn br_0() {
	let body = Opcodes::new(vec![
		Opcode::Block(BlockType::NoResult,			// mark block
			Opcodes::new(vec![
				Opcode::I32Const(1),				//   [1]
				Opcode::If(BlockType::NoResult,		//   if 1
					Opcodes::new(vec![
						Opcode::Br(1),				//     break from block
						Opcode::End,				//   end if
					])),
				Opcode::I32Const(1),				//   [1]
				Opcode::SetLocal(0),				//   [] + arg = 1
				Opcode::End,						// end block
			])),
		Opcode::I32Const(1),						// [1]
		Opcode::SetLocal(1),						// [] + local1 = 1
		Opcode::GetLocal(0),						// [arg]
		Opcode::I32Const(0),						// [arg, 0]
		Opcode::I32Eq,								// [arg == 0]
		Opcode::GetLocal(1),						// [arg == 0, local1]
		Opcode::I32Const(1),						// [arg == 0, local1, 1]
		Opcode::I32Eq,								// [arg == 0, local1 == 1]
		Opcode::I32Add,								// [arg == 0 + local1 == 1]
		Opcode::End]);

	assert_eq!(run_function_i32(&body, 0).unwrap(), 2);
}

/// https://github.com/WebAssembly/wabt/blob/8e1f6031e9889ba770c7be4a9b084da5f14456a0/test/interp/br.txt#L26
#[test]
fn br_1() {
	let body = Opcodes::new(vec![
		Opcode::Block(BlockType::NoResult,					// block1
			Opcodes::new(vec![
				Opcode::Block(BlockType::NoResult,			//   block2
					Opcodes::new(vec![
						Opcode::I32Const(1),				//     [1]
						Opcode::If(BlockType::NoResult,		//     if 1
							Opcodes::new(vec![
								Opcode::Br(2),				//       break from block2
								Opcode::End,				//     end if
							])),
						Opcode::I32Const(1),				//     [1]
						Opcode::SetLocal(0),				//     [] + arg = 1
						Opcode::End,						//   end (block2)
					])),
				Opcode::I32Const(1),						//   [1]
				Opcode::SetLocal(1),						//   [] + local1 = 1
				Opcode::End,								// end (block1)
			])),
		Opcode::I32Const(1),								// [1]
		Opcode::SetLocal(2),								// [] + local2 = 1
		Opcode::GetLocal(0),								// [arg]
		Opcode::I32Const(0),								// [arg, 0]
		Opcode::I32Eq,										// [arg == 0]
		Opcode::GetLocal(1),								// [arg == 0, local1]
		Opcode::I32Const(0),								// [arg == 0, local1, 0]
		Opcode::I32Eq,										// [arg == 0, local1 == 0]
		Opcode::I32Add,										// [arg == 0 + local1 == 0]
		Opcode::GetLocal(2),								// [arg == 0 + local1 == 0, local2]
		Opcode::I32Const(1),								// [arg == 0 + local1 == 0, local2, 1]
		Opcode::I32Eq,										// [arg == 0 + local1 == 0, local2 == 1]
		Opcode::I32Add,										// [arg == 0 + local1 == 0 + local2 == 1]
		Opcode::End]);

	assert_eq!(run_function_i32(&body, 0).unwrap(), 3);
}

/// https://github.com/WebAssembly/wabt/blob/8e1f6031e9889ba770c7be4a9b084da5f14456a0/test/interp/br.txt#L56
#[test]
fn br_2() {
	let body = Opcodes::new(vec![
		Opcode::Block(BlockType::NoResult,					// block1
			Opcodes::new(vec![
				Opcode::Block(BlockType::NoResult,			//   block2
					Opcodes::new(vec![
						Opcode::I32Const(1),				//     [1]
						Opcode::If(BlockType::NoResult,		//     if 1
							Opcodes::new(vec![
								Opcode::Br(2),				//       break from block2
								Opcode::End,				//     end if
							])),
						Opcode::I32Const(1),				//     [1]
						Opcode::Return,						//     return 1
						Opcode::End,						//   end (block2)
					])),
				Opcode::End,								// end (block1)
			])),
		Opcode::I32Const(2),								// [2]
		Opcode::Return,										// return 2
		Opcode::End]);

	assert_eq!(run_function_i32(&body, 0).unwrap(), 2);
}

/// https://github.com/WebAssembly/wabt/blob/8e1f6031e9889ba770c7be4a9b084da5f14456a0/test/interp/br.txt#L71
#[test]
fn br_3() {
	let body = Opcodes::new(vec![
		Opcode::Block(BlockType::NoResult,					// block1
			Opcodes::new(vec![
				Opcode::Loop(BlockType::NoResult,			//   loop
					Opcodes::new(vec![
						Opcode::GetLocal(0),				//     [arg]
						Opcode::I32Const(1),				//     [arg, 1]
						Opcode::I32Add,						//     [arg + 1]
						Opcode::SetLocal(0),				//     [] + arg = arg + 1
						Opcode::GetLocal(0),				//     [arg]
						Opcode::I32Const(5),				//     [arg, 5]
						Opcode::I32GeS,						//     [5 >= arg]
						Opcode::If(BlockType::NoResult,		//     if 5 >= arg
							Opcodes::new(vec![
								Opcode::Br(2),				//       break from block1
								Opcode::End,				//     end
							])),
						Opcode::GetLocal(0),				//     [arg]
						Opcode::I32Const(4),				//     [arg, 4]
						Opcode::I32Eq,						//     [arg == 4]
						Opcode::If(BlockType::NoResult,		//     if arg == 4
							Opcodes::new(vec![
								Opcode::Br(1),				//       break from loop
								Opcode::End,				//     end
							])),
						Opcode::GetLocal(0),				//     [arg]
						Opcode::SetLocal(1),				//     [] + local1 = arg
						Opcode::Br(0),						//     continue loop
						Opcode::End,						//   end (loop)
					])),
				Opcode::End,								// end (block1)
			])),
		Opcode::GetLocal(1),								// [local1]
		Opcode::Return,										// return local1
		Opcode::End]);

	assert_eq!(run_function_i32(&body, 0).unwrap(), 3);
}

/// https://github.com/WebAssembly/wabt/blob/8e1f6031e9889ba770c7be4a9b084da5f14456a0/test/interp/expr-br.txt
#[test]
fn expr_br() {
	let body = Opcodes::new(vec![
		Opcode::Block(BlockType::Value(ValueType::I32),		// block1
			Opcodes::new(vec![
				Opcode::GetLocal(0),						//   [arg]
				Opcode::I32Const(0),						//   [arg, 0]
				Opcode::I32Eq,								//   [arg == 0]
				Opcode::If(BlockType::NoResult,				//   if arg == 0
					Opcodes::new(vec![
						Opcode::I32Const(1),				//     [1]
						Opcode::Br(1),						//     break from block1
						Opcode::End,						//   end (if)
					])),
				Opcode::I32Const(2),						//   [2]
				Opcode::End,								// end (block1)
			])),
		Opcode::End]);

	assert_eq!(run_function_i32(&body, 0).unwrap(), 1);
	assert_eq!(run_function_i32(&body, 1).unwrap(), 2);
}

/// https://github.com/WebAssembly/wabt/blob/8e1f6031e9889ba770c7be4a9b084da5f14456a0/test/interp/brif.txt
#[test]
fn brif() {
	let body = Opcodes::new(vec![
		Opcode::Block(BlockType::NoResult,					// block1
			Opcodes::new(vec![
				Opcode::GetLocal(0),						//   [arg]
				Opcode::BrIf(0),							//   if arg != 0: break from block1
				Opcode::I32Const(1),						//   [1]
				Opcode::Return,								//   return 1
				Opcode::End,								// end (block1)
			])),
		Opcode::I32Const(2),								// [2]
		Opcode::Return,										// return 2
		Opcode::End]);

	assert_eq!(run_function_i32(&body, 0).unwrap(), 1);
	assert_eq!(run_function_i32(&body, 1).unwrap(), 2);
}

/// https://github.com/WebAssembly/wabt/blob/8e1f6031e9889ba770c7be4a9b084da5f14456a0/test/interp/brif-loop.txt
#[test]
fn brif_loop() {
	let body = Opcodes::new(vec![
		Opcode::Loop(BlockType::NoResult,					// loop
			Opcodes::new(vec![
				Opcode::GetLocal(1),						//   [local1]
				Opcode::I32Const(1),						//   [local1, 1]
				Opcode::I32Add,								//   [local1 + 1]
				Opcode::SetLocal(1),						//   [] + local1 = local1 + 1
				Opcode::GetLocal(1),						//   [local1]
				Opcode::GetLocal(0),						//   [local1, arg]
				Opcode::I32LtS,								//   [local1 < arg]
				Opcode::BrIf(0),							//   break loop if local1 < arg
				Opcode::End,								// end (loop)
			])),
		Opcode::GetLocal(1),								// [local1]
		Opcode::Return,										// return
		Opcode::End]);

	assert_eq!(run_function_i32(&body, 3).unwrap(), 3);
	assert_eq!(run_function_i32(&body, 10).unwrap(), 10);
}

/// https://github.com/WebAssembly/wabt/blob/8e1f6031e9889ba770c7be4a9b084da5f14456a0/test/interp/expr-brif.txt
#[test]
fn expr_brif() {
	let body = Opcodes::new(vec![
		Opcode::Loop(BlockType::NoResult,		// loop
			Opcodes::new(vec![
				Opcode::GetLocal(1),			//   [local1]
				Opcode::I32Const(1),			//   [local1, 1]
				Opcode::I32Add,					//   [local1 + 1]
				Opcode::SetLocal(1),			//   [] + local1 = local1 + 1
				Opcode::GetLocal(1),			//   [local1]
				Opcode::GetLocal(0),			//   [local1, local0]
				Opcode::I32LtS,					//   [local1 < local0]
				Opcode::BrIf(0),				//   if local1 < local0: break from loop
				Opcode::End,					// end (loop)
			])),
		Opcode::GetLocal(1),					// [local1]
		Opcode::End]);

	assert_eq!(run_function_i32(&body, 3).unwrap(), 3);
	assert_eq!(run_function_i32(&body, 10).unwrap(), 10);
}

/// https://github.com/WebAssembly/wabt/blob/8e1f6031e9889ba770c7be4a9b084da5f14456a0/test/interp/brtable.txt
#[test]
fn brtable() {
	let body = Opcodes::new(vec![
		Opcode::Block(BlockType::NoResult,										// block3
			Opcodes::new(vec![
				Opcode::Block(BlockType::NoResult,								//   block2
					Opcodes::new(vec![
						Opcode::Block(BlockType::NoResult,						//     block1
							Opcodes::new(vec![
								Opcode::Block(BlockType::NoResult,				//       block0
									Opcodes::new(vec![
										Opcode::GetLocal(0),					//         [arg]
										Opcode::BrTable(vec![0, 1, 2], 3),		//         br_table
										Opcode::End,							//       end (block0)
									])),
								Opcode::I32Const(0),							//       [0]
								Opcode::Return,									//       return 0
								Opcode::End,									//     end (block1)
							])),
						Opcode::I32Const(1),									//       [1]
						Opcode::Return,											//       return 1
						Opcode::End,											//   end (block2)
					])),
				Opcode::End,													// end (block3)
			])),
		Opcode::I32Const(2),													// [2]
		Opcode::Return,															// return 2
		Opcode::End]);

	assert_eq!(run_function_i32(&body, 0).unwrap(), 0);
	assert_eq!(run_function_i32(&body, 1).unwrap(), 1);
	assert_eq!(run_function_i32(&body, 2).unwrap(), 2);
	assert_eq!(run_function_i32(&body, 3).unwrap(), 2);
}

/// https://github.com/WebAssembly/wabt/blob/8e1f6031e9889ba770c7be4a9b084da5f14456a0/test/interp/return.txt
#[test]
fn return_test() {
	let body = Opcodes::new(vec![
		Opcode::GetLocal(0),
		Opcode::I32Const(0),
		Opcode::I32Eq,
		Opcode::If(BlockType::NoResult,
			Opcodes::new(vec![
				Opcode::I32Const(1),
				Opcode::Return,
				Opcode::End,
			])),
		Opcode::GetLocal(0),
		Opcode::I32Const(1),
		Opcode::I32Eq,
		Opcode::If(BlockType::NoResult,
			Opcodes::new(vec![
				Opcode::I32Const(2),
				Opcode::Return,
				Opcode::End,
			])),
		Opcode::I32Const(3),
		Opcode::Return,
		Opcode::End]);

	assert_eq!(run_function_i32(&body, 0).unwrap(), 1);
	assert_eq!(run_function_i32(&body, 1).unwrap(), 2);
	assert_eq!(run_function_i32(&body, 5).unwrap(), 3);
}

/// https://github.com/WebAssembly/wabt/blob/8e1f6031e9889ba770c7be4a9b084da5f14456a0/test/interp/return-void.txt
#[test]
fn return_void() {
	let body = Opcodes::new(vec![
		Opcode::GetLocal(0),
		Opcode::I32Const(0),
		Opcode::I32Eq,
		Opcode::If(BlockType::NoResult,
			Opcodes::new(vec![
				Opcode::Return,
				Opcode::End,
			])),
		Opcode::I32Const(0),
		Opcode::I32Const(1),
		Opcode::I32Store(0, 2),
		Opcode::End,
	]);

	let module = module()
		.memory().build()
		.function().main()
			.signature().param().i32().build()
			.body().with_opcodes(body).build()
			.build()
		.build();
	let program = ProgramInstance::new();
	let module = program.add_module("main", module).unwrap();

	module.execute_main(vec![RuntimeValue::I32(0)]).unwrap();
	let memory = module.memory(ItemIndex::IndexSpace(0)).unwrap();
	assert_eq!(memory.get(0, 4).unwrap(), vec![0, 0, 0, 0]);

	module.execute_main(vec![RuntimeValue::I32(1)]).unwrap();
	let memory = module.memory(ItemIndex::IndexSpace(0)).unwrap();
	assert_eq!(memory.get(0, 4).unwrap(), vec![1, 0, 0, 0]);
}

/// https://github.com/WebAssembly/wabt/blob/8e1f6031e9889ba770c7be4a9b084da5f14456a0/test/interp/call.txt#L3
#[test]
fn call_1() {
	let body1 = Opcodes::new(vec![
		Opcode::I32Const(1),
		Opcode::I64Const(2),
		// f32 && f64 are serialized using binary32 && binary64 formats
		// http://babbage.cs.qc.cuny.edu/IEEE-754/
		Opcode::F32Const(0x40400000),
		Opcode::F64Const(0x4010000000000000),
		Opcode::Call(1),
		Opcode::End,
	]);

	let body2 = Opcodes::new(vec![
		Opcode::GetLocal(1),
		Opcode::I32WarpI64,
		Opcode::GetLocal(0),
		Opcode::I32Add,
		Opcode::GetLocal(2),
		Opcode::I32TruncSF32,
		Opcode::I32Add,
		Opcode::GetLocal(3),
		Opcode::I32TruncSF64,
		Opcode::I32Add,
		Opcode::Return,
		Opcode::End,
	]);

	let module = module()
		.memory().build()
		.function().main()
			.signature().return_type().i32().build()
			.body().with_opcodes(body1).build()
			.build()
		.function()
			.signature()
				.param().i32()
				.param().i64()
				.param().f32()
				.param().f64()
				.return_type().i32()
				.build()
			.body().with_opcodes(body2).build()
			.build()
		.build();

	let program = ProgramInstance::new();
	let module = program.add_module("main", module).unwrap();
	assert_eq!(module.execute_main(vec![]).unwrap().unwrap(), RuntimeValue::I32(10));
}

/// https://github.com/WebAssembly/wabt/blob/8e1f6031e9889ba770c7be4a9b084da5f14456a0/test/interp/call.txt#L23
#[test]
fn call_2() {
	let body1 = Opcodes::new(vec![
		Opcode::I32Const(10),
		Opcode::Call(1),
		Opcode::End,
	]);

	let body2 = Opcodes::new(vec![
		Opcode::GetLocal(0),
		Opcode::I32Const(0),
		Opcode::I32GtS,
		Opcode::If(BlockType::NoResult,
			Opcodes::new(vec![
				Opcode::GetLocal(0),
				Opcode::GetLocal(0),
				Opcode::I32Const(1),
				Opcode::I32Sub,
				Opcode::Call(1),
				Opcode::I32Mul,
				Opcode::Return,
				Opcode::Else,
				Opcode::I32Const(1),
				Opcode::Return,
				Opcode::End,
			])),
		Opcode::End,
	]);

	let module = module()
		.function().main()
			.signature().return_type().i32().build()
			.body().with_opcodes(body1).build()
			.build()
		.function()
			.signature()
				.param().i32()
				.return_type().i32()
				.build()
			.body().with_opcodes(body2).build()
			.build()
		.build();

	let program = ProgramInstance::new();
	let module = program.add_module("main", module).unwrap();
	assert_eq!(module.execute_main(vec![]).unwrap().unwrap(), RuntimeValue::I32(3628800));
}

/// https://github.com/WebAssembly/wabt/blob/8e1f6031e9889ba770c7be4a9b084da5f14456a0/test/interp/call-zero-args.txt
#[test]
fn call_zero_args() {
	let body1 = Opcodes::new(vec![
		Opcode::I32Const(42),
		Opcode::End,
	]);

	let body2 = Opcodes::new(vec![
		Opcode::GetLocal(0),
		Opcode::GetLocal(1),
		Opcode::I32Add,
		Opcode::End,
	]);

	let body3 = Opcodes::new(vec![
		Opcode::I32Const(1),
		Opcode::Call(0),
		Opcode::Call(1),
		Opcode::End,
	]);

	let module = module()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(body1).build()
			.build()
		.function()
			.signature()
				.param().i32()
				.param().i32()
				.return_type().i32()
				.build()
			.body().with_opcodes(body2).build()
			.build()
		.function().main()
			.body().with_opcodes(body3).build()
			.build()
		.build();

	let program = ProgramInstance::new();
	let module = program.add_module("main", module).unwrap();
	assert_eq!(module.execute_main(vec![]).unwrap().unwrap(), RuntimeValue::I32(43));
}

/// https://github.com/WebAssembly/wabt/blob/8e1f6031e9889ba770c7be4a9b084da5f14456a0/test/interp/callimport-zero-args.txt
#[test]
fn callimport_zero_zrgs() {
	// TODO: import needed
}

/// https://github.com/WebAssembly/wabt/blob/8e1f6031e9889ba770c7be4a9b084da5f14456a0/test/interp/callindirect.txt#L31
#[test]
fn callindirect_1() {
	let body1 = Opcodes::new(vec![
		Opcode::I32Const(0),
		Opcode::End,
	]);

	let body2 = Opcodes::new(vec![
		Opcode::I32Const(1),
		Opcode::End,
	]);

	let body3 = Opcodes::new(vec![
		Opcode::GetLocal(0),
		Opcode::CallIndirect(0, false),
		Opcode::End,
	]);

	let module = module()
		.table()
			.with_min(2)
			.with_element(0, vec![0, 1])
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(body1).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(body2).build()
			.build()
		.function().main()
			.signature()
				.param().i32()
				.return_type().i32()
				.build()
			.body().with_opcodes(body3).build()
			.build()
		.build();

	let program = ProgramInstance::new();
	let module = program.add_module("main", module).unwrap();
	assert_eq!(module.execute_main(vec![RuntimeValue::I32(0)]).unwrap().unwrap(), RuntimeValue::I32(0));
	assert_eq!(module.execute_main(vec![RuntimeValue::I32(1)]).unwrap().unwrap(), RuntimeValue::I32(1));
}

/// https://github.com/WebAssembly/wabt/blob/8e1f6031e9889ba770c7be4a9b084da5f14456a0/test/interp/callindirect.txt#L39
#[test]
fn callindirect_2() {
	let body1 = Opcodes::new(vec![
		Opcode::GetLocal(0),
		Opcode::GetLocal(1),
		Opcode::I32Add,
		Opcode::End,
	]);

	let body2 = Opcodes::new(vec![
		Opcode::GetLocal(0),
		Opcode::GetLocal(1),
		Opcode::I32Sub,
		Opcode::End,
	]);

	let body3 = Opcodes::new(vec![
		Opcode::GetLocal(0),
		Opcode::I32Ctz,
		Opcode::End,
	]);

	let body4 = Opcodes::new(vec![
		Opcode::GetLocal(0),
		Opcode::GetLocal(1),
		Opcode::GetLocal(2),
		Opcode::CallIndirect(0, false),
		Opcode::End,
	]);

	let module = module()
		.table()
			.with_min(3)
			.with_element(0, vec![0, 1, 2])
			.build()
		.function()
			.signature()
				.param().i32()
				.param().i32()
				.return_type().i32().build()
			.body().with_opcodes(body1).build()
			.build()
		.function()
			.signature()
				.param().i32()
				.param().i32()
				.return_type().i32().build()
			.body().with_opcodes(body2).build()
			.build()
		.function()
			.signature()
				.param().i32()
				.return_type().i32().build()
			.body().with_opcodes(body3).build()
			.build()
		.function().main()
			.signature()
				.param().i32()
				.param().i32()
				.param().i32()
				.return_type().i32()
				.build()
			.body().with_opcodes(body4).build()
			.build()
		.build();

	let program = ProgramInstance::new();
	let module = program.add_module("main", module).unwrap();
	assert_eq!(module.execute_main(vec![RuntimeValue::I32(10), RuntimeValue::I32(4), RuntimeValue::I32(0)]).unwrap().unwrap(), RuntimeValue::I32(14));
	assert_eq!(module.execute_main(vec![RuntimeValue::I32(10), RuntimeValue::I32(4), RuntimeValue::I32(1)]).unwrap().unwrap(), RuntimeValue::I32(6));
	assert_eq!(module.execute_main(vec![RuntimeValue::I32(10), RuntimeValue::I32(4), RuntimeValue::I32(2)]).unwrap_err(),
		Error::Function("expected function with signature ([I32, I32]) -> Some(I32) when got with ([I32]) -> Some(I32)".into()));
	assert_eq!(module.execute_main(vec![RuntimeValue::I32(10), RuntimeValue::I32(4), RuntimeValue::I32(3)]).unwrap_err(),
		Error::Table("trying to read table item with index 3 when there are only 3 items".into()));
}

/// https://github.com/WebAssembly/wabt/blob/8e1f6031e9889ba770c7be4a9b084da5f14456a0/test/interp/select.txt
#[test]
fn select() {
	let body1 = Opcodes::new(vec![
		Opcode::I32Const(1),
		Opcode::I32Const(2),
		Opcode::GetLocal(0),
		Opcode::Select,
		Opcode::End,
	]);

	let body2 = Opcodes::new(vec![
		Opcode::I64Const(1),
		Opcode::I64Const(2),
		Opcode::GetLocal(0),
		Opcode::Select,
		Opcode::End,
	]);

	let body3 = Opcodes::new(vec![
		// f32 && f64 are serialized using binary32 && binary64 formats
		// http://babbage.cs.qc.cuny.edu/IEEE-754/
		Opcode::F32Const(0x3F800000),
		Opcode::F32Const(0x40000000),
		Opcode::GetLocal(0),
		Opcode::Select,
		Opcode::End,
	]);

	let body4 = Opcodes::new(vec![
		// f32 && f64 are serialized using binary32 && binary64 formats
		// http://babbage.cs.qc.cuny.edu/IEEE-754/
		Opcode::F64Const(0x3FF0000000000000),
		Opcode::F64Const(0x4000000000000000),
		Opcode::GetLocal(0),
		Opcode::Select,
		Opcode::End,
	]);

	let module = module()
		.function()
			.signature().param().i32().return_type().i32().build()
			.body().with_opcodes(body1).build()
			.build()
		.function()
			.signature().param().i32().return_type().i64().build()
			.body().with_opcodes(body2).build()
			.build()
		.function()
			.signature().param().i32().return_type().f32().build()
			.body().with_opcodes(body3).build()
			.build()
		.function()
			.signature().param().i32().return_type().f64().build()
			.body().with_opcodes(body4).build()
			.build()
		.build();

	let program = ProgramInstance::new();
	let module = program.add_module("main", module).unwrap();
	assert_eq!(module.execute(0, vec![RuntimeValue::I32(0)]).unwrap().unwrap(), RuntimeValue::I32(2));
	assert_eq!(module.execute(0, vec![RuntimeValue::I32(1)]).unwrap().unwrap(), RuntimeValue::I32(1));
	assert_eq!(module.execute(1, vec![RuntimeValue::I32(0)]).unwrap().unwrap(), RuntimeValue::I64(2));
	assert_eq!(module.execute(1, vec![RuntimeValue::I32(1)]).unwrap().unwrap(), RuntimeValue::I64(1));
	assert_eq!(module.execute(2, vec![RuntimeValue::I32(0)]).unwrap().unwrap(), RuntimeValue::F32(2f32));
	assert_eq!(module.execute(2, vec![RuntimeValue::I32(1)]).unwrap().unwrap(), RuntimeValue::F32(1f32));
	assert_eq!(module.execute(3, vec![RuntimeValue::I32(0)]).unwrap().unwrap(), RuntimeValue::F64(2f64));
	assert_eq!(module.execute(3, vec![RuntimeValue::I32(1)]).unwrap().unwrap(), RuntimeValue::F64(1f64));
}