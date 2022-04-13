const MEM_CAPACITY: usize = 1024 * 16;
const CSR_CAPACITY: usize = 4096;

pub struct Core {
    x: [i32; 32],
    pc: u32,
    csr: [u32; CSR_CAPACITY],
    mem: [u8; MEM_CAPACITY],
}

enum Inst {
    ADDI,
    SLLI,
    JAL,
    JALR,
    CSRRS,
    CSRRW,
    CSRRWI,
    BNE,
    LUI,
    AUIPC,
}

enum InstType {
    I,
    J,
    B,
    U,
    C,
}

fn get_inst_name(inst: &Inst) -> &'static str {
    match inst {
        Inst::ADDI => "ADDI",
        Inst::SLLI => "SLLI",
        Inst::JAL => "JAL",
        Inst::JALR => "JALR",
        Inst::CSRRS => "CSRRS",
        Inst::CSRRW => "CSRRW",
        Inst::CSRRWI => "CSRRWI",
        Inst::BNE => "BNE",
        Inst::LUI => "LUI",
        Inst::AUIPC => "AUIPC",
    }
}

fn get_instruction_format(inst: &Inst) -> InstType {
    match inst {
        Inst::ADDI | 
        Inst::SLLI |
        Inst::JALR => InstType::I,
        Inst::JAL => InstType::J,
        Inst::CSRRS |
        Inst::CSRRW |
        Inst::CSRRWI  => InstType::C,
        Inst::BNE => InstType::B,
        Inst::LUI | Inst::AUIPC => InstType::U,
    }
}

impl Core {
    pub fn new() -> Self {
        Core {
            x: [0; 32],
            pc: 0x1000,
            csr: [0; CSR_CAPACITY], // NOTE: need to prepare specific val for reg, such as mhardid
            mem: [0; MEM_CAPACITY],
        }
    }

    pub fn run_simu(&mut self, data: Vec<u8>) {
        for i in 0..data.len() {
            self.mem[i] = data[i];
        }

        self.pc = 0x1000;
        // for v in self.csr.iter() {
        //     println!("csr: {}", v);
        // }
        // for v in self.x.iter() {
        //     println!("x: {}", v);
        // }

        // panic!();

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
        // for v in self.x.iter() {
        //     println!("x: {:x}", v);
        // }

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
        let func3 = (word >> 12) & 0x7;
        // let func7 = (word >> 25) & 0x7F;

        if opcode == 0x13 {
            return match func3 {
                0 => Inst::ADDI,
                1 => Inst::SLLI,
                _ => {
                    println!("unkown func3: {:03b}", func3);
                    println!(
                        "[decode: UNKNOWN] PC:{:08x}, Word:{:08x}",
                        self.pc.wrapping_sub(4),
                        word
                    );
                    panic!();
                }
            };
        }

        if opcode == 0x17 {
            return Inst::AUIPC;
        }

        if opcode == 0x37 {
            return Inst::LUI;
        }

        if opcode == 0x63 {
            return Inst::BNE;
        }

        if opcode == 0x67 {
            return Inst::JALR;
        }

        if opcode == 0x6F {
            return Inst::JAL;
        }
        
        // println!("func3: {}", func3);
        if opcode == 0x73 {
            if func3 == 1 {
                return Inst::CSRRW;
            } else if func3 == 2 {
                return Inst::CSRRS;
            } else if func3 == 5 {
                return Inst::CSRRWI;
            } else {
                println!(
                    "[decode: UNKNOWN] PC:{:08x}, Word:{:08x}",
                    self.pc.wrapping_sub(4),
                    word
                );
                panic!();
            }
        }

        println!(
            "[decode: UNKNOWN] PC:{:08x}, Word:{:08x}",
            self.pc.wrapping_sub(4),
            word
        );
        panic!();
    }

