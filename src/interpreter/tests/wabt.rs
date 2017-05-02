///! Tests from https://github.com/WebAssembly/wabt/tree/8e1f6031e9889ba770c7be4a9b084da5f14456a0/test/interp

// TODO: https://github.com/WebAssembly/wabt/blob/8e1f6031e9889ba770c7be4a9b084da5f14456a0/test/interp/import.txt

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

/// https://github.com/WebAssembly/wabt/blob/8e1f6031e9889ba770c7be4a9b084da5f14456a0/test/interp/binary.txt#L3
#[test]
fn binary_i32() {
	let module = module()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(1),
				Opcode::I32Const(2),
				Opcode::I32Add,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(20),
				Opcode::I32Const(4),
				Opcode::I32Sub,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(3),
				Opcode::I32Const(7),
				Opcode::I32Mul,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(-4),
				Opcode::I32Const(2),
				Opcode::I32DivS,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(-4),
				Opcode::I32Const(2),
				Opcode::I32DivU,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(-5),
				Opcode::I32Const(2),
				Opcode::I32RemS,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(-5),
				Opcode::I32Const(2),
				Opcode::I32RemU,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(11),
				Opcode::I32Const(5),
				Opcode::I32And,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(11),
				Opcode::I32Const(5),
				Opcode::I32Or,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(11),
				Opcode::I32Const(5),
				Opcode::I32Xor,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(-100),
				Opcode::I32Const(3),
				Opcode::I32Shl,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(-100),
				Opcode::I32Const(3),
				Opcode::I32ShrU,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(-100),
				Opcode::I32Const(3),
				Opcode::I32ShrS,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(-100),
				Opcode::I32Const(3),
				Opcode::I32Rotl,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(-100),
				Opcode::I32Const(3),
				Opcode::I32Rotr,
				Opcode::End,
			])).build()
			.build()
		.build();

	let program = ProgramInstance::new();
	let module = program.add_module("main", module).unwrap();
	assert_eq!(module.execute(0, vec![]).unwrap().unwrap(), RuntimeValue::I32(3));
	assert_eq!(module.execute(1, vec![]).unwrap().unwrap(), RuntimeValue::I32(16));
	assert_eq!(module.execute(2, vec![]).unwrap().unwrap(), RuntimeValue::I32(21));
	assert_eq!(module.execute(3, vec![]).unwrap().unwrap(), RuntimeValue::I32(-2)); // 4294967294
	assert_eq!(module.execute(4, vec![]).unwrap().unwrap(), RuntimeValue::I32(2147483646));
	assert_eq!(module.execute(5, vec![]).unwrap().unwrap(), RuntimeValue::I32(-1)); // 4294967295
	assert_eq!(module.execute(6, vec![]).unwrap().unwrap(), RuntimeValue::I32(1));
	assert_eq!(module.execute(7, vec![]).unwrap().unwrap(), RuntimeValue::I32(1));
	assert_eq!(module.execute(8, vec![]).unwrap().unwrap(), RuntimeValue::I32(15));
	assert_eq!(module.execute(9, vec![]).unwrap().unwrap(), RuntimeValue::I32(14));
	assert_eq!(module.execute(10, vec![]).unwrap().unwrap(), RuntimeValue::I32(-800)); // 4294966496
	assert_eq!(module.execute(11, vec![]).unwrap().unwrap(), RuntimeValue::I32(536870899));
	assert_eq!(module.execute(12, vec![]).unwrap().unwrap(), RuntimeValue::I32(-13)); // 4294967283
	assert_eq!(module.execute(13, vec![]).unwrap().unwrap(), RuntimeValue::I32(-793)); // 4294966503
	assert_eq!(module.execute(14, vec![]).unwrap().unwrap(), RuntimeValue::I32(-1610612749)); // 2684354547
}

