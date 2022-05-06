use std::io::{stdout, Write};
use std::time::Instant;

pub struct Uart {
    // buf: u8,
}

impl Uart {
    // pub fn new() -> Self {
    //     Uart {
    //         // buf: 0u8,
    //     }
    // }

    pub fn out(dat: u8) {
        print!("{}", dat as char);
        match stdout().flush() {
            Ok(()) => {}
            Err(_e) => panic!(),
        }
    }
}

pub struct Rtc {
    cur_t: Instant,
    buf: u32,
    cnt: u8,
    loading_flag: bool,
}

impl Rtc {
    pub fn new() -> Self {
        Rtc {
            cur_t: Instant::now(),
            buf: 0u32,
            cnt: 0u8,
            loading_flag: false,
        }
    }

    pub fn val(&mut self) -> u8 {
        if !self.loading_flag {
            self.buf = self.cur_t.elapsed().as_micros() as u32;
            self.loading_flag = true;
            self.cnt = 0;
        } else {
            self.cnt += 1;
            if self.cnt == 3 {
                self.loading_flag = false;
            }
        }
        // println!("elapsed: {:08x}", self.buf);
        // println!("cnt: {}, data: {:08x}\n", self.cnt, (self.buf >> (self.cnt * 8)) as u8);
        (self.buf >> (self.cnt * 8)) as u8
    }
}

pub struct Keyboard {
    press: u8,
    code: u8,
}

impl Keyboard {
    pub fn new() -> Self {
        Keyboard {
            press: 0u8,
            code: 0u8,
        }
    }

    pub fn val(&self, offset: bool) -> u8 {
        if !offset {
            self.press
        } else {
            self.code
        }
    }

    pub fn det(&mut self, press: u8, code: u8) {
        self.press = press;
        self.code = code;
        // println!("[det]: pre: {}, code: {}", self.press, self.code);
    }
}

pub struct Device {
    pub rtc: Rtc,
    pub kdb: Keyboard,
}

impl Device {
    pub fn new() -> Self {
        Device {
            rtc: Rtc::new(),
            kdb: Keyboard::new(),
        }
    }
}
