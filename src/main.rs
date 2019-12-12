extern crate termios;

use termios::*;
use std::io::{self, Read};
use std::path::Path;
use std::fs::{File};
use std::env;

macro_rules! usize {
	( $( $x:expr ),* ) => {
			{
				$(
						usize::from($x)
				)*
			}
	};
}

enum Registers {
	R_R0 = 0,
	R_R1,
	R_R2,
	R_R3,
	R_R4,
	R_R5,
	R_R6,
	R_R7,
	R_PC, /* program counter */
	R_COND,
	R_COUNT
}

enum Instructions {
	BR = 0, /* branch */
	ADD,    /* add  */
	LD,     /* load */
	ST,     /* store */
	JSR,    /* jump register */
	AND,    /* bitwise and */
	LDR,    /* load register */
	STR,    /* store register */
	RTI,    /* unused */
	NOT,    /* bitwise not */
	LDI,    /* load indirect */
	STI,    /* store indirect */
	JMP,    /* jump */
	XOR,    /* reserved (unused) */
	LEA,    /* load effective address */
	TRAP    /* execute trap */
}

enum Conditions {
	FL_POS = 1 << 0, /* P */
	FL_ZRO = 1 << 1, /* Z */
	FL_NEG = 1 << 2, /* N */
}

enum MemoryMappedRegisters {
	MR_KBSR = 0xFE00, /* keyboard status */
	MR_KBDR = 0xFE02,  /* keyboard data */
	R_ZERO = 0xFFFF /* special zero register */
}

enum TrapCodes {
	TRAP_GETC = 0x20,  /* get character from keyboard */
	TRAP_OUT = 0x21,   /* output a character */
	TRAP_PUTS = 0x22,  /* output a word string */
	TRAP_IN = 0x23,    /* input a string */
	TRAP_PUTSP = 0x24, /* output a byte string */
	TRAP_HALT = 0x25   /* halt the program */
}

fn sign_extend(x: u16, bit_count: i32) -> u16 {
	if (x >> (bit_count - 1)) & 1 != 0 {
		return x | (0xFFFF << bit_count);
	}
	x
}

fn mem_read(mem_ref: &mut [u16], address: u16) -> u16 {
	if address == MemoryMappedRegisters::MR_KBSR as u16 {
		let mut buffer = [0; 1];
		std::io::stdin().read_exact(&mut buffer).unwrap();
		if buffer[0] != 0 {
			mem_ref[MemoryMappedRegisters::MR_KBSR as usize] = 1 << 15;
			mem_ref[MemoryMappedRegisters::MR_KBDR as usize] = buffer[0] as u16;
		} else {
			mem_ref[MemoryMappedRegisters::MR_KBSR as usize] = 0;
		}
	}
	if address == MemoryMappedRegisters::R_ZERO as u16 {
		mem_ref[MemoryMappedRegisters::R_ZERO as usize] = 0;
	}
	mem_ref[address as usize]
}

fn mem_write(mem_ref: &mut [u16], address: usize, val: u16) {
	// if address == MemoryMappedRegisters::R_ZERO as usize {
	// 	mem_ref[address] = 0; /* zero register */
	// } else { 
	// 	mem_ref[address] = val; 
	// }
	mem_ref[address] = val;
}

fn update_flags(reg_ref: &mut [u16], r: usize) {
	reg_ref[Registers::R_COND as usize] = match reg_ref[r] {
		0                 => Conditions::FL_ZRO as u16,
		r if r >> 15 == 1 => Conditions::FL_NEG as u16,
		_                 => Conditions::FL_POS as u16
	}
}

fn swap16(x: u16) -> u16 {
  x << 8 | x >> 8
}

fn get16(mem: &[u8], ind: usize) -> u16 {
	((mem[ind] as u16) << 8) + mem[ind+1] as u16
}

fn read_image(mem: &mut [u16], image_path: &str) -> u32 {
	let path = Path::new(image_path);
	println!("[*] Loading {}", path.to_str().unwrap());
	let mut file = File::open(&path).expect("Couldn't open file.");

	const SIZE: u32 = std::u16::MAX as u32 * 2 - 2;
	let mut mem_buffer: [u8; SIZE as usize] = [0; SIZE as usize];
	file.read(&mut mem_buffer).expect("Couldn't read file.");
	let length = file.metadata().unwrap().len();
	println!("[*] File length {}", length);

	let base = get16(&mem_buffer, 0) as usize;
	for i in (2..length).step_by(2) {
			println!("{}",i);
			mem[base+(i/2 - 1) as usize] = get16(&mem_buffer, i as usize);
	}
	println!("{:?}", &mem[0x3000..0x4000]);
	length as u32
}