/// https://github.com/WebAssembly/wabt/blob/8e1f6031e9889ba770c7be4a9b084da5f14456a0/test/interp/binary.txt#L65
#[test]
fn binary_i64() {
	let module = module()
		.function()
			.signature().return_type().i64().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I64Const(1),
				Opcode::I64Const(2),
				Opcode::I64Add,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i64().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I64Const(20),
				Opcode::I64Const(4),
				Opcode::I64Sub,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i64().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I64Const(3),
				Opcode::I64Const(7),
				Opcode::I64Mul,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i64().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I64Const(-4),
				Opcode::I64Const(2),
				Opcode::I64DivS,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i64().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I64Const(-4),
				Opcode::I64Const(2),
				Opcode::I64DivU,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i64().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I64Const(-5),
				Opcode::I64Const(2),
				Opcode::I64RemS,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i64().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I64Const(-5),
				Opcode::I64Const(2),
				Opcode::I64RemU,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i64().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I64Const(11),
				Opcode::I64Const(5),
				Opcode::I64And,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i64().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I64Const(11),
				Opcode::I64Const(5),
				Opcode::I64Or,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i64().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I64Const(11),
				Opcode::I64Const(5),
				Opcode::I64Xor,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i64().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I64Const(-100),
				Opcode::I64Const(3),
				Opcode::I64Shl,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i64().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I64Const(-100),
				Opcode::I64Const(3),
				Opcode::I64ShrU,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i64().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I64Const(-100),
				Opcode::I64Const(3),
				Opcode::I64ShrS,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i64().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I64Const(-100),
				Opcode::I64Const(3),
				Opcode::I64Rotl,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i64().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I64Const(-100),
				Opcode::I64Const(3),
				Opcode::I64Rotr,
				Opcode::End,
			])).build()
			.build()
		.build();

	let program = ProgramInstance::new();
	let module = program.add_module("main", module).unwrap();
	assert_eq!(module.execute(0, vec![]).unwrap().unwrap(), RuntimeValue::I64(3));
	assert_eq!(module.execute(1, vec![]).unwrap().unwrap(), RuntimeValue::I64(16));
	assert_eq!(module.execute(2, vec![]).unwrap().unwrap(), RuntimeValue::I64(21));
	assert_eq!(module.execute(3, vec![]).unwrap().unwrap(), RuntimeValue::I64(-2)); // 18446744073709551614
	assert_eq!(module.execute(4, vec![]).unwrap().unwrap(), RuntimeValue::I64(9223372036854775806));
	assert_eq!(module.execute(5, vec![]).unwrap().unwrap(), RuntimeValue::I64(-1)); // 18446744073709551615
	assert_eq!(module.execute(6, vec![]).unwrap().unwrap(), RuntimeValue::I64(1));
	assert_eq!(module.execute(7, vec![]).unwrap().unwrap(), RuntimeValue::I64(1));
	assert_eq!(module.execute(8, vec![]).unwrap().unwrap(), RuntimeValue::I64(15));
	assert_eq!(module.execute(9, vec![]).unwrap().unwrap(), RuntimeValue::I64(14));
	assert_eq!(module.execute(10, vec![]).unwrap().unwrap(), RuntimeValue::I64(-800)); // 18446744073709550816
	assert_eq!(module.execute(11, vec![]).unwrap().unwrap(), RuntimeValue::I64(2305843009213693939));
	assert_eq!(module.execute(12, vec![]).unwrap().unwrap(), RuntimeValue::I64(-13)); // 18446744073709551603
	assert_eq!(module.execute(13, vec![]).unwrap().unwrap(), RuntimeValue::I64(-793)); // 18446744073709550823
	assert_eq!(module.execute(14, vec![]).unwrap().unwrap(), RuntimeValue::I64(-6917529027641081869)); // 11529215046068469747
}

/// https://github.com/WebAssembly/wabt/blob/8e1f6031e9889ba770c7be4a9b084da5f14456a0/test/interp/binary.txt#L3
#[test]
fn binary_f32() {
	// f32 && f64 are serialized using binary32 && binary64 formats
	// http://babbage.cs.qc.cuny.edu/IEEE-754/
	let module = module()
		.function()
			.signature().return_type().f32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F32Const(0x3FA00000), // 1.25
				Opcode::F32Const(0x40700000), // 3.75
				Opcode::F32Add,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().f32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F32Const(0x40900000), // 4.5
				Opcode::F32Const(0x461C4000), // 1e4
				Opcode::F32Sub,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().f32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F32Const(0x449A5000), // 1234.5
				Opcode::F32Const(0xC0DC0000), // -6.875
				Opcode::F32Mul,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().f32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F32Const(0x56B5E621), // 1e14
				Opcode::F32Const(0xC8435000), // -2e5
				Opcode::F32Div,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().f32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F32Const(0x00000000), // 0
				Opcode::F32Const(0x00000000), // 0
				Opcode::F32Min,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().f32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F32Const(0x00000000), // 0
				Opcode::F32Const(0x00000000), // 0
				Opcode::F32Max,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().f32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F32Const(0x00000000), // 0
				Opcode::F32Const(0x00000000), // 0
				Opcode::F32Copysign,
				Opcode::End,
			])).build()
			.build()
		.build();

	let program = ProgramInstance::new();
	let module = program.add_module("main", module).unwrap();
	assert_eq!(module.execute(0, vec![]).unwrap().unwrap(), RuntimeValue::F32(5.000000));
	assert_eq!(module.execute(1, vec![]).unwrap().unwrap(), RuntimeValue::F32(-9995.500000));
	assert_eq!(module.execute(2, vec![]).unwrap().unwrap(), RuntimeValue::F32(-8487.187500));
	assert_eq!(module.execute(3, vec![]).unwrap().unwrap(), RuntimeValue::F32(-500000000.000000));
	assert_eq!(module.execute(4, vec![]).unwrap().unwrap(), RuntimeValue::F32(0.000000));
	assert_eq!(module.execute(5, vec![]).unwrap().unwrap(), RuntimeValue::F32(0.000000));
	assert_eq!(module.execute(6, vec![]).unwrap().unwrap(), RuntimeValue::F32(0.000000));
}

