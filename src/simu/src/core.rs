use crate::data::Word;
use crate::regfile::Regfile;
use crate::trace;
const MEM_CAPACITY: usize = 1024 * 16;
const CSR_CAPACITY: usize = 4096;

pub struct Core {
    regfile: Regfile,
    pc: u32,
    csr: [u32; CSR_CAPACITY],
    mem: [u8; MEM_CAPACITY],
    inst_num: u32,
}

enum Inst {
    LUI,
    AUIPC,
    JAL,
    JALR,
    BEQ,
    BNE,
    BLT,
    BGE,
    BLTU,
    BGEU,
    LB,
    LH,
    LW,
    LBU,
    LHU,
    SB,
    SH,
    SW,
    ADDI,
    SLTI,
    SLTIU,
    XORI,
    ORI,
    ANDI,
    SLLI,
    SRLI,
    SRAI,
    ADD,
    SUB,
    SLL,
    SLT,
    SLTU,
    XOR,
    SRL,
    SRA,
    OR,
    AND,
    CSRRS,
    CSRRW,
    CSRRWI,
    MRET,
    FENCE,
}

enum InstType {
    R,
    I,
    S,
    B,
    U,
    J,
    C,
}

// enum Opcode {
//     IMM,
// }

fn get_inst_name(inst: &Inst) -> &'static str {
    match inst {
        Inst::LUI => "LUI",
        Inst::AUIPC => "AUIPC",
        Inst::JAL => "JAL",
        Inst::JALR => "JALR",
        Inst::BEQ => "BEQ",
        Inst::BNE => "BNE",
        Inst::BLT => "BLT",
        Inst::BGE => "BGE",
        Inst::BLTU => "BLTU",
        Inst::BGEU => "BGEU",
        Inst::LB => "LB",
        Inst::LH => "LH",
        Inst::LW => "LW",
        Inst::LBU => "LBU",
        Inst::LHU => "LHU",
        Inst::SB => "SB",
        Inst::SH => "SH",
        Inst::SW => "SW",
        Inst::ADDI => "ADDI",
        Inst::SLTI => "SLTI",
        Inst::SLTIU => "SLTIU",
        Inst::XORI => "XORI",
        Inst::ORI => "ORI",
        Inst::ANDI => "ANDI",
        Inst::SLLI => "SLLI",
        Inst::SRLI => "SRLI",
        Inst::SRAI => "SRAI",
        Inst::ADD => "ADD",
        Inst::SUB => "SUB",
        Inst::SLL => "SLL",
        Inst::SLT => "SLT",
        Inst::SLTU => "SLTU",
        Inst::XOR => "XOR",
        Inst::SRL => "SRL",
        Inst::SRA => "SRA",
        Inst::OR => "OR",
        Inst::AND => "AND",
        Inst::CSRRS => "CSRRS",
        Inst::CSRRW => "CSRRW",
        Inst::CSRRWI => "CSRRWI",
        Inst::MRET => "MRET",
        Inst::FENCE => "FENCE",
    }
}

fn get_instruction_type(inst: &Inst) -> InstType {
    match inst {
        Inst::ADD
        | Inst::SUB
        | Inst::SLL
        | Inst::SLT
        | Inst::SLTU
        | Inst::XOR
        | Inst::SRL
        | Inst::SRA
        | Inst::OR
        | Inst::AND
        | Inst::MRET => InstType::R,
        Inst::ADDI
        | Inst::SLTI
        | Inst::SLTIU
        | Inst::XORI
        | Inst::ORI
        | Inst::ANDI
        | Inst::SLLI
        | Inst::SRAI
        | Inst::SRLI
        | Inst::JALR
        | Inst::LB
        | Inst::LH
        | Inst::LW
        | Inst::LBU
        | Inst::LHU
        | Inst::FENCE => InstType::I,
        Inst::SB | Inst::SH | Inst::SW => InstType::S,
        Inst::BEQ | Inst::BNE | Inst::BLT | Inst::BGE | Inst::BLTU | Inst::BGEU => InstType::B,
        Inst::LUI | Inst::AUIPC => InstType::U,
        Inst::JAL => InstType::J,
        Inst::CSRRS | Inst::CSRRW | Inst::CSRRWI => InstType::C,
    }
}

