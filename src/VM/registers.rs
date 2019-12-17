pub type RegistersBase = [u16; Registers::COUNT as usize];

enum Registers {
  R0 = 0,
  R1,
  R2,
  R3,
  R4,
  R5,
  R6,
  R7,
  PC, /* program counter */
  COND,
  COUNT,
}

impl Registers {
  fn init() -> RegistersBase {
    [0; Registers::COUNT as usize]
  }
}
