use super::lib::{sign_extend, update_flags};
use super::memory::*;
use super::registers::{Registers, RegistersBase};

use std::io::Read;

pub enum Instructions {
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
  TRAP,   /* execute trap */
}

pub enum TrapCodes {
  TRAP_GETC = 0x20,  /* get character from keyboard */
  TRAP_OUT = 0x21,   /* output a character */
  TRAP_PUTS = 0x22,  /* output a word string */
  TRAP_IN = 0x23,    /* input a string */
  TRAP_PUTSP = 0x24, /* output a byte string */
  TRAP_HALT = 0x25,  /* halt the program */
}

impl Instructions {
  pub fn branch(registers: &mut RegistersBase, instr: u16) {
    let pc_offset = sign_extend((instr) & 0x1ff, 9);
    let cond_flag = (instr >> 9) & 0x7;
    if (cond_flag & registers[Registers::COND as usize]) != 0 {
      registers[Registers::PC as usize] += pc_offset;
    }
  }

  pub fn add(registers: &mut RegistersBase, instr: u16) {
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

    update_flags(registers, r0);
  }

  // LOAD
  pub fn load(registers: &mut RegistersBase, instr: u16, memory: &mut MemoryBase) {
    let r0 = usize!(instr >> 9 & 0x7);
    let pc_offset = sign_extend(instr & 0x1ff, 9);
    registers[r0] = mem_read(memory, registers[Registers::PC as usize] + pc_offset);
    update_flags(registers, r0);
  }

  // STORE
  pub fn store(registers: &mut RegistersBase, instr: u16, memory: &mut MemoryBase) {
    let r0 = usize!(instr >> 9 & 0x7);
    let pc_offset = sign_extend(instr & 0x1ff, 9);
    mem_write(
      memory,
      usize!(registers[Registers::PC as usize] + pc_offset),
      registers[r0],
    );
  }

  // JUMP REGISTER
  pub fn jump_register(registers: &mut RegistersBase, instr: u16) {
    let r1 = usize!(instr >> 6 & 0x7);
    let long_pc_offset = sign_extend(instr & 0x7ff, 11);
    let long_flag = (instr >> 11) & 1;
    registers[Registers::R7 as usize] = registers[Registers::PC as usize];
    if long_flag != 0 {
      registers[Registers::PC as usize] += long_pc_offset; /* JSR */
    } else {
      registers[Registers::PC as usize] = registers[r1]; /* JSRR */
    }
  }

  // BITWISE AND
  pub fn and(registers: &mut RegistersBase, instr: u16, memory: &mut MemoryBase) {
    let r0 = usize!(instr >> 9 & 0x7);
    let r1 = usize!(instr >> 6 & 0x7);
    let imm_flag = (instr >> 5) & 0x1;
    if imm_flag == 1 {
      let imm5 = sign_extend(instr & 0x1F, 5);
      registers[r0] = registers[r1] & imm5;
    } else {
      let r2 = usize!(instr & 0x7);
      registers[r0] = registers[r1] & registers[r2];
    }
    update_flags(memory, r0);
  }

  // LOAD REGISTER
  pub fn load_registers(registers: &mut RegistersBase, instr: u16, memory: &mut MemoryBase) {
    let r0 = usize!(instr >> 9 & 0x7);
    let r1 = usize!(instr >> 6 & 0x7);
    let offset = sign_extend(instr & 0x3F, 6);
    registers[r0] = mem_read(memory, registers[r1] + offset);
    update_flags(registers, r0);
  }

  // STORE REGISTER
  pub fn store_registers(registers: &mut RegistersBase, instr: u16, memory: &mut MemoryBase) {
    let r0 = usize!(instr >> 9 & 0x7);
    let r1 = usize!(instr >> 6 & 0x7);
    let offset = sign_extend(instr & 0x3F, 6);
    mem_write(memory, usize!(registers[r1] + offset), registers[r0]);
  }

  pub fn rti() {
    std::process::exit(1);
  }

