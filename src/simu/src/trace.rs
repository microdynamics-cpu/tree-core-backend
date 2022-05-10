use crate::inst::{get_inst_name, Inst};
use crate::regfile::Regfile;
use object::{Object, ObjectSection};
use std::error::Error;
use std::fs;


pub fn execpt_handle(pc: u64, word: u32) {
    println!(
        "[UNKNOWN] PC:{:016x}, Word:{:08x}",
        pc.wrapping_sub(4),
        word
    );
    panic!();
}

pub fn itrace(pc: u64, word: u32, inst: &Inst) {
    println!(
        "PC:{:016x}, Word:{:08x}, Inst:{}",
        pc.wrapping_sub(4),
        word,
        get_inst_name(&inst)
    );
}

pub fn mtrace() {
    println!("mtrace");
}

pub struct FTrace {
    // bin_data: io::Result<Vec<u8>>,
// obj_file: Result<object::File<'a, &'a [u8]>>,
}

impl FTrace {
    pub fn new(elf: &str) -> Self {
        FTrace {
            // bin_data: fs::read(elf),
            // match fs::read(elf) {
                // Ok(v) => obj_file: object::File::parse(&*v),
                // Err(_e) => panic!(),
            // }
            // obj_file: object::File::parse(&*fs::read(elf)?),
        }
    }

    pub fn ftrace(addr: u64) -> Result<(), Box<dyn Error>> {
        let bin_data = fs::read("./dependency/crt/am-kernels/tests/cpu-tests/build/printf-riscv64-treecore.elf")?;
        let obj_file = object::File::parse(&*bin_data)?;
        let map = obj_file.symbol_map();
        match map.get(addr) {
            Some(v) => println!("{}", v.name()),
            None => {},
        }
        Ok(())
    }
}

pub fn rtrace(regfile: &Regfile, val: &str) {
    if val != "0" {
        println!("{}: {:016x}", val, regfile.val(val));
    }
}
pub fn csr_trace() {}

pub fn dtrace() {}

pub fn etrace() {}

macro_rules! log {
    ($($args: expr),*) => {
        print!("[{}] line: {}", file!(), line!());
        $(
            print!(" {}: {:016x}", stringify!($args), $args);
        )*
        println!(""); // to get a new line at the end
    }
}

pub(crate) use log;
