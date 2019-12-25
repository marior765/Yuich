pub enum Registers {
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

pub enum Conditions {
  FL_POS = 1 << 0, /* P */
  FL_ZRO = 1 << 1, /* Z */
  FL_NEG = 1 << 2, /* N */
}

pub enum MemoryMappedRegisters {
  MR_KBSR = 0xFE00, /* keyboard status */
  MR_KBDR = 0xFE02, /* keyboard data */
  R_ZERO = 0xFFFF,  /* special zero register */
}

pub type RegistersBase = [u16; Registers::COUNT as usize];

// impl std::ops::Deref for Registers {
//   type Target = u16;

//   fn deref(&self) -> &Self::Target {
//     &self
//   }
// }

impl Registers {
  pub fn init() -> RegistersBase {
    [0; Registers::COUNT as usize]
  }
}