/// https://github.com/WebAssembly/wabt/blob/8e1f6031e9889ba770c7be4a9b084da5f14456a0/test/interp/binary.txt#L157
#[test]
fn binary_f64() {
	// f32 && f64 are serialized using binary32 && binary64 formats
	// http://babbage.cs.qc.cuny.edu/IEEE-754/
	let module = module()
		.function()
			.signature().return_type().f64().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F64Const(0x41CD6F3458800000), // 987654321
				Opcode::F64Const(0x419D6F3454000000), // 123456789
				Opcode::F64Add,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().f64().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F64Const(0x4C33A8A41D39B24E), // 1234e56
				Opcode::F64Const(0x44DD1DE3D2D5C713), // 5.5e23
				Opcode::F64Sub,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().f64().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F64Const(0xC132C4B000000000), // -123e4
				Opcode::F64Const(0x416789FE40000000), // 12341234
				Opcode::F64Mul,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().f64().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F64Const(0x6974E718D7D7625A), // 1e200
				Opcode::F64Const(0x4A511B0EC57E649A), // 1e50
				Opcode::F64Div,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().f64().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F64Const(0x0000000000000000), // 0
				Opcode::F64Const(0x0000000000000000), // 0
				Opcode::F64Min,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().f64().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F64Const(0x0000000000000000), // 0
				Opcode::F64Const(0x0000000000000000), // 0
				Opcode::F64Max,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().f64().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F64Const(0x0000000000000000), // 0
				Opcode::F64Const(0x0000000000000000), // 0
				Opcode::F64Copysign,
				Opcode::End,
			])).build()
			.build()
		.build();

	let program = ProgramInstance::new();
	let module = program.add_module("main", module).unwrap();
	assert_eq!(module.execute(0, vec![]).unwrap().unwrap(), RuntimeValue::F64(1111111110.000000));
	assert_eq!(module.execute(1, vec![]).unwrap().unwrap(), RuntimeValue::F64(123400000000000007812762268812638756607430593436581896388608.000000));
	assert_eq!(module.execute(2, vec![]).unwrap().unwrap(), RuntimeValue::F64(-15179717820000.000000));
	// TODO: result differs
	// assert_eq!(module.execute(3, vec![]).unwrap().unwrap(), RuntimeValue::F64(99999999999999998083559617243737459057312001403031879309116481015410011220367858297629826861622.0f64));
	assert_eq!(module.execute(4, vec![]).unwrap().unwrap(), RuntimeValue::F64(0.000000));
	assert_eq!(module.execute(5, vec![]).unwrap().unwrap(), RuntimeValue::F64(0.000000));
	assert_eq!(module.execute(6, vec![]).unwrap().unwrap(), RuntimeValue::F64(0.000000));
}

/// https://github.com/WebAssembly/wabt/blob/8e1f6031e9889ba770c7be4a9b084da5f14456a0/test/interp/cast.txt
#[test]
fn cast() {
	// f32 && f64 are serialized using binary32 && binary64 formats
	// http://babbage.cs.qc.cuny.edu/IEEE-754/
	let module = module()
		.function()
			.signature().return_type().f32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(0x40900000),
				Opcode::F32ReinterpretI32,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F32Const(0xC0600000),
				Opcode::I32ReinterpretF32,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().f64().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I64Const(0x405f480000000000),
				Opcode::F64ReinterpretI64,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i64().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F64Const(0x42099C82CC000000),
				Opcode::I64ReinterpretF64,
				Opcode::End,
			])).build()
			.build()
		.build();

	let program = ProgramInstance::new();
	let module = program.add_module("main", module).unwrap();
	assert_eq!(module.execute(0, vec![]).unwrap().unwrap(), RuntimeValue::F32(4.5));
	assert_eq!(module.execute(1, vec![]).unwrap().unwrap(), RuntimeValue::I32(-1067450368)); // 3227516928
	assert_eq!(module.execute(2, vec![]).unwrap().unwrap(), RuntimeValue::F64(125.125000));
	assert_eq!(module.execute(3, vec![]).unwrap().unwrap(), RuntimeValue::I64(4758506566875873280));
}

