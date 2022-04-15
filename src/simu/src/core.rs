use crate::data::Word;
use crate::trace;

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
    ORI,
    SLLI,
    JAL,
    JALR,
    CSRRS,
    CSRRW,
    CSRRWI,
    BEQ,
    BNE,
    BLT,
    LUI,
    AUIPC,
    MRET,
    FENCE,
}

enum InstType {
    I,
    R,
    J,
    B,
    U,
    C,
}

// enum Opcode {
//     IMM,
// }

fn get_inst_name(inst: &Inst) -> &'static str {
    match inst {
        Inst::ADDI => "ADDI",
        Inst::ORI => "ORI",
        Inst::SLLI => "SLLI",
        Inst::JAL => "JAL",
        Inst::JALR => "JALR",
        Inst::CSRRS => "CSRRS",
        Inst::CSRRW => "CSRRW",
        Inst::CSRRWI => "CSRRWI",
        Inst::BEQ => "BEQ",
        Inst::BNE => "BNE",
        Inst::BLT => "BLT",
        Inst::LUI => "LUI",
        Inst::AUIPC => "AUIPC",
        Inst::MRET => "MRET",
        Inst::FENCE => "FENCE",
    }
}

fn get_instruction_type(inst: &Inst) -> InstType {
    match inst {
        Inst::ADDI | Inst::ORI | Inst::SLLI | Inst::JALR | Inst::FENCE => InstType::I,
        Inst::MRET => InstType::R,
        Inst::JAL => InstType::J,
        Inst::CSRRS | Inst::CSRRW | Inst::CSRRWI => InstType::C,
        Inst::BEQ | Inst::BNE | Inst::BLT => InstType::B,
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
            // println!("val: {:08x}", self.load_word(self.pc));
            let end = match self.load_word(self.pc) {
                0x00000073 => true,
                _ => false,
            };

            if end {
                match self.x[10] {
                    0 => println!("Test Passed"),
                    _ => println!("Test Failed"),
                };
                break;
            }

            self.tick();
            // println!("ra: {:08x} t2: {:08x} a4: {:08x}", self.x[1], self.x[7], self.x[14]);
        }
    }

    fn tick(&mut self) {
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

    fn imm_ext_gen(inst_type: InstType, word: u32) -> i32 {
        let inst = Word::new(word);
        match inst_type {
            InstType::I => {
                // imm[31:11] = inst[31]
                // imm[10:0] = inst[30:20]
                return (match inst.val(31, 31) {
                    1 => 0xFFFF_F800,
                    0 => 0,
                    _ => panic!()
                } | (inst.pos(30, 20, 0))) as i32;
            }
            InstType::J => {
                // imm[31:20] = [31]
                // imm[19:12] = [19:12]
                // imm[11] = [20]
                // imm[10:1] = [30:21]
                return (match inst.val(31, 31) {
                    1 => 0xFFF0_0000,
                    0 => 0,
                    _ => panic!()
                } | (inst.pos(19, 12, 12))
                    | (inst.pos(20, 20, 11))
                    | (inst.pos(30, 21, 1))) as i32;
            }
            InstType::B => {
                // imm[31:12] = [31]
                // imm[11] = [7]
                // imm[10:5] = [30:25]
                // imm[4:1] = [11:8]
                return (match inst.val(31, 31) {
                    1 => 0xFFFF_F800,
                    0 => 0,
                    _ => panic!()
                } | (inst.pos(7, 7, 11))
                    | (inst.pos(30, 25, 5))
                    | (inst.pos(11, 8, 1))) as i32;
            }
            _ => {
                panic!();
            }
        }
    }

    fn decode(&mut self, word: u32) -> Inst {
        let inst = Word::new(word);
        let opcode = inst.val(6, 0);
        let func3 = inst.val(14, 12);
        // let func7 = (word >> 25) & 0x7F;
        match opcode {
            0x0F => {
                return match func3 {
                    0 => Inst::FENCE,
                    _ => {
                        panic!()
                    }
                };
            }
            0x13 => {
                return match func3 {
                    0 => Inst::ADDI,
                    1 => Inst::SLLI,
                    6 => Inst::ORI,
                    _ => {
                        trace::execpt_handle(self.pc, word);
                        panic!();
                    }
                };
            }
            0x17 => {
                return Inst::AUIPC;
            }
            0x37 => {
                return Inst::LUI;
            }
            0x63 => {
                return match func3 {
                    0 => Inst::BEQ,
                    1 => Inst::BNE,
                    4 => Inst::BLT,
                    _ => {
                        trace::execpt_handle(self.pc, word);
                        panic!();
                    }
                }
            }
            0x67 => {
                return Inst::JALR;
            }
            0x6F => {
                return Inst::JAL;
            }
            0x73 => {
                return match func3 {
                    0 => {
                        if word == 0x30200073 {
                            return Inst::MRET;
                        } else {
                            panic!();
                        }
                    }
                    1 => Inst::CSRRW,
                    2 => Inst::CSRRS,
                    5 => Inst::CSRRWI,
                    _ => {
                        trace::execpt_handle(self.pc, word);
                        panic!()
                    }
                };
            }
            _ => {
                trace::execpt_handle(self.pc, word);
                panic!()
            }
        }
    }

    fn exec(&mut self, word: u32, inst: Inst) {
        let inst_type = get_instruction_type(&inst);
        let inst_wrap = Word::new(word);
        match inst_type {
            InstType::I => {
                let rd = inst_wrap.val(11, 7);
                let rs1 = inst_wrap.val(19, 15);
                let imm = Core::imm_ext_gen(InstType::I, word);

                match inst {
                    Inst::ADDI => {
                        // println!("imm: {}, rd: {}, rs1: {}, x[rs1]: {:08x} ", imm, rd, rs1, self.x[rs1 as usize]);
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
                    Inst::ORI => {
                        if rd > 0 {
                            self.x[rd as usize] = self.x[rs1 as usize] | imm;
                        }
                    }
                    Inst::JALR => {
                        if rd > 0 {
                            self.x[rd as usize] = self.pc as i32; // HACK:  x0 is all zero!
                        }
                        self.pc = (self.x[rs1 as usize] as u32).wrapping_add(imm as u32);
                    }
                    Inst::FENCE => {
                        // no impl
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
            InstType::R => {
                // let rd = (word >> 7) & 0x1F; // [11:7]
                // let rs1 = (word >> 15) & 0x1F; // [19:15]
                // let rs2 = (word >> 20) & 0x1F; // [24:20]
                match inst {
                    Inst::MRET => {}
                    _ => {
                        panic!()
                    }
                }
            }
            InstType::J => {
                let rd = inst_wrap.val(11, 7);
                let imm = Core::imm_ext_gen(InstType::J, word);
                match inst {
                    Inst::JAL => {
                        if rd > 0 {
                            self.x[rd as usize] = self.pc as i32; // HACK:  x0 is all zero!
                        }
                        self.pc = self.pc.wrapping_sub(4).wrapping_add(imm as u32);
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
                let rs1 = inst_wrap.val(19, 15);
                let rs2 = inst_wrap.val(24, 20);
                let imm = Core::imm_ext_gen(InstType::B, word);
                // println!("x[rs1]: {}, x[rs2]: {}", self.x[rs1 as usize], self.x[rs2 as usize]);
                // panic!();
                match inst {
                    Inst::BEQ => {
                        if self.x[rs1 as usize] == self.x[rs2 as usize] {
                            self.pc = self.pc.wrapping_sub(4).wrapping_add(imm as u32);
                        }
                    }
                    Inst::BNE => {
                        if self.x[rs1 as usize] != self.x[rs2 as usize] {
                            self.pc = self.pc.wrapping_sub(4).wrapping_add(imm as u32);
                        }
                    }
                    Inst::BLT => {
                        // println!("rs1: {}, rs2: {}", rs1, rs2);
                        // trace::execpt_handle(self.pc, word);
                        if self.x[rs1 as usize] < self.x[rs2 as usize] {
                            self.pc = self.pc.wrapping_sub(4).wrapping_add(imm as u32);
                        }
                    }
                    _ => {
                        panic!();
                    }
                }
            }
            InstType::U => {
                let rd = inst_wrap.val(11, 7);
                let imm = word & 0xFFFF_F000; // HACK: need to modfiy
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
                let rd = inst_wrap.val(11, 7);
                let rs1 = inst_wrap.val(19, 15);
                let csr = inst_wrap.val(31, 20);

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
                        self.csr[csr as usize] =
                            self.csr[csr as usize] | self.x[rs1 as usize] as u32;
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