    fn exec(&mut self, word: u32, inst: Inst) {
        let format = get_instruction_format(&inst);
        match format {
            InstType::I => {
                let rd = (word >> 7) & 0x1F; // [11:7]
                let rs1 = (word >> 15) & 0x1F; // [19:15]
                let imm = (match word & 0x8000_0000 {
                    // sign extn
                    // imm[31:11] = inst[31]
                    // imm[10:0] = inst[30:20]
                    0x8000_0000 => 0xFFFF_F800,
                    _ => 0,
                } | ((word >> 20) & 0x0000_07FF)) as i32;

                match inst {
                    Inst::ADDI => {
                        // println!("imm: {}, rd: {}, rs1: {}, x[rs1]: {} ", imm, rd, rs1, self.x[rs1 as usize]);
                        if rd > 0 {
                            self.x[rd as usize] = self.x[rs1 as usize].wrapping_add(imm);
                        }
                        
                    }
                    Inst::SLLI => {
                        if rd > 0 {
                            let shamt = (imm & 0x1F) as u32;
                            self.x[rd as usize] = self.x[rs1 as usize] << shamt;
                        }
                    }
                    Inst::JALR => {
                        if rd > 0 {
                            self.x[rd as usize] = self.pc as i32; // HACK:  x0 is all zero!
                        }
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
            InstType::J => {
                let rd = (word >> 7) & 0x1F; // [11:7]
                let imm = (
                    match word & 0x8000_0000 { // imm[31:20] = [31]
                        0x8000_0000 => 0xFFF0_0000,
                        _ => 0
                    } |
                    (word & 0x000F_F000) | // imm[19:12] = [19:12]
                    ((word & 0x0010_0000) >> 9) | // imm[11] = [20]
                    ((word & 0x7FE0_0000) >> 20)
                    // imm[10:1] = [30:21]
                ) as u32;
                match inst {
                    Inst::JAL => {
                        if rd > 0 {
                            self.x[rd as usize] = self.pc as i32; // HACK:  x0 is all zero!
                        }
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
            InstType::B => {
                let rs1 = (word >> 15) & 0x1F; // [11:7]
                let rs2 = (word >> 20) & 0x1F; // [24:20]
                let imm = (
                    match word & 0x80000000 { // imm[31:12] = [31]
                        0x80000000 => 0xFFFF_F800,
                        _ => 0
                    } |
                    ((word & 0x0000_0080) << 4) | // imm[11] = [7]
                    ((word & 0x7E00_0000) >> 20) | // imm[10:5] = [30:25]
                    ((word & 0x0000_0F00) >> 7)
                    // imm[4:1] = [11:8]
                ) as u32;
                // println!("x[rs1]: {}, x[rs2]: {}", self.x[rs1 as usize], self.x[rs2 as usize]);
                // panic!();
                if self.x[rs1 as usize] != self.x[rs2 as usize] {
                    self.pc = self.pc.wrapping_sub(4).wrapping_add(imm);
                }
            }
            InstType::U => {
                let rd = (word >> 7) & 0x1F; // [11:7]
                let imm = word & 0xFFFF_F000;
                match inst {
                    Inst::AUIPC => {
                        if rd > 0 {
                            self.x[rd as usize] = self.pc.wrapping_sub(4).wrapping_add(imm) as i32;
                        }
                    }
                    Inst::LUI => {
                        if rd > 0 {
                            self.x[rd as usize] = imm as i32;
                        }
                    }
                    _ => {
                        panic!();
                    }
                }



            }

            InstType::C => {
                let rd = (word >> 7) & 0x1F; // [11:7]
                let rs1 = (word >> 15) & 0x1F; // [19:15]
                let csr = (word >> 20) & 0xFF; // [31:20]

                match inst {
                    Inst::CSRRW => {
                        if rd > 0 {
                            self.x[rd as usize] = self.csr[csr as usize] as i32;
                        }
                        self.csr[csr as usize] = self.x[rs1 as usize] as u32;
                    }
                    Inst::CSRRS => {
                        if rd > 0 {
                            self.x[rd as usize] = self.csr[csr as usize] as i32;
                        }
                        self.csr[csr as usize] = self.csr[csr as usize] | self.x[rs1 as usize] as u32;
                    }
                    Inst::CSRRWI => {
                        if rd > 0 {
                            self.x[rd as usize] = self.csr[csr as usize] as i32;
                        }
                        self.csr[csr as usize] = rs1;
                    }
                    _ => {
                        panic!();
                    }
                }

            }
        }
    }
}