/// https://github.com/WebAssembly/wabt/blob/8e1f6031e9889ba770c7be4a9b084da5f14456a0/test/interp/compare.txt#L3
#[test]
fn compare_i32() {
	let module = module()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(-1),
				Opcode::I32Const(-1),
				Opcode::I32Eq,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(1),
				Opcode::I32Const(-1),
				Opcode::I32Eq,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(1),
				Opcode::I32Const(-1),
				Opcode::I32Ne,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(-1),
				Opcode::I32Const(-1),
				Opcode::I32Ne,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(-1),
				Opcode::I32Const(1),
				Opcode::I32LtS,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(-1),
				Opcode::I32Const(-1),
				Opcode::I32LtS,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(1),
				Opcode::I32Const(-1),
				Opcode::I32LtS,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(1),
				Opcode::I32Const(-1),
				Opcode::I32LtU,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(1),
				Opcode::I32Const(1),
				Opcode::I32LtU,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(-1),
				Opcode::I32Const(1),
				Opcode::I32LtU,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(-1),
				Opcode::I32Const(1),
				Opcode::I32LeS,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(-1),
				Opcode::I32Const(-1),
				Opcode::I32LeS,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(1),
				Opcode::I32Const(-1),
				Opcode::I32LeS,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(1),
				Opcode::I32Const(-1),
				Opcode::I32LeU,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(1),
				Opcode::I32Const(1),
				Opcode::I32LeU,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(-1),
				Opcode::I32Const(1),
				Opcode::I32LeU,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(-1),
				Opcode::I32Const(1),
				Opcode::I32GtS,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(-1),
				Opcode::I32Const(-1),
				Opcode::I32GtS,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(1),
				Opcode::I32Const(-1),
				Opcode::I32GtS,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(1),
				Opcode::I32Const(-1),
				Opcode::I32GtU,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(1),
				Opcode::I32Const(1),
				Opcode::I32GtU,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(-1),
				Opcode::I32Const(1),
				Opcode::I32GtU,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(-1),
				Opcode::I32Const(1),
				Opcode::I32GeS,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(-1),
				Opcode::I32Const(-1),
				Opcode::I32GeS,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(1),
				Opcode::I32Const(-1),
				Opcode::I32GeS,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(1),
				Opcode::I32Const(-1),
				Opcode::I32GeU,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(1),
				Opcode::I32Const(1),
				Opcode::I32GeU,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(-1),
				Opcode::I32Const(1),
				Opcode::I32GeU,
				Opcode::End,
			])).build()
			.build()
		.build();

	let program = ProgramInstance::new();
	let module = program.add_module("main", module).unwrap();
	assert_eq!(module.execute(0, vec![]).unwrap().unwrap(), RuntimeValue::I32(1));
	assert_eq!(module.execute(1, vec![]).unwrap().unwrap(), RuntimeValue::I32(0));
	assert_eq!(module.execute(2, vec![]).unwrap().unwrap(), RuntimeValue::I32(1));
	assert_eq!(module.execute(3, vec![]).unwrap().unwrap(), RuntimeValue::I32(0));
	assert_eq!(module.execute(4, vec![]).unwrap().unwrap(), RuntimeValue::I32(1));
	assert_eq!(module.execute(5, vec![]).unwrap().unwrap(), RuntimeValue::I32(0));
	assert_eq!(module.execute(6, vec![]).unwrap().unwrap(), RuntimeValue::I32(0));
	assert_eq!(module.execute(7, vec![]).unwrap().unwrap(), RuntimeValue::I32(1));
	assert_eq!(module.execute(8, vec![]).unwrap().unwrap(), RuntimeValue::I32(0));
	assert_eq!(module.execute(9, vec![]).unwrap().unwrap(), RuntimeValue::I32(0));
	assert_eq!(module.execute(10, vec![]).unwrap().unwrap(), RuntimeValue::I32(1));
	assert_eq!(module.execute(11, vec![]).unwrap().unwrap(), RuntimeValue::I32(1));
	assert_eq!(module.execute(12, vec![]).unwrap().unwrap(), RuntimeValue::I32(0));
	assert_eq!(module.execute(13, vec![]).unwrap().unwrap(), RuntimeValue::I32(1));
	assert_eq!(module.execute(14, vec![]).unwrap().unwrap(), RuntimeValue::I32(1));
	assert_eq!(module.execute(15, vec![]).unwrap().unwrap(), RuntimeValue::I32(0));
	assert_eq!(module.execute(16, vec![]).unwrap().unwrap(), RuntimeValue::I32(0));
	assert_eq!(module.execute(17, vec![]).unwrap().unwrap(), RuntimeValue::I32(0));
	assert_eq!(module.execute(18, vec![]).unwrap().unwrap(), RuntimeValue::I32(1));
	assert_eq!(module.execute(19, vec![]).unwrap().unwrap(), RuntimeValue::I32(0));
	assert_eq!(module.execute(20, vec![]).unwrap().unwrap(), RuntimeValue::I32(0));
	assert_eq!(module.execute(21, vec![]).unwrap().unwrap(), RuntimeValue::I32(1));
	assert_eq!(module.execute(22, vec![]).unwrap().unwrap(), RuntimeValue::I32(0));
	assert_eq!(module.execute(23, vec![]).unwrap().unwrap(), RuntimeValue::I32(1));
	assert_eq!(module.execute(24, vec![]).unwrap().unwrap(), RuntimeValue::I32(1));
	assert_eq!(module.execute(25, vec![]).unwrap().unwrap(), RuntimeValue::I32(0));
	assert_eq!(module.execute(26, vec![]).unwrap().unwrap(), RuntimeValue::I32(1));
	assert_eq!(module.execute(27, vec![]).unwrap().unwrap(), RuntimeValue::I32(1));
}

