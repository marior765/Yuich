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
  TRAP,   /* execute trap */
}

impl Instructions {
  fn branch(registers: RegistersBase) {
    let pc_offset = sign_extend((instr) & 0x1ff, 9);
    let cond_flag = (instr >> 9) & 0x7;
    if (cond_flag & registers[Registers::COND as usize]) != 0 {
      registers[Registers::PC as usize] += pc_offset;
    }
  }
}
