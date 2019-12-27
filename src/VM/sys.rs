use termios::*;

pub struct Sys {
  termios: Termios,
  stdin: i32,
}

impl Sys {
  pub fn init() -> Self {
    Sys {
      stdin: 0,
      termios: Termios::from_fd(0).unwrap(),
    }
  }

  pub fn set_flags(&self) {
    let mut new_termios = self.termios.clone(); // make a mutable copy of termios
                                                // that we will modify
    new_termios.c_iflag &= IGNBRK | BRKINT | PARMRK | ISTRIP | INLCR | IGNCR | ICRNL | IXON;
    new_termios.c_lflag &= !(ICANON | ECHO); // no echo and canonical mode
    tcsetattr(self.stdin, TCSANOW, &mut new_termios).unwrap();
  }

  pub fn set_terminal(&self) {
    tcsetattr(self.stdin, TCSANOW, &self.termios).unwrap();
  }
}