/// https://github.com/WebAssembly/wabt/blob/8e1f6031e9889ba770c7be4a9b084da5f14456a0/test/interp/compare.txt#L123
#[test]
fn compare_i64() {
	let module = module()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I64Const(-1),
				Opcode::I64Const(-1),
				Opcode::I64Eq,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I64Const(1),
				Opcode::I64Const(-1),
				Opcode::I64Eq,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I64Const(1),
				Opcode::I64Const(-1),
				Opcode::I64Ne,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I64Const(-1),
				Opcode::I64Const(-1),
				Opcode::I64Ne,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I64Const(-1),
				Opcode::I64Const(1),
				Opcode::I64LtS,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I64Const(-1),
				Opcode::I64Const(-1),
				Opcode::I64LtS,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I64Const(1),
				Opcode::I64Const(-1),
				Opcode::I64LtS,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I64Const(1),
				Opcode::I64Const(-1),
				Opcode::I64LtU,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I64Const(1),
				Opcode::I64Const(1),
				Opcode::I64LtU,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I64Const(-1),
				Opcode::I64Const(1),
				Opcode::I64LtU,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I64Const(-1),
				Opcode::I64Const(1),
				Opcode::I64LeS,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I64Const(-1),
				Opcode::I64Const(-1),
				Opcode::I64LeS,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I64Const(1),
				Opcode::I64Const(-1),
				Opcode::I64LeS,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I64Const(1),
				Opcode::I64Const(-1),
				Opcode::I64LeU,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I64Const(1),
				Opcode::I64Const(1),
				Opcode::I64LeU,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I64Const(-1),
				Opcode::I64Const(1),
				Opcode::I64LeU,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I64Const(-1),
				Opcode::I64Const(1),
				Opcode::I64GtS,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I64Const(-1),
				Opcode::I64Const(-1),
				Opcode::I64GtS,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I64Const(1),
				Opcode::I64Const(-1),
				Opcode::I64GtS,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I64Const(1),
				Opcode::I64Const(-1),
				Opcode::I64GtU,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I64Const(1),
				Opcode::I64Const(1),
				Opcode::I64GtU,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I64Const(-1),
				Opcode::I64Const(1),
				Opcode::I64GtU,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I64Const(-1),
				Opcode::I64Const(1),
				Opcode::I64GeS,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I64Const(-1),
				Opcode::I64Const(-1),
				Opcode::I64GeS,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I64Const(1),
				Opcode::I64Const(-1),
				Opcode::I64GeS,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I64Const(1),
				Opcode::I64Const(-1),
				Opcode::I64GeU,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I64Const(1),
				Opcode::I64Const(1),
				Opcode::I64GeU,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I64Const(-1),
				Opcode::I64Const(1),
				Opcode::I64GeU,
				Opcode::End,
			])).build()
			.build()
		.build();

	let program = ProgramInstance::new();
	let module = program.add_module("main", module).unwrap();
	assert_eq!(module.execute(0, vec![]).unwrap().unwrap(), RuntimeValue::I32(1));
	assert_eq!(module.execute(1, vec![]).unwrap().unwrap(), RuntimeValue::I32(0));
	assert_eq!(module.execute(2, vec![]).unwrap().unwrap(), RuntimeValue::I32(1));
	assert_eq!(module.execute(3, vec![]).unwrap().unwrap(), RuntimeValue::I32(0));
	assert_eq!(module.execute(4, vec![]).unwrap().unwrap(), RuntimeValue::I32(1));
	assert_eq!(module.execute(5, vec![]).unwrap().unwrap(), RuntimeValue::I32(0));
	assert_eq!(module.execute(6, vec![]).unwrap().unwrap(), RuntimeValue::I32(0));
	assert_eq!(module.execute(7, vec![]).unwrap().unwrap(), RuntimeValue::I32(1));
	assert_eq!(module.execute(8, vec![]).unwrap().unwrap(), RuntimeValue::I32(0));
	assert_eq!(module.execute(9, vec![]).unwrap().unwrap(), RuntimeValue::I32(0));
	assert_eq!(module.execute(10, vec![]).unwrap().unwrap(), RuntimeValue::I32(1));
	assert_eq!(module.execute(11, vec![]).unwrap().unwrap(), RuntimeValue::I32(1));
	assert_eq!(module.execute(12, vec![]).unwrap().unwrap(), RuntimeValue::I32(0));
	assert_eq!(module.execute(13, vec![]).unwrap().unwrap(), RuntimeValue::I32(1));
	assert_eq!(module.execute(14, vec![]).unwrap().unwrap(), RuntimeValue::I32(1));
	assert_eq!(module.execute(15, vec![]).unwrap().unwrap(), RuntimeValue::I32(0));
	assert_eq!(module.execute(16, vec![]).unwrap().unwrap(), RuntimeValue::I32(0));
	assert_eq!(module.execute(17, vec![]).unwrap().unwrap(), RuntimeValue::I32(0));
	assert_eq!(module.execute(18, vec![]).unwrap().unwrap(), RuntimeValue::I32(1));
	assert_eq!(module.execute(19, vec![]).unwrap().unwrap(), RuntimeValue::I32(0));
	assert_eq!(module.execute(20, vec![]).unwrap().unwrap(), RuntimeValue::I32(0));
	assert_eq!(module.execute(21, vec![]).unwrap().unwrap(), RuntimeValue::I32(1));
	assert_eq!(module.execute(22, vec![]).unwrap().unwrap(), RuntimeValue::I32(0));
	assert_eq!(module.execute(23, vec![]).unwrap().unwrap(), RuntimeValue::I32(1));
	assert_eq!(module.execute(24, vec![]).unwrap().unwrap(), RuntimeValue::I32(1));
	assert_eq!(module.execute(25, vec![]).unwrap().unwrap(), RuntimeValue::I32(0));
	assert_eq!(module.execute(26, vec![]).unwrap().unwrap(), RuntimeValue::I32(1));
	assert_eq!(module.execute(27, vec![]).unwrap().unwrap(), RuntimeValue::I32(1));
}

/// https://github.com/WebAssembly/wabt/blob/8e1f6031e9889ba770c7be4a9b084da5f14456a0/test/interp/compare.txt#L246
#[test]
fn compare_f32() {
	// f32 && f64 are serialized using binary32 && binary64 formats
	// http://babbage.cs.qc.cuny.edu/IEEE-754/
	let module = module()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F32Const(0xBF800000), // -1
				Opcode::F32Const(0xBF800000), // -1
				Opcode::F32Eq,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F32Const(0x3F800000), // 1
				Opcode::F32Const(0xBF800000), // -1
				Opcode::F32Eq,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F32Const(0x3F800000), // 1
				Opcode::F32Const(0xBF800000), // -1
				Opcode::F32Ne,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F32Const(0xBF800000), // -1
				Opcode::F32Const(0xBF800000), // -1
				Opcode::F32Ne,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F32Const(0xBF800000), // -1
				Opcode::F32Const(0x3F800000), // 1
				Opcode::F32Lt,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F32Const(0xBF800000), // -1
				Opcode::F32Const(0xBF800000), // -1
				Opcode::F32Lt,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F32Const(0x3F800000), // 1
				Opcode::F32Const(0xBF800000), // -1
				Opcode::F32Lt,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F32Const(0xBF800000), // -1
				Opcode::F32Const(0x3F800000), // 1
				Opcode::F32Le,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F32Const(0xBF800000), // -1
				Opcode::F32Const(0xBF800000), // -1
				Opcode::F32Le,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F32Const(0x3F800000), // 1
				Opcode::F32Const(0xBF800000), // -1
				Opcode::F32Le,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F32Const(0xBF800000), // -1
				Opcode::F32Const(0x3F800000), // 1
				Opcode::F32Gt,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F32Const(0xBF800000), // -1
				Opcode::F32Const(0xBF800000), // -1
				Opcode::F32Gt,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F32Const(0x3F800000), // 1
				Opcode::F32Const(0xBF800000), // -1
				Opcode::F32Gt,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F32Const(0xBF800000), // -1
				Opcode::F32Const(0x3F800000), // 1
				Opcode::F32Ge,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F32Const(0xBF800000), // -1
				Opcode::F32Const(0xBF800000), // -1
				Opcode::F32Ge,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F32Const(0x3F800000), // 1
				Opcode::F32Const(0xBF800000), // -1
				Opcode::F32Ge,
				Opcode::End,
			])).build()
			.build()
		.build();

	let program = ProgramInstance::new();
	let module = program.add_module("main", module).unwrap();
	assert_eq!(module.execute(0, vec![]).unwrap().unwrap(), RuntimeValue::I32(1));
	assert_eq!(module.execute(1, vec![]).unwrap().unwrap(), RuntimeValue::I32(0));
	assert_eq!(module.execute(2, vec![]).unwrap().unwrap(), RuntimeValue::I32(1));
	assert_eq!(module.execute(3, vec![]).unwrap().unwrap(), RuntimeValue::I32(0));
	assert_eq!(module.execute(4, vec![]).unwrap().unwrap(), RuntimeValue::I32(1));
	assert_eq!(module.execute(5, vec![]).unwrap().unwrap(), RuntimeValue::I32(0));
	assert_eq!(module.execute(6, vec![]).unwrap().unwrap(), RuntimeValue::I32(0));
	assert_eq!(module.execute(7, vec![]).unwrap().unwrap(), RuntimeValue::I32(1));
	assert_eq!(module.execute(8, vec![]).unwrap().unwrap(), RuntimeValue::I32(1));
	assert_eq!(module.execute(9, vec![]).unwrap().unwrap(), RuntimeValue::I32(0));
	assert_eq!(module.execute(10, vec![]).unwrap().unwrap(), RuntimeValue::I32(0));
	assert_eq!(module.execute(11, vec![]).unwrap().unwrap(), RuntimeValue::I32(0));
	assert_eq!(module.execute(12, vec![]).unwrap().unwrap(), RuntimeValue::I32(1));
	assert_eq!(module.execute(13, vec![]).unwrap().unwrap(), RuntimeValue::I32(0));
	assert_eq!(module.execute(14, vec![]).unwrap().unwrap(), RuntimeValue::I32(1));
	assert_eq!(module.execute(15, vec![]).unwrap().unwrap(), RuntimeValue::I32(1));
}

