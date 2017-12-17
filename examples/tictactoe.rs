extern crate parity_wasm;

use std::env;
use std::fmt;
use std::rc::Rc;
use parity_wasm::elements::Module;
use parity_wasm::interpreter::{Error as InterpreterError, HostModule, HostModuleBuilder,
							   ModuleInstance, UserError};

#[derive(Debug)]
pub enum Error {
	OutOfRange,
	AlreadyOccupied,
	Interpreter(InterpreterError),
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

impl UserError for Error {}

mod tictactoe {
	use std::cell::RefCell;
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
		board: RefCell<[Option<Player>; 9]>,
	}

	impl Game {
		pub fn new() -> Game {
			Game {
				board: RefCell::new([None; 9]),
			}
		}

		pub fn set(&self, idx: i32, player: Player) -> Result<(), Error> {
			let mut board = self.board.borrow_mut();
			if idx < 0 || idx > 9 {
				return Err(Error::OutOfRange);
			}
			if board[idx as usize] != None {
				return Err(Error::AlreadyOccupied);
			}
			board[idx as usize] = Some(player);
			Ok(())
		}

		pub fn get(&self, idx: i32) -> Result<Option<Player>, Error> {
			if idx < 0 || idx > 9 {
				return Err(Error::OutOfRange);
			}
			Ok(self.board.borrow()[idx as usize])
		}

		pub fn game_result(&self) -> Option<GameResult> {
			let board = self.board.borrow();

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
				if board[i1].is_none() {
					return None;
				}
				if board[i1] == board[i2] && board[i2] == board[i3] {
					return board[i1];
				}
				None
			};

			for &(i1, i2, i3) in patterns {
				if let Some(player) = all_same(i1, i2, i3) {
					return Some(GameResult::Won(player));
				}
			}

			// Ok, there is no winner. Check if it's draw.
			let all_occupied = board.iter().all(|&cell| cell.is_some());
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
	game: &'a tictactoe::Game,
}

fn instantiate<'a, 'b>(
	module: &Module,
	env: &HostModule<Runtime<'a>>,
	runtime: &'b Runtime<'a>,
) -> Result<Rc<ModuleInstance<Runtime<'a>>>, Error> {
	let instance = ModuleInstance::instantiate(module)
		.with_import("env", &*env)
		.run_start(runtime)?;

	Ok(instance)
}

fn env_host_module<'a>() -> HostModule<Runtime<'a>> {
	let mut builder = HostModuleBuilder::<Runtime>::new();
	builder.with_func1(
		"set",
		|state: &Runtime, idx: i32| -> Result<Option<()>, InterpreterError> {
			state.game.set(idx, state.player)?;
			Ok(None)
		},
	);
	builder.with_func1(
		"get",
		|state: &Runtime, idx: i32| -> Result<Option<i32>, InterpreterError> {
			let val: i32 = tictactoe::Player::into_i32(state.game.get(idx).unwrap());
			Ok(Some(val))
		},
	);
	builder.build()
}

fn play<'a>(
	x_module: &Module,
	o_module: &Module,
	host_module: &HostModule<Runtime<'a>>,
	game: &'a tictactoe::Game,
) -> Result<tictactoe::GameResult, Error> {
	// Instantiate modules of X and O players.
	let x_instance = {
		let mut runtime = Runtime {
			player: tictactoe::Player::X,
			game: game,
		};
		instantiate(x_module, host_module, &runtime)?
	};
	let o_instance = {
		let mut runtime = Runtime {
			player: tictactoe::Player::O,
			game: game,
		};
		instantiate(o_module, host_module, &runtime)?
	};

	let mut turn_of = tictactoe::Player::X;
	let game_result = loop {
		let (instance, next_turn_of) = match turn_of {
			tictactoe::Player::X => (&x_instance, tictactoe::Player::O),
			tictactoe::Player::O => (&o_instance, tictactoe::Player::X),
		};

		let mut runtime = Runtime {
			player: turn_of,
			game: game,
		};
		let _ = instance.invoke_export("mk_turn", vec![], &runtime)?;

		match game.game_result() {
			Some(game_result) => break game_result,
			None => {}
		}

		turn_of = next_turn_of;
	};

	Ok(game_result)
}

fn main() {
	let game = tictactoe::Game::new();
	let env_host_module = env_host_module();

	let args: Vec<_> = env::args().collect();
	if args.len() < 3 {
		println!("Usage: {} <x player module> <y player module>", args[0]);
		return;
	}
	let x_module = parity_wasm::deserialize_file(&args[1]).expect("X player module to load");
	let o_module = parity_wasm::deserialize_file(&args[2]).expect("Y player module to load");

	let result = play(&x_module, &o_module, &env_host_module, &game);
	println!("result = {:?}, game = {:#?}", result, game);
}
