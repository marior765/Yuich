macro_rules! usize {
	( $( $x:expr ),* ) => {
			{
				$(
						usize::from($x)
				)*
			}
	};
}

pub fn sign_extend(x: u16, bit_count: i32) -> u16 {
  if (x >> (bit_count - 1)) & 1 != 0 {
    return x | (0xFFFF << bit_count);
  }
  x
}

pub fn swap16(x: u16) -> u16 {
  x << 8 | x >> 8
}

pub fn get16(mem: &[u8], ind: usize) -> u16 {
  ((mem[ind] as u16) << 8) + mem[ind + 1] as u16
}