/// https://github.com/WebAssembly/wabt/blob/8e1f6031e9889ba770c7be4a9b084da5f14456a0/test/interp/compare.txt#L317
#[test]
fn compare_f64() {
	// f32 && f64 are serialized using binary32 && binary64 formats
	// http://babbage.cs.qc.cuny.edu/IEEE-754/
	let module = module()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F64Const(0xBFF0000000000000), // -1
				Opcode::F64Const(0xBFF0000000000000), // -1
				Opcode::F64Eq,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F64Const(0x3FF0000000000000), // 1
				Opcode::F64Const(0xBFF0000000000000), // -1
				Opcode::F64Eq,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F64Const(0x3FF0000000000000), // 1
				Opcode::F64Const(0xBFF0000000000000), // -1
				Opcode::F64Ne,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F64Const(0xBFF0000000000000), // -1
				Opcode::F64Const(0xBFF0000000000000), // -1
				Opcode::F64Ne,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F64Const(0xBFF0000000000000), // -1
				Opcode::F64Const(0x3FF0000000000000), // 1
				Opcode::F64Lt,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F64Const(0xBFF0000000000000), // -1
				Opcode::F64Const(0xBFF0000000000000), // -1
				Opcode::F64Lt,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F64Const(0x3FF0000000000000), // 1
				Opcode::F64Const(0xBFF0000000000000), // -1
				Opcode::F64Lt,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F64Const(0xBFF0000000000000), // -1
				Opcode::F64Const(0x3FF0000000000000), // 1
				Opcode::F64Le,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F64Const(0xBFF0000000000000), // -1
				Opcode::F64Const(0xBFF0000000000000), // -1
				Opcode::F64Le,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F64Const(0x3FF0000000000000), // 1
				Opcode::F64Const(0xBFF0000000000000), // -1
				Opcode::F64Le,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F64Const(0xBFF0000000000000), // -1
				Opcode::F64Const(0x3FF0000000000000), // 1
				Opcode::F64Gt,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F64Const(0xBFF0000000000000), // -1
				Opcode::F64Const(0xBFF0000000000000), // -1
				Opcode::F64Gt,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F64Const(0x3FF0000000000000), // 1
				Opcode::F64Const(0xBFF0000000000000), // -1
				Opcode::F64Gt,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F64Const(0xBFF0000000000000), // -1
				Opcode::F64Const(0x3FF0000000000000), // 1
				Opcode::F64Ge,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F64Const(0xBFF0000000000000), // -1
				Opcode::F64Const(0xBFF0000000000000), // -1
				Opcode::F64Ge,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F64Const(0x3FF0000000000000), // 1
				Opcode::F64Const(0xBFF0000000000000), // -1
				Opcode::F64Ge,
				Opcode::End,
			])).build()
			.build()
		.build();

	let program = ProgramInstance::new();
	let module = program.add_module("main", module).unwrap();
	assert_eq!(module.execute(0, vec![]).unwrap().unwrap(), RuntimeValue::I32(1));
	assert_eq!(module.execute(1, vec![]).unwrap().unwrap(), RuntimeValue::I32(0));
	assert_eq!(module.execute(2, vec![]).unwrap().unwrap(), RuntimeValue::I32(1));
	assert_eq!(module.execute(3, vec![]).unwrap().unwrap(), RuntimeValue::I32(0));
	assert_eq!(module.execute(4, vec![]).unwrap().unwrap(), RuntimeValue::I32(1));
	assert_eq!(module.execute(5, vec![]).unwrap().unwrap(), RuntimeValue::I32(0));
	assert_eq!(module.execute(6, vec![]).unwrap().unwrap(), RuntimeValue::I32(0));
	assert_eq!(module.execute(7, vec![]).unwrap().unwrap(), RuntimeValue::I32(1));
	assert_eq!(module.execute(8, vec![]).unwrap().unwrap(), RuntimeValue::I32(1));
	assert_eq!(module.execute(9, vec![]).unwrap().unwrap(), RuntimeValue::I32(0));
	assert_eq!(module.execute(10, vec![]).unwrap().unwrap(), RuntimeValue::I32(0));
	assert_eq!(module.execute(11, vec![]).unwrap().unwrap(), RuntimeValue::I32(0));
	assert_eq!(module.execute(12, vec![]).unwrap().unwrap(), RuntimeValue::I32(1));
	assert_eq!(module.execute(13, vec![]).unwrap().unwrap(), RuntimeValue::I32(0));
	assert_eq!(module.execute(14, vec![]).unwrap().unwrap(), RuntimeValue::I32(1));
	assert_eq!(module.execute(15, vec![]).unwrap().unwrap(), RuntimeValue::I32(1));
}

