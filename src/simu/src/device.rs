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

const VGA_BUF_SIZE: usize = 200 * 180 * 4;

pub struct Vga {
    width: u16,
    height: u16,
    pub sync: bool,
    cnt: u8,
    buf: [u8; VGA_BUF_SIZE],
}

impl Vga {
    pub fn new() -> Self {
        Vga {
            width: 192,
            height: 128,
            sync: false,
            cnt: 0,
            buf: [0; VGA_BUF_SIZE],
        }
    }

    pub fn val(&self, addr: u64) -> u8 {
        self.buf[(addr - 0xa0000000u64) as usize]
    }

    pub fn store(&mut self, addr: u64, val: u8) {
        self.buf[(addr - 0xa0000000u64) as usize] = val;
    }

    pub fn set_sync(&mut self, val: u8) -> bool {
        if self.cnt == 0 {
            self.sync = val == 1u8;
            println!("self.sync: {}", self.sync);
        }

        self.cnt += 1;
        if self.cnt == 4 {
            self.cnt = 0;
        }
        self.sync
    }

    pub fn send_dat(&mut self) -> String {
        // TODO: send data here
        // [0, self.width * self.height - 1];
        // HACK: PERF
        let mut res = "".to_string();
        let mut cnt = 0;
        for v in self.buf.iter() {
            if cnt == self.width * self.height {
                break;
            }
            res.push(*v as char);
            cnt += 1;
        }
        self.sync = false;
        res
    }
}

pub struct Device {
    pub rtc: Rtc,
    pub kdb: Keyboard,
    pub vga: Vga,
}

impl Device {
    pub fn new() -> Self {
        Device {
            rtc: Rtc::new(),
            kdb: Keyboard::new(),
            vga: Vga::new(),
        }
    }
}
