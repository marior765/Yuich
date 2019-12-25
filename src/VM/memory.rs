use std::fs::File;
use std::io::Read;
use std::path::Path;

use super::lib::get16;
use super::registers::MemoryMappedRegisters;

pub type MemoryBase = [u16; std::u16::MAX as usize];

pub struct Memory {
  memory: MemoryBase,
}

impl Memory {
  pub fn init() -> MemoryBase {
    [0; std::u16::MAX as usize]
  }
}

pub fn mem_read(mem_ref: &mut [u16], address: u16) -> u16 {
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

pub fn mem_write(mem_ref: &mut [u16], address: usize, val: u16) {
  if address == MemoryMappedRegisters::R_ZERO as usize {
    mem_ref[address] = 0; /* zero register */
  } else {
    mem_ref[address] = val;
  }
  // mem_ref[address] = val;
}

pub fn read_image(mem: &mut [u16], image_path: &str) -> u32 {
  let path = Path::new(image_path);
  println!("[*] Loading {}", path.to_str().unwrap());
  let mut file = File::open(&path).expect("Couldn't open file.");

  const SIZE: u32 = std::u16::MAX as u32 * 2 - 2;
  let mut mem_buffer: [u8; SIZE as usize] = [0; SIZE as usize];
  file.read(&mut mem_buffer).expect("Couldn't read file.");
  let length = file.metadata().unwrap().len();
  // println!("[*] File length {}", length);

  let base = get16(&mem_buffer, 0) as usize;
  for i in (2..length).step_by(2) {
    // println!("{}", i);
    mem[base + (i / 2 - 1) as usize] = get16(&mem_buffer, i as usize);
  }
  // println!("{:?}", &mem[0x3000..0x4000]);
  length as u32
}
