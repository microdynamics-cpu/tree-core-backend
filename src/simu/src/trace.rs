use crate::inst::{get_inst_name, Inst};
use crate::privilege::Exception;
use crate::regfile::Regfile;
use object::{Object, ObjectSymbol};
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

pub fn itrace(pc: u64, word: u32, inst: &Inst, rge: &[u64; 2]) {
    if pc >= rge[0] && pc <= rge[1] {
        println!(
            "PC:{:016x}, Word:{:08x}, Inst:{}",
            pc.wrapping_sub(4),
            word,
            get_inst_name(&inst)
        );
    }
}

pub fn mtrace() {
    println!("mtrace");
}

pub struct FTrace {
    sym_addr_sta: Vec<u64>,
    sym_addr_name: Vec<String>,
    sym_num: u16,
}

impl FTrace {
    pub fn new(_elf: &str) -> Self {
        FTrace {
            sym_addr_sta: vec![],
            sym_addr_name: vec![],
            sym_num: 0u16,
        }
    }

    pub fn ftrace(&mut self, ori_addr: u64, addr: u64) -> Result<(), Box<dyn Error>> {
        let bin_data = fs::read(
            "./dependency/crt/am-kernels/tests/cpu-tests/build/string-riscv64-treecore.elf",
        )?;
        let obj_file = object::File::parse(&*bin_data)?;
        // let dat = obj_file.symbol_table();
        for v in obj_file.symbols() {
            if v.address() == addr {
                match v.name() {
                    Ok(vv) => {
                        self.sym_addr_sta.push(ori_addr);
                        self.sym_num += 1;
                        self.sym_addr_name.push(vv.to_string());
                        print!("{:#x}:", ori_addr);
                        print!("{:>1$}", " call ", (self.sym_num * 5) as usize);
                        println!("[{}@{:#x}]", vv, addr);
                    }
                    Err(_e) => {}
                }
            } else {
                if self.sym_addr_sta.last() == Some(&addr.wrapping_sub(4)) {
                    print!("{:#x}:", ori_addr);
                    print!("{:>1$}", " ret ", (self.sym_num * 5) as usize);
                    match self.sym_addr_name.last() {
                        Some(v) => println!("[{}]", v),
                        None => {}
                    }
                    self.sym_addr_sta.pop();
                    self.sym_addr_name.pop();
                    self.sym_num -= 1;
                }
            }
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

pub fn etrace(excpt: &Exception) {
    println!(
        "[etrace] type: {:?} addr: {:016x}",
        excpt.excpt_type, excpt.addr
    );
}

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
