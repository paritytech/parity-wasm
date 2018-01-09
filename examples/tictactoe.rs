extern crate parity_wasm;

use std::env;
use std::fmt;
use parity_wasm::elements::{FunctionType, ValueType, TableType, GlobalType, MemoryType};
use parity_wasm::interpreter::{
	Error as InterpreterError, ModuleInstance, UserError, ModuleRef,
	HostFuncIndex, Externals, RuntimeValue, GlobalInstance, TableInstance, MemoryInstance,
	TableRef, MemoryRef, GlobalRef, FuncRef, TryInto, ImportResolver, FuncInstance,
	HostGlobalIndex, HostMemoryIndex, HostTableIndex,
};
use parity_wasm::elements::{Error as DeserializationError};
use parity_wasm::ValidationError;

#[derive(Debug)]
pub enum Error {
	OutOfRange,
	AlreadyOccupied,
	Interpreter(InterpreterError),
	Deserialize(DeserializationError),
	Validation(ValidationError),
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{:?}", self)
	}
}

impl From<InterpreterError> for Error {
	fn from(e: InterpreterError) -> Self {
		Error::Interpreter(e)
	}
}

impl From<DeserializationError> for Error {
	fn from(e: DeserializationError) -> Error {
		Error::Deserialize(e)
	}
}

impl From<ValidationError> for Error {
	fn from(e: ValidationError) -> Error {
		Error::Validation(e)
	}
}

impl UserError for Error {}

mod tictactoe {
	use super::Error;

	#[derive(Copy, Clone, Debug, PartialEq, Eq)]
	pub enum Player {
		X,
		O,
	}

	#[derive(Copy, Clone, Debug, PartialEq, Eq)]
	pub enum GameResult {
		Draw,
		Won(Player),
	}

	impl Player {
		pub fn into_i32(maybe_player: Option<Player>) -> i32 {
			match maybe_player {
				None => 0,
				Some(Player::X) => 1,
				Some(Player::O) => 2,
			}
		}
	}

	#[derive(Debug)]
	pub struct Game {
		board: [Option<Player>; 9],
	}

	impl Game {
		pub fn new() -> Game {
			Game {
				board: [None; 9],
			}
		}

		pub fn set(&mut self, idx: i32, player: Player) -> Result<(), Error> {
			if idx < 0 || idx > 9 {
				return Err(Error::OutOfRange);
			}
			if self.board[idx as usize] != None {
				return Err(Error::AlreadyOccupied);
			}
			self.board[idx as usize] = Some(player);
			Ok(())
		}

		pub fn get(&self, idx: i32) -> Result<Option<Player>, Error> {
			if idx < 0 || idx > 9 {
				return Err(Error::OutOfRange);
			}
			Ok(self.board[idx as usize])
		}

		pub fn game_result(&self) -> Option<GameResult> {
			// 0, 1, 2
			// 3, 4, 5
			// 6, 7, 8
			let patterns = &[
				// Rows
				(0, 1, 2),
				(3, 4, 5),
				(6, 7, 8),

				// Columns
				(0, 3, 6),
				(1, 4, 7),
				(2, 5, 8),

				// Diagonals
				(0, 4, 8),
				(2, 4, 6),
			];

			// Returns Some(player) if all cells contain same Player.
			let all_same = |i1: usize, i2: usize, i3: usize| -> Option<Player> {
				if self.board[i1].is_none() {
					return None;
				}
				if self.board[i1] == self.board[i2] && self.board[i2] == self.board[i3] {
					return self.board[i1];
				}
				None
			};

			for &(i1, i2, i3) in patterns {
				if let Some(player) = all_same(i1, i2, i3) {
					return Some(GameResult::Won(player));
				}
			}

			// Ok, there is no winner. Check if it's draw.
			let all_occupied = self.board.iter().all(|&cell| cell.is_some());
			if all_occupied {
				Some(GameResult::Draw)
			} else {
				// Nah, there are still empty cells left.
				None
			}
		}
	}
}

struct Runtime<'a> {
	player: tictactoe::Player,
	game: &'a mut tictactoe::Game,
}

const SET_FUNC_INDEX: HostFuncIndex = 0;
const GET_FUNC_INDEX: HostFuncIndex = 1;

