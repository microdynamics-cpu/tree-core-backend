use crate::core::Core;
// use std::io::Write;

// like nemu
// 0x00000297,  // auipc t0,0
// 0x0002b823,  // sd  zero,16(t0)
// 0x0102b503,  // ld  a0,16(t0)
// 0x0000006b,  // treecore_trap
// 0xdeadbeef,  // some data


pub struct Cli {
    prompt: String,
}

impl Cli {
    pub fn new() -> Self {
        Cli {
            prompt: ">>>".to_string(),
        }
    }

    pub fn inter_mode(&self, core: &mut Core) {
        println!("TreeCore RISCV ISA Simulator 0.0.1");
        println!("[last-release] on Ubuntu 20.04 LTS");
        println!("Type 'help' for more information.");
        println!("{}", self.prompt);

        let dummy_bin: Vec<u8> = vec![
            0x97, 0x02, 0x00, 0x00, 0x23, 0xb8, 0x02, 0x00, 0x03, 0xb5, 0x02, 0x01, 0x6b, 0x00, 0x00,
            0x00, 0xef, 0xbe, 0xad, 0xde,
        ];
    
        core.load_bin_file(dummy_bin);
        core.run_simu(None, None);
        
        loop {

        }
    }
}


