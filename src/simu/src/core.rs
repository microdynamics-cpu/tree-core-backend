const MEMORY_CAPACITY: usize = 1024 * 16;
// const CSR_CAPACITY: usize = 4096;

pub struct Core {
    x: [i32; 32],
    pc: u32,
    // csr: [u32; CSR_CAPACITY],
    mem: [u8; MEMORY_CAPACITY],
}

enum Inst {
    ADDI,
    JAL,
    JALR,
}

enum InstFormat {
    I,
    J,
}

fn get_inst_name(inst: &Inst) -> &'static str {
    match inst {
        Inst::ADDI => "ADDI",
        Inst::JAL => "JAL",
        Inst::JALR => "JALR",
    }
}

fn get_instruction_format(inst: &Inst) -> InstFormat {
    match inst {
        Inst::ADDI | Inst::JALR => InstFormat::I,
        Inst::JAL => InstFormat::J,
    }
}

impl Core {
    pub fn new() -> Self {
        Core {
            x: [0; 32],
            pc: 0x1000,
            // csr: [0; CSR_CAPACITY],
            mem: [0; MEMORY_CAPACITY],
        }
    }

    pub fn run_simu(&mut self, data: Vec<u8>) {
        for i in 0..data.len() {
            self.mem[i] = data[i];
        }

        self.pc = 0x1000;
        loop {
            let end = match self.load_word(self.pc) {
                0x00000073 => true,
                _ => false,
            };

            self.tick();
            if end {
                match self.x[10] {
                    0 => println!("Test Passed"),
                    _ => println!("Test Failed"),
                };
                break;
            }
        }
    }

    pub fn tick(&mut self) {
        let word = self.fetch();
        let inst = self.decode(word);
        println!(
            "PC:{:08x}, Word:{:08x}, Inst:{}",
            self.pc.wrapping_sub(4),
            word,
            get_inst_name(&inst)
        );
        self.exec(word, inst);
    }

    fn fetch(&mut self) -> u32 {
        let word = self.load_word(self.pc);
        self.pc = self.pc.wrapping_add(4);
        word
    }

    fn load_word(&mut self, addr: u32) -> u32 {
        ((self.mem[addr as usize + 3] as u32) << 24)
            | ((self.mem[addr as usize + 2] as u32) << 16)
            | ((self.mem[addr as usize + 1] as u32) << 8)
            | (self.mem[addr as usize] as u32)
    }

    fn decode(&mut self, word: u32) -> Inst {
        let opcode = word & 0x7F;
        let func3 = (word >> 12) & 0x7F;
        // let func7 = (word >> 25) & 0x7F;

        if opcode == 0x13 {
            return match func3 {
                0 => Inst::ADDI,
                _ => {
                    println!("unkown func3: {:03b}", func3);
                    panic!();
                }
            };
        }

        if opcode == 0x67 {
            return Inst::JALR;
        }
        if opcode == 0x6F {
            return Inst::JAL;
        }
        println!("Unknown Inst type: {:03x}", word);
        panic!();
    }

    fn exec(&mut self, word: u32, inst: Inst) {
        let format = get_instruction_format(&inst);
        match format {
            InstFormat::I => {
                let rd = (word >> 7) & 0x1F; // [11:7]
                let rs1 = (word >> 15) & 0x1F; // [19:15]
                let imm = (
                    match word & 0x80000000 {
                        // imm[31:11] = [31]
                        0x80000000 => 0xfffff800,
                        _ => 0,
                    } | ((word >> 20) & 0x000007ff)
                    // imm[10:0] = [30:20]
                ) as i32;

                match inst {
                    Inst::ADDI => {
                        self.x[rd as usize] = self.x[rs1 as usize].wrapping_add(imm);
                    }
                    Inst::JALR => {
                        self.x[rd as usize] = self.pc as i32;
                        self.pc = (self.x[rs1 as usize] as u32).wrapping_add(imm as u32);
                    }
                    _ => {
                        println!(
                            "{}",
                            get_inst_name(&inst).to_owned() + " inst is not supported yet."
                        );
                        panic!();
                    }
                }
            }
            InstFormat::J => {
                let rd = (word >> 7) & 0x1f; // [11:7]
                let imm = (
                    match word & 0x80000000 { // imm[31:20] = [31]
						0x80000000 => 0xfff00000,
						_ => 0
					} |
					(word & 0x000ff000) | // imm[19:12] = [19:12]
					((word & 0x00100000) >> 9) | // imm[11] = [20]
					((word & 0x7fe00000) >> 20)
                    // imm[10:1] = [30:21]
                ) as u32;
                match inst {
                    Inst::JAL => {
                        self.x[rd as usize] = self.pc as i32;
                        self.pc = self.pc.wrapping_sub(4).wrapping_add(imm);
                    }
                    _ => {
                        println!(
                            "{}",
                            get_inst_name(&inst).to_owned() + " inst is not supported yet."
                        );
                        // self.dump_instruction(self.pc.wrapping_sub(4));
                        panic!();
                    }
                };
            }
        }
    }
}
