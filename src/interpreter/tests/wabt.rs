///! Tests from https://github.com/WebAssembly/wabt/tree/8e1f6031e9889ba770c7be4a9b084da5f14456a0/test/interp

use std::sync::Weak;
use elements::{Module, ValueType, Opcodes, Opcode, BlockType, FunctionType};
use interpreter::Error;
use interpreter::module::ModuleInstance;
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
	// TODO: linear memory required
}


















/*

	#[test]
	fn basics_loop() {
		let body = Opcodes::new(vec![
			Opcode::I32Const(2),									// [2]
			Opcode::SetLocal(1),									// [] + local1 = 2
			Opcode::Block(BlockType::NoResult,						// add block label to exit from loop. TODO: is it the correct pattern?
				Opcodes::new(vec![
					Opcode::Loop(BlockType::NoResult,				//   start loop
						Opcodes::new(vec![
							Opcode::GetLocal(0),					//    [local0]
							Opcode::I32Const(1),					//    [local0, 1]
							Opcode::I32Sub,							//    [local0 - 1]
							Opcode::SetLocal(0),					//    [] + local0 = local0 - 1
							Opcode::GetLocal(0),					//    [local0]
							Opcode::If(BlockType::NoResult,			//    if local0 != 0
								Opcodes::new(vec![
									Opcode::GetLocal(1),			//      [local1]
									Opcode::I32Const(2),			//      [local1, 2]
									Opcode::I32Mul,					//      [local1 * 2]
									Opcode::SetLocal(1),			//      [] + local1 = local1 * 2
									Opcode::Else,					//    else
									Opcode::Br(2),					//      exit from loop (2 = if + loop)
									Opcode::End,					//    end (if)
								])),
							Opcode::End,							//   end (loop)
						])),
					Opcode::End,									// end (block)
				])),
			Opcode::GetLocal(1),									// [local1]
			Opcode::End]);											// end (fun)

		assert_eq!(run_function_i32(&body, 2).unwrap(), 4);
		assert_eq!(run_function_i32(&body, 8).unwrap(), 256);
	}


	#[test]
	fn basics_if_then() {
		let body = Opcodes::new(vec![
			Opcode::I32Const(20),							// 20
			Opcode::GetLocal(0),							// read argument
			Opcode::If(BlockType::Value(ValueType::I32),	// if argument != 0
				Opcodes::new(vec![
					Opcode::I32Const(10),					//  10
					Opcode::End,							// end
				])),
			Opcode::End]);

		assert_eq!(run_function_i32(&body, 0).unwrap(), 20);
		assert_eq!(run_function_i32(&body, 1).unwrap(), 10);
	}

	#[test]
	fn basics_if_then_else() {
		let body = Opcodes::new(vec![
			Opcode::GetLocal(0),							// read argument
			Opcode::If(BlockType::Value(ValueType::I32),	// if argument != 0
				Opcodes::new(vec![
					Opcode::I32Const(10),					//  10
					Opcode::Else,							// else
					Opcode::I32Const(20),					//  20
					Opcode::End,							// end
				])),
			Opcode::End]);

		assert_eq!(run_function_i32(&body, 0).unwrap(), 20);
		assert_eq!(run_function_i32(&body, 1).unwrap(), 10);
	}

	#[test]
	fn basics_return() {
		let body = Opcodes::new(vec![
			Opcode::GetLocal(0),							// read argument
			Opcode::If(BlockType::Value(ValueType::I32),	// if argument != 0
				Opcodes::new(vec![
					Opcode::I32Const(20),					//  20
					Opcode::Return,							//  return
					Opcode::End,
				])),
			Opcode::I32Const(10),							// 10
			Opcode::End]);

		assert_eq!(run_function_i32(&body, 0).unwrap(), 10);
		assert_eq!(run_function_i32(&body, 1).unwrap(), 20);
	}

	#[test]
	fn branch_if() {
		// TODO
	}

	#[test]
	fn branch_table() {
		// TODO
	}

	#[test]
	fn drop() {
		// TODO
	}

	#[test]
	fn select() {
		// TODO
	}*/
