use std::io::{Write, stdout};

pub struct Uart {
    buf: u8,
}

impl Uart {
    pub fn new() -> Self {
        Uart {
            buf: 0u8,
        }
    }

    pub fn out(dat: u8) {
        print!("{}", dat as char);
        stdout().flush();
    }
}