pub fn execpt_handle(pc: u32, word: u32) {
    println!(
        "[UNKNOWN] PC:{:08x}, Word:{:08x}",
        pc.wrapping_sub(4),
        word
    );
    panic!();
}

pub fn regfile_trace() {}
pub fn csr_trace() {}