/// https://github.com/WebAssembly/wabt/blob/8e1f6031e9889ba770c7be4a9b084da5f14456a0/test/interp/convert.txt#L3
#[test]
fn convert_i32() {
	// f32 && f64 are serialized using binary32 && binary64 formats
	// http://babbage.cs.qc.cuny.edu/IEEE-754/
	let module = module()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I64Const(-1),
				Opcode::I32WarpI64,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F32Const(0xC2C83F35), // -100.12345
				Opcode::I32TruncSF32,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F32Const(0x4F32D05E), // 3e9
				Opcode::I32TruncUF32,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F64Const(0xC05907E69AD42C3D), // -100.12345
				Opcode::I32TruncSF64,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F64Const(0x41E65A0BC0000000), // 3e9
				Opcode::I32TruncUF64,
				Opcode::End,
			])).build()
			.build()
		.build();

	let program = ProgramInstance::new();
	let module = program.add_module("main", module).unwrap();
	assert_eq!(module.execute(0, vec![]).unwrap().unwrap(), RuntimeValue::I32(-1));				// 4294967295
	assert_eq!(module.execute(1, vec![]).unwrap().unwrap(), RuntimeValue::I32(-100));			// 4294967196
	assert_eq!(module.execute(2, vec![]).unwrap().unwrap(), RuntimeValue::I32(-1294967296));	// 3000000000
	assert_eq!(module.execute(3, vec![]).unwrap().unwrap(), RuntimeValue::I32(-100));			// 4294967196
	assert_eq!(module.execute(4, vec![]).unwrap().unwrap(), RuntimeValue::I32(-1294967296));	// 3000000000
}

/// https://github.com/WebAssembly/wabt/blob/8e1f6031e9889ba770c7be4a9b084da5f14456a0/test/interp/convert.txt#L21
#[test]
fn convert_i64() {
	// f32 && f64 are serialized using binary32 && binary64 formats
	// http://babbage.cs.qc.cuny.edu/IEEE-754/
	let module = module()
		.function()
			.signature().return_type().i64().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(-1),
				Opcode::I64ExtendUI32,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i64().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(-1),
				Opcode::I64ExtendSI32,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F32Const(0xC2C83F35), // -100.12345
				Opcode::I64TruncSF32,
				Opcode::I64Const(-100),
				Opcode::I64Eq,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F32Const(0x4F32D05E), // 3e9
				Opcode::I64TruncUF32,
				Opcode::I64Const(3000000000),
				Opcode::I64Eq,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F64Const(0xC05907E69AD42C3D), // -100.12345
				Opcode::I64TruncSF64,
				Opcode::I64Const(-100),
				Opcode::I64Eq,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F64Const(0x41E65A0BC0000000), // 3e9
				Opcode::I64TruncUF64,
				Opcode::I64Const(3000000000),
				Opcode::I64Eq,
				Opcode::End,
			])).build()
			.build()
		.build();

	let program = ProgramInstance::new();
	let module = program.add_module("main", module).unwrap();
	assert_eq!(module.execute(0, vec![]).unwrap().unwrap(), RuntimeValue::I64(4294967295));
	assert_eq!(module.execute(1, vec![]).unwrap().unwrap(), RuntimeValue::I64(-1)); // 18446744073709551615
	assert_eq!(module.execute(2, vec![]).unwrap().unwrap(), RuntimeValue::I32(1));
	assert_eq!(module.execute(3, vec![]).unwrap().unwrap(), RuntimeValue::I32(1));
	assert_eq!(module.execute(4, vec![]).unwrap().unwrap(), RuntimeValue::I32(1));
	assert_eq!(module.execute(5, vec![]).unwrap().unwrap(), RuntimeValue::I32(1));
}

/// https://github.com/WebAssembly/wabt/blob/8e1f6031e9889ba770c7be4a9b084da5f14456a0/test/interp/convert.txt#L50
#[test]
fn convert_f32() {
	// f32 && f64 are serialized using binary32 && binary64 formats
	// http://babbage.cs.qc.cuny.edu/IEEE-754/
	let module = module()
		.function()
			.signature().return_type().f32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(-1),
				Opcode::F32ConvertSI32,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().f32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(-1),
				Opcode::F32ConvertUI32,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().f32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F64Const(0x41678C29DCCCCCCD), // 12345678.9
				Opcode::F32DemoteF64,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().f32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I64Const(0),
				Opcode::F32ConvertSI64,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().f32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I64Const(0),
				Opcode::F32ConvertUI64,
				Opcode::End,
			])).build()
			.build()
		.build();

	let program = ProgramInstance::new();
	let module = program.add_module("main", module).unwrap();
	assert_eq!(module.execute(0, vec![]).unwrap().unwrap(), RuntimeValue::F32(-1.000000));
	assert_eq!(module.execute(1, vec![]).unwrap().unwrap(), RuntimeValue::F32(4294967296.000000));
	assert_eq!(module.execute(2, vec![]).unwrap().unwrap(), RuntimeValue::F32(12345679.000000));
	assert_eq!(module.execute(3, vec![]).unwrap().unwrap(), RuntimeValue::F32(0.000000));
	assert_eq!(module.execute(4, vec![]).unwrap().unwrap(), RuntimeValue::F32(0.000000));
}

