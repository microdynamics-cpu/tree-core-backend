use crate::inst::{get_inst_name, Inst};
use crate::regfile::Regfile;

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

pub fn rtrace(regfile: &Regfile, val: &str) {
    if val != "0" {
        println!("{}: {:016x}", val, regfile.val(val));
    }
}
pub fn csr_trace() {}

pub fn mtrace() {}
pub fn dtrace() {}

pub fn etrace() {}