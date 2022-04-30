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
        stdout().flush();
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
        (self.buf >> self.cnt) as u8
    }
}

pub struct Device {
    pub rtc: Rtc,
}

impl Device {
    pub fn new() -> Self {
        Device { rtc: Rtc::new() }
    }
}