/// https://github.com/WebAssembly/wabt/blob/8e1f6031e9889ba770c7be4a9b084da5f14456a0/test/interp/convert.txt#L50
#[test]
fn convert_f64() {
	// f32 && f64 are serialized using binary32 && binary64 formats
	// http://babbage.cs.qc.cuny.edu/IEEE-754/
	let module = module()
		.function()
			.signature().return_type().f64().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(-1),
				Opcode::F64ConvertSI32,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().f64().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(-1),
				Opcode::F64ConvertUI32,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().f64().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::F32Const(0x4B3C614F), // 12345678.9
				Opcode::F64PromoteF32,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().f64().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I64Const(0),
				Opcode::F64ConvertSI64,
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().f64().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I64Const(0),
				Opcode::F64ConvertUI64,
				Opcode::End,
			])).build()
			.build()
		.build();

	let program = ProgramInstance::new();
	let module = program.add_module("main", module).unwrap();
	assert_eq!(module.execute(0, vec![]).unwrap().unwrap(), RuntimeValue::F64(-1.000000));
	assert_eq!(module.execute(1, vec![]).unwrap().unwrap(), RuntimeValue::F64(4294967295.000000));
	assert_eq!(module.execute(2, vec![]).unwrap().unwrap(), RuntimeValue::F64(12345679.000000));
	assert_eq!(module.execute(3, vec![]).unwrap().unwrap(), RuntimeValue::F64(0.000000));
	assert_eq!(module.execute(4, vec![]).unwrap().unwrap(), RuntimeValue::F64(0.000000));
}

/// https://github.com/WebAssembly/wabt/blob/8e1f6031e9889ba770c7be4a9b084da5f14456a0/test/interp/load.txt#L9
#[test]
fn load_i32() {
	let module = module()
		.memory()
			.with_data(0, vec![0xff, 0xff, 0xff, 0xff, 0x00, 0x00, 0xce, 0x41,
				0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0x8f, 0x40,
				0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff])
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(0),
				Opcode::I32Load8S(0, 0),
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(0),
				Opcode::I32Load16S(0, 0),
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(0),
				Opcode::I32Load(0, 0),
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(0),
				Opcode::I32Load8U(0, 0),
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(0),
				Opcode::I32Load16U(0, 0),
				Opcode::End,
			])).build()
			.build()
		.build();

	let program = ProgramInstance::new();
	let module = program.add_module("main", module).unwrap();
	assert_eq!(module.execute(0, vec![]).unwrap().unwrap(), RuntimeValue::I32(-1));
	assert_eq!(module.execute(1, vec![]).unwrap().unwrap(), RuntimeValue::I32(-1));
	assert_eq!(module.execute(2, vec![]).unwrap().unwrap(), RuntimeValue::I32(-1));
	assert_eq!(module.execute(3, vec![]).unwrap().unwrap(), RuntimeValue::I32(255));
	assert_eq!(module.execute(4, vec![]).unwrap().unwrap(), RuntimeValue::I32(65535));
}

/// https://github.com/WebAssembly/wabt/blob/8e1f6031e9889ba770c7be4a9b084da5f14456a0/test/interp/load.txt#L26
#[test]
fn load_i64() {
	let module = module()
		.memory()
			.with_data(0, vec![0xff, 0xff, 0xff, 0xff, 0x00, 0x00, 0xce, 0x41,
				0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0x8f, 0x40,
				0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff])
			.build()
		.function()
			.signature().return_type().i64().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(0),
				Opcode::I64Load8S(0, 0),
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i64().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(0),
				Opcode::I64Load16S(0, 0),
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i64().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(0),
				Opcode::I64Load32S(0, 0),
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i64().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(16),
				Opcode::I64Load(0, 0),
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i64().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(0),
				Opcode::I64Load8U(0, 0),
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i64().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(0),
				Opcode::I64Load16U(0, 0),
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i64().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(0),
				Opcode::I64Load32U(0, 0),
				Opcode::End,
			])).build()
			.build()
		.build();

	let program = ProgramInstance::new();
	let module = program.add_module("main", module).unwrap();
	assert_eq!(module.execute(0, vec![]).unwrap().unwrap(), RuntimeValue::I64(-1));
	assert_eq!(module.execute(1, vec![]).unwrap().unwrap(), RuntimeValue::I64(-1));
	assert_eq!(module.execute(2, vec![]).unwrap().unwrap(), RuntimeValue::I64(-1));
	assert_eq!(module.execute(3, vec![]).unwrap().unwrap(), RuntimeValue::I64(-1));
	assert_eq!(module.execute(4, vec![]).unwrap().unwrap(), RuntimeValue::I64(255));
	assert_eq!(module.execute(5, vec![]).unwrap().unwrap(), RuntimeValue::I64(65535));
	assert_eq!(module.execute(6, vec![]).unwrap().unwrap(), RuntimeValue::I64(4294967295));
}