impl<'a> Externals for Runtime<'a> {
	fn invoke_index(
		&mut self,
		index: HostFuncIndex,
		args: &[RuntimeValue],
	) -> Result<Option<RuntimeValue>, InterpreterError> {
		match index {
			SET_FUNC_INDEX => {
				let idx: i32 = args[0].try_into().unwrap();
				self.game.set(idx, self.player)?;
				Ok(None)
			}
			GET_FUNC_INDEX => {
				let idx: i32 = args[0].try_into().unwrap();
				let val: i32 = tictactoe::Player::into_i32(self.game.get(idx)?);
				Ok(Some(val.into()))
			}
			_ => panic!("unknown function index")
		}
	}

	fn check_signature(&self, index: HostFuncIndex, sig: &FunctionType) -> bool {
		match index {
			SET_FUNC_INDEX => {
				sig.params() == &[ValueType::I32] && sig.return_type() == None
			}
			GET_FUNC_INDEX => {
				sig.params() == &[ValueType::I32] && sig.return_type() == Some(ValueType::I32)
			}
			_ => panic!("unknown function index")
		}
	}
}

struct RuntimeImportResolver;

impl<'a> ImportResolver for RuntimeImportResolver {
	fn resolve_func(
		&self,
		field_name: &str,
		_func_type: &FunctionType,
	) -> Result<FuncRef, InterpreterError> {
		let func_ref = match field_name {
			"set" => {
				FuncInstance::alloc_host(FunctionType::new(vec![ValueType::I32], None), SET_FUNC_INDEX)
			},
			"get" => FuncInstance::alloc_host(FunctionType::new(vec![ValueType::I32], Some(ValueType::I32)), GET_FUNC_INDEX),
			_ => return Err(
				InterpreterError::Function(
					format!("host module doesn't export function with name {}", field_name)
				)
			)
		};
		Ok(func_ref)
	}

	fn resolve_global(
		&self,
		_field_name: &str,
		_global_type: &GlobalType,
	) -> Result<GlobalRef, InterpreterError> {
		Err(
			InterpreterError::Global("host module doesn't export any globals".to_owned())
		)
	}

	fn resolve_memory(
		&self,
		_field_name: &str,
		_memory_type: &MemoryType,
	) -> Result<MemoryRef, InterpreterError> {
		Err(
			InterpreterError::Global("host module doesn't export any memories".to_owned())
		)
	}

	fn resolve_table(
		&self,
		_field_name: &str,
		_table_type: &TableType,
	) -> Result<TableRef, InterpreterError> {
		Err(
			InterpreterError::Global("host module doesn't export any tables".to_owned())
		)
	}
}

fn instantiate(
	path: &str,
) -> Result<ModuleRef, Error> {
	let module = parity_wasm::deserialize_file(path)?;
	let validated_module = parity_wasm::validate_module(module)?;

	let instance = ModuleInstance::new(&validated_module)
		.with_import("env", &RuntimeImportResolver)
		.assert_no_start()?;

	Ok(instance)
}

fn play(
	x_instance: ModuleRef,
	o_instance: ModuleRef,
	game: &mut tictactoe::Game,
) -> Result<tictactoe::GameResult, Error> {
	let mut turn_of = tictactoe::Player::X;
	let game_result = loop {
		let (instance, next_turn_of) = match turn_of {
			tictactoe::Player::X => (&x_instance, tictactoe::Player::O),
			tictactoe::Player::O => (&o_instance, tictactoe::Player::X),
		};

		{
			let mut runtime = Runtime {
				player: turn_of,
				game: game,
			};
			let _ = instance.invoke_export("mk_turn", &[], &mut runtime)?;
		}

		match game.game_result() {
			Some(game_result) => break game_result,
			None => {}
		}

		turn_of = next_turn_of;
	};

	Ok(game_result)
}

fn main() {
	let mut game = tictactoe::Game::new();

	let args: Vec<_> = env::args().collect();
	if args.len() < 3 {
		println!("Usage: {} <x player module> <y player module>", args[0]);
		return;
	}

	// Instantiate modules of X and O players.
	let x_instance = instantiate(&args[1]).expect("X player module to load");
	let o_instance = instantiate(&args[2]).expect("Y player module to load");

	let result = play(x_instance, o_instance, &mut game);
	println!("result = {:?}, game = {:#?}", result, game);
}