impl Core {
    pub fn new() -> Self {
        Core {
            regfile: Regfile::new(),
            pc: 0x1000,
            csr: [0; CSR_CAPACITY], // NOTE: need to prepare specific val for reg, such as mhardid
            mem: [0; MEM_CAPACITY],
            inst_num: 0u32,
        }
    }

    pub fn run_simu(&mut self, data: Vec<u8>) {
        for i in 0..data.len() {
            self.mem[i] = data[i];
        }

        self.pc = 0x1000;

        loop {
            // println!("val: {:08x}", self.load_word(self.pc));
            let end = match self.load_word(self.pc) {
                0x00000073 => true,
                _ => false,
            };

            if end {
                match self.regfile.x[10] {
                    0 => println!("Test Passed, inst_num: {}", self.inst_num),
                    _ => println!("Test Failed"),
                };
                break;
            }

            self.tick();
            self.inst_num += 1;
            // println!("mem: {:08x}", self.mem[0x3000]);
            // println!("ra: {:08x} t2: {:08x} a4: {:08x}\n", self.regfile.val("ra"), self.regfile.val("t2"), self.regfile.val("a4"));
        }
    }

    fn tick(&mut self) {
        // for v in self.regfile.x.iter() {
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

    fn load_byte(&self, addr: u32) -> u8 {
        self.mem[addr as usize]
    }

    fn load_halfword(&self, addr: u32) -> u16 {
        ((self.mem[addr as usize + 1] as u16) << 8) | (self.mem[addr as usize] as u16)
    }

    fn load_word(&self, addr: u32) -> u32 {
        ((self.mem[addr as usize + 3] as u32) << 24)
            | ((self.mem[addr as usize + 2] as u32) << 16)
            | ((self.mem[addr as usize + 1] as u32) << 8)
            | (self.mem[addr as usize] as u32)
    }

    fn store_byte(&mut self, addr: u32, val: u8) {
        self.mem[addr as usize] = val;
    }

    fn store_halfword(&mut self, addr: u32, val: u16) {
        self.mem[addr as usize] = (val & 0xFFu16) as u8;
        self.mem[addr as usize + 1] = ((val >> 8) & 0xFFu16) as u8;
    }

    fn store_word(&mut self, addr: u32, val: u32) {
        self.mem[addr as usize] = (val & 0xFFu32) as u8;
        self.mem[addr as usize + 1] = ((val >> 8) & 0xFFu32) as u8;
        self.mem[addr as usize + 2] = ((val >> 16) & 0xFFu32) as u8;
        self.mem[addr as usize + 3] = ((val >> 24) & 0xFFu32) as u8;
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
                    _ => panic!(),
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
                    _ => panic!(),
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
                    _ => panic!(),
                } | (inst.pos(7, 7, 11))
                    | (inst.pos(30, 25, 5))
                    | (inst.pos(11, 8, 1))) as i32;
            }
            InstType::S => {
                return (match inst.val(31, 31) {
                    1 => 0xFFFF_F000,
                    0 => 0,
                    _ => panic!(),
                } | (inst.pos(31, 25, 5))
                    | (inst.pos(11, 7, 0))) as i32;
            }
            InstType::U => (word & 0xFFFF_F000) as i32,
            _ => {
                panic!();
            }
        }
    }

    fn decode(&mut self, word: u32) -> Inst {
        let inst = Word::new(word);
        let opcode = inst.val(6, 0);
        let func3 = inst.val(14, 12);
        let func7 = inst.val(31, 25);
        match opcode {
            0x03 => {
                return match func3 {
                    0 => Inst::LB,
                    1 => Inst::LH,
                    2 => Inst::LW,
                    4 => Inst::LBU,
                    5 => Inst::LHU,
                    _ => panic!(),
                }
            }
            0x0F => {
                return match func3 {
                    0 => Inst::FENCE,
                    _ => panic!(),
                };
            }
            0x13 => {
                return match func3 {
                    0 => Inst::ADDI,
                    1 => Inst::SLLI,
                    2 => Inst::SLTI,
                    3 => Inst::SLTIU,
                    4 => Inst::XORI,
                    5 => match func7 {
                        0x0 => Inst::SRLI,
                        0x20 => Inst::SRAI,
                        _ => panic!(),
                    },
                    6 => Inst::ORI,
                    7 => Inst::ANDI,
                    _ => {
                        trace::execpt_handle(self.pc, word);
                        panic!();
                    }
                };
            }
            0x17 => {
                return Inst::AUIPC;
            }
            0x23 => {
                return match func3 {
                    0 => Inst::SB,
                    1 => Inst::SH,
                    2 => Inst::SW,
                    _ => panic!(),
                }
            }
            0x33 => {
                return match func3 {
                    0 => match func7 {
                        0x00 => Inst::ADD,
                        0x20 => Inst::SUB,
                        _ => panic!(),
                    },
                    1 => Inst::SLL,
                    2 => Inst::SLT,
                    3 => Inst::SLTU,
                    4 => Inst::XOR,
                    5 => match func7 {
                        0x00 => Inst::SRL,
                        0x20 => Inst::SRA,
                        _ => panic!(),
                    },
                    6 => Inst::OR,
                    7 => Inst::AND,
                    _ => panic!(),
                }
            }
            0x37 => {
                return Inst::LUI;
            }
            0x63 => {
                return match func3 {
                    0 => Inst::BEQ,
                    1 => Inst::BNE,
                    4 => Inst::BLT,
                    5 => Inst::BGE,
                    6 => Inst::BLTU,
                    7 => Inst::BGEU,
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
                        // println!("imm: {}, rd: {}, rs1: {}, x[rs1]: {:08x} ", imm, rd, rs1, self.regfile.x[rs1 as usize]);
                        // self.regfile.wt(rd: usize, rs1: usize, val: i32, op: &str)
                        if rd > 0 {
                            self.regfile.x[rd as usize] =
                                self.regfile.x[rs1 as usize].wrapping_add(imm);
                        }
                    }
                    Inst::SLLI => {
                        if rd > 0 {
                            let shamt = (imm & 0x1F) as u32;
                            self.regfile.x[rd as usize] = self.regfile.x[rs1 as usize] << shamt;
                        }
                    }
                    Inst::SLTI => {
                        if rd > 0 {
                            if self.regfile.x[rs1 as usize] < imm {
                                self.regfile.x[rd as usize] = 1 as i32;
                            } else {
                                self.regfile.x[rd as usize] = 0 as i32;
                            }
                        }
                    }
                    Inst::SLTIU => {
                        if rd > 0 {
                            if (self.regfile.x[rs1 as usize] as u32) < (imm as u32) {
                                self.regfile.x[rd as usize] = 1 as i32;
                            } else {
                                self.regfile.x[rd as usize] = 0 as i32;
                            }
                        }
                    }
                    Inst::XORI => {
                        if rd > 0 {
                            self.regfile.x[rd as usize] = self.regfile.x[rs1 as usize] ^ imm;
                        }
                    }
                    Inst::ORI => {
                        if rd > 0 {
                            self.regfile.x[rd as usize] = self.regfile.x[rs1 as usize] | imm;
                        }
                    }
                    Inst::ANDI => {
                        self.regfile.x[rd as usize] = self.regfile.x[rs1 as usize] & imm;
                    }
                    Inst::SRLI => {
                        if rd > 0 {
                            let shamt = (imm & 0x1F) as u32;
                            self.regfile.x[rd as usize] =
                                ((self.regfile.x[rs1 as usize] as u32) >> shamt) as i32;
                        }
                    }
                    Inst::SRAI => {
                        if rd > 0 {
                            let shamt = (imm & 0x1F) as u32;
                            self.regfile.x[rd as usize] = self.regfile.x[rs1 as usize] >> shamt;
                        }
                    }
                    Inst::JALR => {
                        let tmp_pc = self.pc; // important!!!, if rs1 == rd
                        self.pc = (self.regfile.x[rs1 as usize] as u32).wrapping_add(imm as u32);
                        if rd > 0 {
                            self.regfile.x[rd as usize] = tmp_pc as i32;
                        }
                    }
                    Inst::LB => {
                        self.regfile.x[rd as usize] = self
                            .load_byte(self.regfile.x[rs1 as usize].wrapping_add(imm) as u32)
                            as i8 as i32; // NOTE: convert to i8 is important!!! different from 'LBU'
                                          // println!("val: {}", self.regfile.x[rd as usize]);
                    }
                    Inst::LH => {
                        self.regfile.x[rd as usize] = self
                            .load_halfword(self.regfile.x[rs1 as usize].wrapping_add(imm) as u32)
                            as i16 as i32;
                    }
                    Inst::LW => {
                        self.regfile.x[rd as usize] = self
                            .load_word(self.regfile.x[rs1 as usize].wrapping_add(imm) as u32)
                            as i32;
                    }
                    Inst::LBU => {
                        self.regfile.x[rd as usize] = self
                            .load_byte(self.regfile.x[rs1 as usize].wrapping_add(imm) as u32)
                            as i32;
                    }
                    Inst::LHU => {
                        self.regfile.x[rd as usize] = self
                            .load_halfword(self.regfile.x[rs1 as usize].wrapping_add(imm) as u32)
                            as i32;
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
                let rd = inst_wrap.val(11, 7);
                let rs1 = inst_wrap.val(19, 15);
                let rs2 = inst_wrap.val(24, 20);
                match inst {
                    Inst::ADD => {
                        if rd > 0 {
                            self.regfile.x[rd as usize] = self.regfile.x[rs1 as usize]
                                .wrapping_add(self.regfile.x[rs2 as usize]);
                        }
                    }
                    Inst::SUB => {
                        if rd > 0 {
                            self.regfile.x[rd as usize] = self.regfile.x[rs1 as usize]
                                .wrapping_sub(self.regfile.x[rs2 as usize]);
                        }
                    }
                    Inst::SLL => {
                        if rd > 0 {
                            self.regfile.x[rd as usize] = self.regfile.x[rs1 as usize]
                                .wrapping_shl(self.regfile.x[rs2 as usize] as u32);
                        }
                    }
                    Inst::SLT => {
                        if rd > 0 {
                            if self.regfile.x[rs1 as usize] < self.regfile.x[rs2 as usize] {
                                self.regfile.x[rd as usize] = 1;
                            } else {
                                self.regfile.x[rd as usize] = 0;
                            }
                        }
                    }
                    Inst::SLTU => {
                        if rd > 0 {
                            if (self.regfile.x[rs1 as usize] as u32)
                                < (self.regfile.x[rs2 as usize] as u32)
                            {
                                self.regfile.x[rd as usize] = 1;
                            } else {
                                self.regfile.x[rd as usize] = 0;
                            }
                        }
                    }
                    Inst::XOR => {
                        if rd > 0 {
                            self.regfile.x[rd as usize] =
                                self.regfile.x[rs1 as usize] ^ self.regfile.x[rs2 as usize];
                        }
                    }
                    Inst::SRL => {
                        if rd > 0 {
                            self.regfile.x[rd as usize] = ((self.regfile.x[rs1 as usize] as u32)
                                .wrapping_shr(self.regfile.x[rs2 as usize] as u32))
                                as i32;
                        }
                    }
                    Inst::SRA => {
                        if rd > 0 {
                            self.regfile.x[rd as usize] = self.regfile.x[rs1 as usize]
                                .wrapping_shr(self.regfile.x[rs2 as usize] as u32);
                        }
                    }
                    Inst::OR => {
                        if rd > 0 {
                            self.regfile.x[rd as usize] =
                                self.regfile.x[rs1 as usize] | self.regfile.x[rs2 as usize];
                        }
                    }
                    Inst::AND => {
                        if rd > 0 {
                            self.regfile.x[rd as usize] =
                                self.regfile.x[rs1 as usize] & self.regfile.x[rs2 as usize];
                        }
                    }
                    Inst::MRET => {}
                    _ => {
                        panic!()
                    }
                }
            }
            InstType::S => {
                let rs1 = inst_wrap.val(19, 15);
                let rs2 = inst_wrap.val(24, 20);
                let offset = Core::imm_ext_gen(InstType::S, word);
                match inst {
                    Inst::SB => {
                        self.store_byte(
                            self.regfile.x[rs1 as usize].wrapping_add(offset) as u32,
                            (self.regfile.x[rs2 as usize] as u8) & 0xFFu8,
                        );
                    }
                    Inst::SH => {
                        self.store_halfword(
                            self.regfile.x[rs1 as usize].wrapping_add(offset) as u32,
                            (self.regfile.x[rs2 as usize] as u16) & 0xFFFFu16,
                        );
                    }
                    Inst::SW => {
                        self.store_word(
                            self.regfile.x[rs1 as usize].wrapping_add(offset) as u32,
                            self.regfile.x[rs2 as usize] as u32,
                        );
                    }
                    _ => panic!(),
                }
            }
            InstType::J => {
                let rd = inst_wrap.val(11, 7);
                let imm = Core::imm_ext_gen(InstType::J, word);
                match inst {
                    Inst::JAL => {
                        if rd > 0 {
                            self.regfile.x[rd as usize] = self.pc as i32; // HACK:  x0 is all zero!
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
                // println!("x[rs1]: {}, x[rs2]: {}", self.regfile.x[rs1 as usize], self.regfile.x[rs2 as usize]);
                // panic!();
                match inst {
                    Inst::BEQ => {
                        if self.regfile.x[rs1 as usize] == self.regfile.x[rs2 as usize] {
                            self.pc = self.pc.wrapping_sub(4).wrapping_add(imm as u32);
                        }
                    }
                    Inst::BNE => {
                        if self.regfile.x[rs1 as usize] != self.regfile.x[rs2 as usize] {
                            self.pc = self.pc.wrapping_sub(4).wrapping_add(imm as u32);
                        }
                    }
                    Inst::BLT => {
                        // println!("rs1: {}, rs2: {}", rs1, rs2);
                        // trace::execpt_handle(self.pc, word);
                        if self.regfile.x[rs1 as usize] < self.regfile.x[rs2 as usize] {
                            self.pc = self.pc.wrapping_sub(4).wrapping_add(imm as u32);
                        }
                    }
                    Inst::BGE => {
                        if self.regfile.x[rs1 as usize] >= self.regfile.x[rs2 as usize] {
                            self.pc = self.pc.wrapping_sub(4).wrapping_add(imm as u32);
                        }
                    }
                    Inst::BLTU => {
                        if (self.regfile.x[rs1 as usize] as u32)
                            < (self.regfile.x[rs2 as usize] as u32)
                        {
                            self.pc = self.pc.wrapping_sub(4).wrapping_add(imm as u32);
                        }
                    }
                    Inst::BGEU => {
                        if (self.regfile.x[rs1 as usize] as u32)
                            >= (self.regfile.x[rs2 as usize] as u32)
                        {
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
                let imm = Core::imm_ext_gen(InstType::U, word) as u32;
                match inst {
                    Inst::AUIPC => {
                        if rd > 0 {
                            self.regfile.x[rd as usize] =
                                self.pc.wrapping_sub(4).wrapping_add(imm) as i32;
                        }
                    }
                    Inst::LUI => {
                        if rd > 0 {
                            self.regfile.x[rd as usize] = imm as i32;
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
                            self.regfile.x[rd as usize] = self.csr[csr as usize] as i32;
                        }
                        self.csr[csr as usize] = self.regfile.x[rs1 as usize] as u32;
                    }
                    Inst::CSRRS => {
                        if rd > 0 {
                            self.regfile.x[rd as usize] = self.csr[csr as usize] as i32;
                        }
                        self.csr[csr as usize] =
                            self.csr[csr as usize] | self.regfile.x[rs1 as usize] as u32;
                    }
                    Inst::CSRRWI => {
                        if rd > 0 {
                            self.regfile.x[rd as usize] = self.csr[csr as usize] as i32;
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