  // BITWISE NOT
  pub fn bitwise_not(registers: &mut RegistersBase, instr: u16) {
    let r0 = usize!(instr >> 9 & 0x7);
    let r1 = usize!(instr >> 6 & 0x7);

    registers[r0] = !registers[r1];
    update_flags(registers, r0);
  }
  // LOAD INDIRECT
  pub fn load_indirect(registers: &mut RegistersBase, instr: u16, memory: &mut MemoryBase) {
    let r0 = usize!(instr >> 9 & 0x7);
    /* PCoffset 9*/
    let pc_offset = sign_extend(instr & 0x1ff, 9);
    /* add pc_offset to the current PC, look at that memory location to get the final address */
    let preload = mem_read(memory, registers[Registers::PC as usize] + pc_offset);
    registers[r0] = mem_read(memory, preload);
    update_flags(registers, r0);
  }

  // STORE INDIRECT
  pub fn store_indirect(registers: &mut RegistersBase, instr: u16, memory: &mut MemoryBase) {
    let r0 = usize!(instr >> 9 & 0x7);
    let pc_offset = sign_extend(instr & 0x1ff, 9);
    let mem_read_result = usize!(mem_read(
      memory,
      registers[Registers::PC as usize] + pc_offset
    ));
    mem_write(memory, mem_read_result, registers[r0]);
  }

  // JUMP
  pub fn jump(registers: &mut RegistersBase, instr: u16) {
    /* Also handles RET */
    let r1 = usize!(instr >> 6 & 0x7);
    registers[Registers::PC as usize] = registers[r1];
  }

  // XOR
  pub fn xor(registers: &mut RegistersBase, instr: u16) {
    let r0 = usize!(instr >> 9 & 0x7);
    let r1 = usize!(instr >> 6 & 0x7);
    registers[r0] = registers[r0] ^ registers[r1];
    update_flags(registers, r0);
  }

  // LOAD EFFECTIVE ADDRESS
  pub fn effective_address(registers: &mut RegistersBase, instr: u16) {
    let r0 = usize!(instr >> 9 & 0x7);
    let pc_offset = sign_extend(instr & 0x1ff, 9);
    registers[r0] = registers[Registers::PC as usize] + pc_offset;
    update_flags(registers, r0);
  }

  // TRAP
  pub fn trap(registers: &mut RegistersBase, instr: u16, memory: &mut MemoryBase, flag: &mut bool) {
    let action = instr & 0xFF;
    match action {
      _ if action == TrapCodes::TRAP_GETC as u16 => {
        let mut buffer = [0; 1];
        std::io::stdin().read_exact(&mut buffer).unwrap();
        registers[Registers::R0 as usize] = buffer[0] as u16;
      }
      _ if action == TrapCodes::TRAP_HALT as u16 => {
        println!("[!] HALT");
        // break 'runtime;
        *flag = false;
      }
      _ if action == TrapCodes::TRAP_IN as u16 => {
        print!("Enter a character: ");
        registers[Registers::R0 as usize] = std::io::stdin()
          .bytes()
          .next()
          .and_then(|result| result.ok())
          .map(|byte| byte as u16)
          .unwrap();
      }
      _ if action == TrapCodes::TRAP_OUT as u16 => {
        let c = registers[Registers::R0 as usize] as u8;
        print!("{}", c as char);
      }
      _ if action == TrapCodes::TRAP_PUTS as u16 => {
        for c in &memory[registers[Registers::R0 as usize] as usize..] {
          let c8 = (c & 0xff) as u8;
          if c8 != 0x00 {
            print!("{}", c8 as char);
          } else {
            break;
          }
        }
      }
      _ if action == TrapCodes::TRAP_PUTSP as u16 => {
        for c in &memory[registers[Registers::R0 as usize] as usize..] {
          let b1 = (*c >> 8) as u8;
          let b2 = (*c & 0xff) as u8;
          if b1 != 0 {
            print!("{}", b1 as char);
            if b2 != 0 {
              print!("{}", b2 as char);
            }
          }
        }
      }
      _ => {
        println!("UNKNOW TRAPCODE");
        std::process::exit(1);
      }
    }
  }
}