fn run(args: &Vec<String>) {
	let mut memory: [u16; std::u16::MAX as usize] = [0; std::u16::MAX as usize];
	let mut registers: [u16; Registers::R_COUNT as usize] = [0; Registers::R_COUNT as usize];

	for image in args.iter().skip(1) {
		read_image(&mut memory, image);
	}

	/* set the PC to starting position */
	/* 0x3000 is the default */
	let pc_start = 0x3000;
	
	registers[Registers::R_PC as usize] = pc_start;

	'runtime: loop {
		let instr = mem_read(&mut memory, registers[Registers::R_PC as usize]);
		registers[Registers::R_PC as usize] += 1;
		let op: u16 = instr >> 12;
		match op {
			// BRANCH
			_ if op == Instructions::BR as u16  => {
				let pc_offset = sign_extend((instr) & 0x1ff, 9);
				let cond_flag = (instr >> 9) & 0x7;
				if (cond_flag & registers[Registers::R_COND as usize]) != 0 {
						registers[Registers::R_PC as usize] += pc_offset;
				}
			},

			// ADD
			_ if op == Instructions::ADD as u16  => {
				/* destination register (DR) */
				let r0 = usize!(instr >> 9 & 0x7);
				/* first operand (SR1) */
				let r1 = usize!(instr >> 6 & 0x7);
				/* whether we are in immediate mode */
				let imm_flag: u16 = instr >> 5 & 0x1;

				if imm_flag == 1 {
					let imm5 = sign_extend(instr & 0x1F, 5);
					registers[r0] = registers[r1] + imm5;
				} else {
					let r2 = usize!(instr & 0x7);
					registers[r0] = registers[r1] + registers[r2];
				}

				update_flags(&mut registers, r0);
			},

			// LOAD
			_ if op == Instructions::LD as u16  => {
				let r0 = usize!(instr >> 9 & 0x7);
				let pc_offset = sign_extend(instr & 0x1ff, 9);
				registers[r0] = mem_read(&mut memory, registers[Registers::R_PC as usize] + pc_offset);
				update_flags(&mut registers, r0);
			},

			// STORE
			_ if op == Instructions::ST as u16  => {
				let r0 = usize!(instr >> 9 & 0x7);
				let pc_offset = sign_extend(instr & 0x1ff, 9);
				mem_write(&mut memory, usize!(registers[Registers::R_PC as usize] + pc_offset), registers[r0]);
			},

			// JUMP REGISTER
			_ if op == Instructions::JSR as u16  => {
				let r1 = usize!(instr >> 6 & 0x7);
				let long_pc_offset = sign_extend(instr & 0x7ff, 11);
				let long_flag = (instr >> 11) & 1;
		
				registers[Registers::R_R7 as usize] = registers[Registers::R_PC as usize];
				if long_flag != 0 {
						registers[Registers::R_PC as usize] += long_pc_offset;  /* JSR */
				} else {
						registers[Registers::R_PC as usize] = registers[r1]; /* JSRR */
				}
			},

			// BITWISE AND
			_ if op == Instructions::AND as u16  => {
				let r0 = usize!(instr >> 9 & 0x7);
				let r1 = usize!(instr >> 6 & 0x7);
				let imm_flag = (instr >> 5) & 0x1;
		
				if imm_flag == 1 {
						let imm5 = sign_extend(instr & 0x1F, 5);
						registers[r0] = registers[r1] & imm5;
				}	else {
						let r2 = usize!(instr & 0x7);
						registers[r0] = registers[r1] & registers[r2];
				}
				update_flags(&mut memory, r0);
			},

			// LOAD REGISTER
			_ if op == Instructions::LDR as u16  => {
				let r0 = usize!(instr >> 9 & 0x7);
				let r1 = usize!(instr >> 6 & 0x7);
				let offset = sign_extend(instr & 0x3F, 6);
				registers[r0] = mem_read(&mut memory, registers[r1] + offset);
				update_flags(&mut registers, r0);
			},

			// STORE REGISTER
			_ if op == Instructions::STR as u16  => {
				let r0 = usize!(instr >> 9 & 0x7);
				let r1 = usize!(instr >> 6 & 0x7);
				let offset = sign_extend(instr & 0x3F, 6);
				mem_write(&mut memory, usize!(registers[r1] + offset), registers[r0]);
			},

			_ if op == Instructions::RTI as u16  => {
				std::process::exit(1);
			},

			// BITWISE NOT
			_ if op == Instructions::NOT as u16  => {
					let r0 = usize!(instr >> 9 & 0x7);
					let r1 = usize!(instr >> 6 & 0x7);
			
					registers[r0] = !registers[r1];
					update_flags(&mut registers, r0);
			},

			// LOAD INDIRECT
			_ if op == Instructions::LDI as u16  => {
				let r0 = usize!(instr >> 9 & 0x7);
				/* PCoffset 9*/
				let pc_offset = sign_extend(instr & 0x1ff, 9);
				/* add pc_offset to the current PC, look at that memory location to get the final address */
				let preload = mem_read(&mut memory, registers[Registers::R_PC as usize] + pc_offset);
				registers[r0] = mem_read(&mut memory, preload);
				update_flags(&mut registers, r0);
			},

			// STORE INDIRECT
			_ if op == Instructions::STI as u16  => {
				let r0 = usize!(instr >> 9 & 0x7);
				let pc_offset = sign_extend(instr & 0x1ff, 9);
				let mem_read_result = usize!(mem_read(&mut memory, registers[Registers::R_PC as usize] + pc_offset));
				mem_write(&mut memory, mem_read_result, registers[r0]);
			},

			// JUMP
			_ if op == Instructions::JMP as u16  => {
				/* Also handles RET */
				let r1 = usize!(instr >> 6 & 0x7);
				registers[Registers::R_PC as usize] = registers[r1];
			},

			// XOR
			_ if op == Instructions::XOR as u16  => {
				let r0 = usize!(instr >> 9 & 0x7);
				let r1 = usize!(instr >> 6 & 0x7);
				registers[r0] = registers[r0] ^ registers[r1];
				update_flags(&mut registers, r0);
			},

			// LOAD EFFECTIVE ADDRESS
			_ if op == Instructions::LEA as u16  => {
				let r0 = usize!(instr >> 9 & 0x7);
				let pc_offset = sign_extend(instr & 0x1ff, 9);
				registers[r0] = registers[Registers::R_PC as usize] + pc_offset;
				update_flags(&mut registers, r0);
			},

			// TRAP
			_ if op == Instructions::TRAP as u16 => {
				let action = instr & 0xFF;
				match action {
					_ if action == TrapCodes::TRAP_GETC as u16 =>  {
						let mut buffer = [0; 1];
						std::io::stdin().read_exact(&mut buffer).unwrap();
						registers[Registers::R_R0 as usize] = buffer[0] as u16;
					},
					_ if action == TrapCodes::TRAP_HALT as u16 => {
						println!("[!] HALT");
						break 'runtime;
					},
					_ if action == TrapCodes::TRAP_IN as u16 => {
						print!("Enter a character: ");
						registers[Registers::R_R0 as usize] = std::io::stdin()
								.bytes()
								.next()
								.and_then(|result| result.ok())
								.map(|byte| byte as u16)
								.unwrap();
					},
					_ if action == TrapCodes::TRAP_OUT as u16 => {
						let c = registers[Registers::R_R0 as usize] as u8;
            print!("{}", c as char);
					},
					_ if action == TrapCodes::TRAP_PUTS as u16 => {
						for c in &memory[registers[Registers::R_R0 as usize] as usize..] {
							let c8 = (c & 0xff) as u8;
							if c8 != 0x00 {
									print!("{}", c8 as char);
							} else {
									break;
							}
						}
					},
					_ if action == TrapCodes::TRAP_PUTSP as u16 => {
						for c in &memory[registers[Registers::R_R0 as usize] as usize..] {
							let b1 = (*c >> 8) as u8;
							let b2 = (*c & 0xff) as u8;
							if b1!= 0 {
									print!("{}", b1 as char);
									if b2 != 0 {
											print!("{}", b2 as char);
									}
							}
						}
					},
					_ => {
						println!("UNKNOW TRAPCODE");
						std::process::exit(1);
					}
				}
			},

			_ => {
				println!("UNKNOW OPCODE");
				std::process::exit(1);
			},
		}
	}
}

fn main() {
	let stdin = 0;

	let termios = Termios::from_fd(stdin).unwrap();
	let mut new_termios = termios.clone();  // make a mutable copy of termios 
																					// that we will modify
	new_termios.c_iflag &= IGNBRK | BRKINT | PARMRK | ISTRIP | INLCR | IGNCR | ICRNL | IXON;
	new_termios.c_lflag &= !(ICANON | ECHO); // no echo and canonical mode
	tcsetattr(stdin, TCSANOW, &mut new_termios).unwrap();

	let args: Vec<String> = env::args().collect();

	if args.len() < 2 {
    /* show usage string */
    println!("lc3 [image-file1] ...\n");
    std::process::exit(2);
	}

	run(&args);

	tcsetattr(stdin, TCSANOW, & termios).unwrap();

}