use super::instructions::Instructions;
use super::memory::{mem_read, read_image, Memory};
use super::registers::Registers;

const PC_START: u16 = 0x3000;

pub fn run(args: &Vec<String>) {
  let mut memory = Memory::init();
  let mut registers = Registers::init();

  for image in args.iter().skip(1) {
    read_image(&mut memory, image);
  }

  registers[Registers::PC as usize] = PC_START;

  let mut flag = true;

  //'runtime: loop
  while flag {
    let instr = mem_read(&mut memory, registers[Registers::PC as usize]);
    registers[Registers::PC as usize] += 1;
    let op: u16 = instr >> 12;
    match op {
      _ if op == Instructions::BR as u16 => Instructions::branch(&mut registers, instr),
      _ if op == Instructions::ADD as u16 => Instructions::add(&mut registers, instr),
      _ if op == Instructions::LD as u16 => Instructions::load(&mut registers, instr, &mut memory),
      _ if op == Instructions::ST as u16 => Instructions::store(&mut registers, instr, &mut memory),
      _ if op == Instructions::JSR as u16 => Instructions::jump_register(&mut registers, instr),
      _ if op == Instructions::AND as u16 => Instructions::and(&mut registers, instr, &mut memory),
      _ if op == Instructions::LDR as u16 => {
        Instructions::load_indirect(&mut registers, instr, &mut memory)
      }
      _ if op == Instructions::STR as u16 => {
        Instructions::store_registers(&mut registers, instr, &mut memory)
      }
      _ if op == Instructions::RTI as u16 => Instructions::rti(),
      _ if op == Instructions::NOT as u16 => Instructions::bitwise_not(&mut registers, instr),
      _ if op == Instructions::LDI as u16 => {
        Instructions::load_indirect(&mut registers, instr, &mut memory)
      }
      _ if op == Instructions::STI as u16 => {
        Instructions::store_indirect(&mut registers, instr, &mut memory)
      }
      _ if op == Instructions::JMP as u16 => Instructions::jump(&mut registers, instr),
      _ if op == Instructions::XOR as u16 => Instructions::xor(&mut registers, instr),
      _ if op == Instructions::LEA as u16 => Instructions::effective_address(&mut registers, instr),
      _ if op == Instructions::TRAP as u16 => {
        Instructions::trap(&mut registers, instr, &mut memory, &mut flag)
      }
      _ => {
        println!("UNKNOW OPCODE");
        std::process::exit(1);
      }
    }
  }
}
