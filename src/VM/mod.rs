extern crate termios;

#[macro_use]
pub mod lib;
pub mod executor;
pub mod instructions;
pub mod memory;
pub mod registers;
pub mod sys;

use memory::{Memory, MemoryBase};
use registers::{Registers, RegistersBase};
use std::env;

pub struct VM {
	memory: MemoryBase,
	registers: RegistersBase,
	// GC: Option<unimplemented!()>,
}

impl VM {
	pub fn init() -> Self {
		VM {
			memory: Memory::init(),
			registers: Registers::init(),
		}
	}

	pub fn run(&self) {
		let sys = sys::Sys::init();
		sys.set_flags();
		let args: Vec<String> = env::args().collect();
		if args.len() < 2 {
			println!("lc3 [image-file1] ...\n");
			std::process::exit(2);
		}
		executor::run(&args);
		sys.set_terminal();
	}
}
