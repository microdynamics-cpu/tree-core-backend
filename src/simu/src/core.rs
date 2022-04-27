use crate::data::Word;
use crate::decode::Decode;
use crate::inst::{get_inst_name, get_instruction_type, Inst, InstType};
use crate::mmu::AddrMode;
use crate::privilege::PrivMode;
use crate::regfile::Regfile;
use crate::trace::{inst_trace, regfile_trace};

const START_ADDR: u64 = 0x1000u64;
const MEM_CAPACITY: usize = 1024 * 512;
const CSR_CAPACITY: usize = 4096;

pub struct Core {
    regfile: Regfile,
    pc: u64,
    ppn: u64,
    priv_mode: PrivMode,
    addr_mode: AddrMode,
    csr: [u64; CSR_CAPACITY],
    mem: [u8; MEM_CAPACITY],
    inst_num: u64,
    xlen: XLen,
    debug: bool,
}

#[derive(Debug)]
pub enum XLen {
    X32,
    X64,
}

impl Core {
    pub fn new(debug_val: bool, xlen_val: XLen) -> Self {
        Core {
            regfile: Regfile::new(),
            pc: 0u64,
            ppn: 0u64,
            priv_mode: PrivMode::Machine,
            addr_mode: AddrMode::None,
            csr: [0; CSR_CAPACITY], // NOTE: need to prepare specific val for reg, such as mhardid
            mem: [0; MEM_CAPACITY],
            inst_num: 0u64,
            xlen: xlen_val,
            debug: debug_val,
        }
    }

    pub fn run_simu(&mut self, data: Vec<u8>) {
        for i in 0..data.len() {
            self.mem[i] = data[i];
        }

        self.pc = START_ADDR;
        loop {
            // println!("val: {:08x}", self.load_word(self.pc));
            let end = match self.load_word(self.pc) {
                0x0000_0073 => true,
                _ => false,
            };

            if end {
                match self.regfile.x[10] {
                    0 => println!("\x1b[92mTest Passed, inst_num: {}\x1b[0m", self.inst_num),
                    _ => println!("\x1b[91mTest Failed\x1b[0m"),
                };
                break;
            }

            self.tick();
            self.inst_num += 1;
        }
    }

    fn tick(&mut self) {
        let word = self.fetch();
        let inst = Decode::decode(self.pc, word);
        if self.debug {
            inst_trace(self.pc, word, &inst);
        }
        self.exec(word, inst);
        // regfile_trace(&self.regfile, "ra");
        // regfile_trace(&self.regfile, "sp");
        // regfile_trace(&self.regfile, "a4");
        // regfile_trace(&self.regfile, "t2");
    }

    fn fetch(&mut self) -> u32 {
        let word = self.load_word(self.pc);
        self.pc = self.pc.wrapping_add(4);
        word
    }

    fn load_byte(&self, addr: u64) -> u8 {
        self.mem[match self.xlen {
            XLen::X32 => addr & 0xFFFF_FFFF,
            XLen::X64 => addr,
        } as usize]
    }

    fn load_halfword(&self, addr: u64) -> u16 {
        ((self.load_byte(addr.wrapping_add(1)) as u16) << 8) | (self.load_byte(addr) as u16)
    }

    fn load_word(&self, addr: u64) -> u32 {
        ((self.load_halfword(addr.wrapping_add(2)) as u32) << 16)
            | (self.load_halfword(addr) as u32)
    }

    fn load_doubleword(&self, addr: u64) -> u64 {
        ((self.load_word(addr.wrapping_add(4)) as u64) << 32) | (self.load_word(addr) as u64)
    }

    fn store_byte(&mut self, addr: u64, val: u8) {
        self.mem[match self.xlen {
            XLen::X32 => addr & 0xFFFF_FFFF,
            XLen::X64 => addr,
        } as usize] = val;
    }

    fn store_halfword(&mut self, addr: u64, val: u16) {
        self.store_byte(addr, (val & 0xFFu16) as u8);
        self.store_byte(addr.wrapping_add(1), ((val >> 8) & 0xFFu16) as u8);
    }

    fn store_word(&mut self, addr: u64, val: u32) {
        self.store_halfword(addr, (val & 0xFFFFu32) as u16);
        self.store_halfword(addr.wrapping_add(2), ((val >> 16) & 0xFFFFu32) as u16);
    }

    fn store_doubleword(&mut self, addr: u64, val: u64) {
        self.store_word(addr, (val & 0xFFFF_FFFFFu64) as u32);
        self.store_word(addr.wrapping_add(4), ((val >> 32) & 0xFFFF_FFFFu64) as u32);
    }

    fn imm_ext_gen(inst_type: InstType, word: u32) -> i64 {
        let inst = Word::new(word);
        match inst_type {
            InstType::I => {
                // imm[31:11] = inst[31]
                // imm[10:0] = inst[30:20]
                return (match inst.val(31, 31) {
                    1 => 0xFFFF_F800,
                    0 => 0,
                    _ => panic!(),
                } | (inst.pos(30, 20, 0))) as i32 as i64;
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
                    | (inst.pos(30, 21, 1))) as i32 as i64;
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
                    | (inst.pos(11, 8, 1))) as i32 as i64;
            }
            InstType::S => {
                return (match inst.val(31, 31) {
                    1 => 0xFFFF_F000,
                    0 => 0,
                    _ => panic!(),
                } | (inst.pos(31, 25, 5))
                    | (inst.pos(11, 7, 0))) as i32 as i64;
            }
            InstType::U => (word & 0xFFFF_F000) as i32 as i64,
            _ => {
                panic!();
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
                            let shamt = (imm
                                & match self.xlen {
                                    XLen::X32 => 0x1F,
                                    XLen::X64 => 0x3F,
                                }) as u32;
                            self.regfile.x[rd as usize] = self.regfile.x[rs1 as usize] << shamt;
                        }
                    }
                    Inst::SLTI => {
                        if rd > 0 {
                            match self.xlen {
                                XLen::X32 => {
                                    if (self.regfile.x[rs1 as usize] as i32) < (imm as i32) {
                                        self.regfile.x[rd as usize] = 1 as i64;
                                    } else {
                                        self.regfile.x[rd as usize] = 0 as i64;
                                    }
                                }
                                XLen::X64 => {
                                    if self.regfile.x[rs1 as usize] < imm {
                                        self.regfile.x[rd as usize] = 1 as i64;
                                    } else {
                                        self.regfile.x[rd as usize] = 0 as i64;
                                    }
                                }
                            }
                        }
                    }
                    Inst::SLTIU => {
                        if rd > 0 {
                            if (self.regfile.x[rs1 as usize] as u64) < (imm as u64) {
                                self.regfile.x[rd as usize] = 1 as i64;
                            } else {
                                self.regfile.x[rd as usize] = 0 as i64;
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
                        if rd > 0 {
                            self.regfile.x[rd as usize] = self.regfile.x[rs1 as usize] & imm;
                        }
                    }
                    Inst::SRLI => {
                        if rd > 0 {
                            let shamt = (imm
                                & match self.xlen {
                                    XLen::X32 => 0x1F,
                                    XLen::X64 => 0x3F,
                                }) as u32;

                            match self.xlen {
                                XLen::X32 => {
                                    self.regfile.x[rd as usize] =
                                        ((self.regfile.x[rs1 as usize] as u32) >> shamt) as i64;
                                }
                                XLen::X64 => {
                                    self.regfile.x[rd as usize] =
                                        ((self.regfile.x[rs1 as usize] as u64) >> shamt) as i64;
                                }
                            }
                        }
                    }
                    Inst::SRAI => {
                        if rd > 0 {
                            let shamt = (imm
                                & match self.xlen {
                                    XLen::X32 => 0x1F,
                                    XLen::X64 => 0x3F,
                                }) as u32;

                            match self.xlen {
                                XLen::X32 => {
                                    self.regfile.x[rd as usize] =
                                        ((self.regfile.x[rs1 as usize] as i32) >> shamt) as i64;
                                }
                                XLen::X64 => {
                                    self.regfile.x[rd as usize] =
                                        self.regfile.x[rs1 as usize] >> shamt;
                                }
                            }
                        }
                    }
                    Inst::ADDIW => {
                        if rd > 0 {
                            self.regfile.x[rd as usize] =
                                (self.regfile.x[rs1 as usize].wrapping_add(imm)) as i32 as i64;
                        }
                    }
                    Inst::SLLIW => {
                        if rd > 0 {
                            let shamt = (imm & 0x3F) as u32;
                            self.regfile.x[rd as usize] =
                                self.regfile.x[rs1 as usize].wrapping_shl(shamt) as i32 as i64;
                        }
                    }
                    Inst::SRLIW => {
                        if rd > 0 {
                            let shamt = (imm & 0x3F) as u32;
                            self.regfile.x[rd as usize] =
                                (self.regfile.x[rs1 as usize] as u64 as u32).wrapping_shr(shamt)
                                    as i32 as i64;
                        }
                    }
                    Inst::SRAIW => {
                        if rd > 0 {
                            let shamt = (imm & 0x3F) as u32;
                            self.regfile.x[rd as usize] =
                                (self.regfile.x[rs1 as usize] as i32).wrapping_shr(shamt) as i32
                                    as i64;
                        }
                    }
                    Inst::JALR => {
                        let tmp_pc = self.pc; // important!!!, if rs1 == rd
                        self.pc = (self.regfile.x[rs1 as usize] as u64).wrapping_add(imm as u64);
                        if rd > 0 {
                            self.regfile.x[rd as usize] = tmp_pc as i64;
                        }
                    }
                    Inst::LB => {
                        self.regfile.x[rd as usize] = self
                            .load_byte(self.regfile.x[rs1 as usize].wrapping_add(imm) as u64)
                            as i8 as i64; // NOTE: convert to i8 is important!!! different from 'LBU'
                                          // println!("val: {}", self.regfile.x[rd as usize]);
                    }
                    Inst::LH => {
                        self.regfile.x[rd as usize] = self
                            .load_halfword(self.regfile.x[rs1 as usize].wrapping_add(imm) as u64)
                            as i16 as i64;
                    }
                    Inst::LW => {
                        self.regfile.x[rd as usize] = self
                            .load_word(self.regfile.x[rs1 as usize].wrapping_add(imm) as u64)
                            as i32 as i64;
                    }
                    Inst::LD => {
                        self.regfile.x[rd as usize] = self
                            .load_doubleword(self.regfile.x[rs1 as usize].wrapping_add(imm) as u64)
                            as i64;
                    }
                    Inst::LBU => {
                        self.regfile.x[rd as usize] = self
                            .load_byte(self.regfile.x[rs1 as usize].wrapping_add(imm) as u64)
                            as i64;
                    }
                    Inst::LHU => {
                        self.regfile.x[rd as usize] = self
                            .load_halfword(self.regfile.x[rs1 as usize].wrapping_add(imm) as u64)
                            as i64;
                    }
                    Inst::LWU => {
                        self.regfile.x[rd as usize] = self
                            .load_word(self.regfile.x[rs1 as usize].wrapping_add(imm) as u64)
                            as u32 as i64;
                    }
                    Inst::FENCE => {
                        // HACK: no impl
                    }
                    Inst::ECALL => {
                        // HACK: no impl
                    }
                    Inst::EBREAK => {
                        // HACK: no impl
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
                    Inst::MUL => {
                        if rd > 0 {
                            self.regfile.x[rd as usize] = self.regfile.x[rs1 as usize]
                                .wrapping_mul(self.regfile.x[rs2 as usize]);
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
                    Inst::MULH => {
                        if rd > 0 {
                            let tmp = (self.regfile.x[rs1 as usize] as i128)
                                * (self.regfile.x[rs2 as usize] as i128);
                            self.regfile.x[rd as usize] = match self.xlen {
                                XLen::X32 => (tmp >> 32) as i64,
                                XLen::X64 => (tmp >> 64) as i64,
                            };
                        }
                    }
                    Inst::SLT => {
                        if rd > 0 {
                            match self.xlen {
                                XLen::X32 => {
                                    if (self.regfile.x[rs1 as usize] as i32)
                                        < (self.regfile.x[rs2 as usize] as i32)
                                    {
                                        self.regfile.x[rd as usize] = 1;
                                    } else {
                                        self.regfile.x[rd as usize] = 0;
                                    }
                                }
                                XLen::X64 => {
                                    if self.regfile.x[rs1 as usize] < self.regfile.x[rs2 as usize] {
                                        self.regfile.x[rd as usize] = 1;
                                    } else {
                                        self.regfile.x[rd as usize] = 0;
                                    }
                                }
                            }
                        }
                    }
                    Inst::MULHSU => {
                        if rd > 0 {
                            self.regfile.x[rd as usize] = match self.xlen {
                                XLen::X32 => {
                                    let tmp = (self.regfile.x[rs1 as usize] as i128)
                                        * (self.regfile.x[rs2 as usize] as u32 as i128);
                                    (tmp >> 32) as i64
                                }
                                XLen::X64 => {
                                    let tmp = (self.regfile.x[rs1 as usize] as i128)
                                        * (self.regfile.x[rs2 as usize] as u64 as i128);
                                    (tmp >> 64) as i64
                                }
                            };
                        }
                    }
                    Inst::SLTU => {
                        if rd > 0 {
                            if (self.regfile.x[rs1 as usize] as u64)
                                < (self.regfile.x[rs2 as usize] as u64)
                            {
                                self.regfile.x[rd as usize] = 1i64;
                            } else {
                                self.regfile.x[rd as usize] = 0i64;
                            }
                        }
                    }
                    Inst::MULHU => {
                        if rd > 0 {
                            self.regfile.x[rd as usize] = match self.xlen {
                                XLen::X32 => {
                                    let tmp = (self.regfile.x[rs1 as usize] as u32 as u64)
                                        * (self.regfile.x[rs2 as usize] as u32 as u64);
                                    (tmp >> 32) as i32 as i64
                                }
                                XLen::X64 => {
                                    let tmp = (self.regfile.x[rs1 as usize] as u64 as u128)
                                        * (self.regfile.x[rs2 as usize] as u64 as u128);
                                    (tmp >> 64) as i64
                                }
                            };
                        }
                    }
                    Inst::XOR => {
                        if rd > 0 {
                            self.regfile.x[rd as usize] =
                                self.regfile.x[rs1 as usize] ^ self.regfile.x[rs2 as usize];
                        }
                    }
                    Inst::DIV => {
                        if rd > 0 {
                            self.regfile.x[rd as usize] = match self.regfile.x[rs2 as usize] {
                                0 => -1i64,
                                _ => self.regfile.x[rs1 as usize]
                                    .wrapping_div(self.regfile.x[rs2 as usize]),
                            }
                        }
                    }
                    Inst::SRL => {
                        if rd > 0 {
                            match self.xlen {
                                XLen::X32 => {
                                    self.regfile.x[rd as usize] =
                                        ((self.regfile.x[rs1 as usize] as u32).wrapping_shr(
                                            (self.regfile.x[rs2 as usize] & 0x1Fi64) as u32,
                                        )) as i64;
                                }
                                XLen::X64 => {
                                    self.regfile.x[rd as usize] =
                                        ((self.regfile.x[rs1 as usize] as u64).wrapping_shr(
                                            (self.regfile.x[rs2 as usize] & 0x3Fi64) as u32,
                                        )) as i64;
                                }
                            }
                        }
                    }
                    Inst::DIVU => {
                        if rd > 0 {
                            self.regfile.x[rd as usize] = match self.regfile.x[rs2 as usize] {
                                0 => -1i64,
                                _ => match self.xlen {
                                    XLen::X32 => (self.regfile.x[rs1 as usize] as u32)
                                        .wrapping_div(self.regfile.x[rs2 as usize] as u32)
                                        as i64,
                                    XLen::X64 => (self.regfile.x[rs1 as usize] as u64)
                                        .wrapping_div(self.regfile.x[rs2 as usize] as u64)
                                        as i64,
                                },
                            }
                        }
                    }
                    Inst::SRA => {
                        if rd > 0 {
                            match self.xlen {
                                XLen::X32 => {
                                    self.regfile.x[rd as usize] =
                                        ((self.regfile.x[rs1 as usize] as i32).wrapping_shr(
                                            (self.regfile.x[rs2 as usize] & 0x1Fi64) as u32,
                                        )) as i64;
                                }
                                XLen::X64 => {
                                    self.regfile.x[rd as usize] = self.regfile.x[rs1 as usize]
                                        .wrapping_shr(
                                            (self.regfile.x[rs2 as usize] & 0x3Fi64) as u32,
                                        );
                                }
                            }
                        }
                    }
                    Inst::OR => {
                        if rd > 0 {
                            self.regfile.x[rd as usize] =
                                self.regfile.x[rs1 as usize] | self.regfile.x[rs2 as usize];
                        }
                    }
                    Inst::REM => {
                        if rd > 0 {
                            self.regfile.x[rd as usize] = match self.regfile.x[rs2 as usize] {
                                0 => self.regfile.x[rs1 as usize],
                                _ => self.regfile.x[rs1 as usize]
                                    .wrapping_rem(self.regfile.x[rs2 as usize]),
                            }
                        }
                    }
                    Inst::AND => {
                        if rd > 0 {
                            self.regfile.x[rd as usize] =
                                self.regfile.x[rs1 as usize] & self.regfile.x[rs2 as usize];
                        }
                    }
                    Inst::REMU => {
                        if rd > 0 {
                            self.regfile.x[rd as usize] = match self.regfile.x[rs2 as usize] {
                                0 => self.regfile.x[rs1 as usize],
                                _ => {
                                    ((self.regfile.x[rs1 as usize] as u64)
                                        .wrapping_rem(self.regfile.x[rs2 as usize] as u64))
                                        as i64
                                }
                            }
                        }
                    }
                    Inst::ADDW => {
                        if rd > 0 {
                            self.regfile.x[rd as usize] = self.regfile.x[rs1 as usize]
                                .wrapping_add(self.regfile.x[rs2 as usize])
                                as i32
                                as i64;
                        }
                    }
                    Inst::MULW => {
                        if rd > 0 {
                            let tmp = (self.regfile.x[rs1 as usize] as i128)
                                * (self.regfile.x[rs2 as usize] as i128);
                            self.regfile.x[rd as usize] = tmp as i32 as i64;
                        }
                    }
                    Inst::SUBW => {
                        if rd > 0 {
                            self.regfile.x[rd as usize] = self.regfile.x[rs1 as usize]
                                .wrapping_sub(self.regfile.x[rs2 as usize])
                                as i32
                                as i64;
                        }
                    }
                    Inst::SLLW => {
                        if rd > 0 {
                            self.regfile.x[rd as usize] = (self.regfile.x[rs1 as usize] as u64)
                                .wrapping_shl((self.regfile.x[rs2 as usize] & 0x1Fi64) as u32)
                                as i32
                                as i64;
                        }
                    }
                    Inst::DIVW => {
                        if rd > 0 {
                            self.regfile.x[rd as usize] = match self.regfile.x[rs2 as usize] {
                                0 => -1i64,
                                _ => (self.regfile.x[rs1 as usize] as i32)
                                    .wrapping_div(self.regfile.x[rs2 as usize] as i32)
                                    as i64,
                            }
                        }
                    }
                    Inst::SRLW => {
                        if rd > 0 {
                            self.regfile.x[rd as usize] =
                                (self.regfile.x[rs1 as usize] as u64 as u32)
                                    .wrapping_shr((self.regfile.x[rs2 as usize] & 0x1Fi64) as u32)
                                    as i32 as i64;
                        }
                    }
                    Inst::DIVUW => {
                        if rd > 0 {
                            self.regfile.x[rd as usize] = match self.regfile.x[rs2 as usize] {
                                0 => -1i64,
                                _ => (self.regfile.x[rs1 as usize] as u32)
                                    .wrapping_div(self.regfile.x[rs2 as usize] as u32)
                                    as i32 as i64,
                            }
                        }
                    }
                    Inst::SRAW => {
                        if rd > 0 {
                            self.regfile.x[rd as usize] = (self.regfile.x[rs1 as usize] as i32)
                                .wrapping_shr((self.regfile.x[rs2 as usize] & 0x1Fi64) as u32)
                                as i32
                                as i64;
                        }
                    }
                    Inst::REMW => {
                        if rd > 0 {
                            self.regfile.x[rd as usize] = match self.regfile.x[rs2 as usize] {
                                0 => self.regfile.x[rs1 as usize],
                                _ => (self.regfile.x[rs1 as usize] as i32)
                                    .wrapping_rem(self.regfile.x[rs2 as usize] as i32)
                                    as i64,
                            }
                        }
                    }
                    Inst::REMUW => {
                        if rd > 0 {
                            self.regfile.x[rd as usize] = match self.regfile.x[rs2 as usize] {
                                0 => self.regfile.x[rs1 as usize],
                                _ => (self.regfile.x[rs1 as usize] as u32)
                                    .wrapping_rem(self.regfile.x[rs2 as usize] as u32)
                                    as i32 as i64,
                            }
                        }
                    }
                    Inst::URET => {}
                    Inst::SRET => {}
                    Inst::SFENCEVMA => {}
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
                            self.regfile.x[rs1 as usize].wrapping_add(offset) as u64,
                            (self.regfile.x[rs2 as usize] as u8) & 0xFFu8,
                        );
                    }
                    Inst::SH => {
                        self.store_halfword(
                            self.regfile.x[rs1 as usize].wrapping_add(offset) as u64,
                            (self.regfile.x[rs2 as usize] as u64 as u16) & 0xFFFFu16,
                        );
                    }
                    Inst::SW => {
                        self.store_word(
                            self.regfile.x[rs1 as usize].wrapping_add(offset) as u64,
                            self.regfile.x[rs2 as usize] as u32,
                        );
                    }
                    Inst::SD => {
                        self.store_doubleword(
                            self.regfile.x[rs1 as usize].wrapping_add(offset) as u64,
                            self.regfile.x[rs2 as usize] as u64,
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
                            self.regfile.x[rd as usize] = self.pc as i64;
                        }
                        self.pc = self.pc.wrapping_sub(4).wrapping_add(imm as u64);
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
                            self.pc = self.pc.wrapping_sub(4).wrapping_add(imm as u64);
                        }
                    }
                    Inst::BNE => match self.xlen {
                        XLen::X32 => {
                            if (self.regfile.x[rs1 as usize] as i32)
                                != (self.regfile.x[rs2 as usize] as i32)
                            {
                                self.pc = self.pc.wrapping_sub(4).wrapping_add(imm as u64);
                            }
                        }
                        XLen::X64 => {
                            if self.regfile.x[rs1 as usize] != self.regfile.x[rs2 as usize] {
                                self.pc = self.pc.wrapping_sub(4).wrapping_add(imm as u64);
                            }
                        }
                    },
                    Inst::BLT => {
                        // trace::execpt_handle(self.pc, word);
                        match self.xlen {
                            XLen::X32 => {
                                if (self.regfile.x[rs1 as usize] as i32)
                                    < (self.regfile.x[rs2 as usize] as i32)
                                {
                                    self.pc = self.pc.wrapping_sub(4).wrapping_add(imm as u64);
                                }
                            }
                            XLen::X64 => {
                                if self.regfile.x[rs1 as usize] < self.regfile.x[rs2 as usize] {
                                    self.pc = self.pc.wrapping_sub(4).wrapping_add(imm as u64);
                                }
                            }
                        }
                    }
                    Inst::BGE => {
                        if self.regfile.x[rs1 as usize] >= self.regfile.x[rs2 as usize] {
                            self.pc = self.pc.wrapping_sub(4).wrapping_add(imm as u64);
                        }
                    }
                    Inst::BLTU => {
                        if (self.regfile.x[rs1 as usize] as u64)
                            < (self.regfile.x[rs2 as usize] as u64)
                        {
                            self.pc = self.pc.wrapping_sub(4).wrapping_add(imm as u64);
                        }
                    }
                    Inst::BGEU => {
                        if (self.regfile.x[rs1 as usize] as u64)
                            >= (self.regfile.x[rs2 as usize] as u64)
                        {
                            self.pc = self.pc.wrapping_sub(4).wrapping_add(imm as u64);
                        }
                    }
                    _ => {
                        panic!();
                    }
                }
            }
            InstType::U => {
                let rd = inst_wrap.val(11, 7);
                let imm = Core::imm_ext_gen(InstType::U, word) as u64;
                match inst {
                    Inst::AUIPC => {
                        if rd > 0 {
                            self.regfile.x[rd as usize] =
                                self.pc.wrapping_sub(4).wrapping_add(imm) as i64;
                        }
                    }
                    Inst::LUI => {
                        if rd > 0 {
                            self.regfile.x[rd as usize] = imm as i64;
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
                            self.regfile.x[rd as usize] = self.csr[csr as usize] as i64;
                        }
                        self.csr[csr as usize] = self.regfile.x[rs1 as usize] as u64;
                    }
                    Inst::CSRRS => {
                        if rd > 0 {
                            self.regfile.x[rd as usize] = self.csr[csr as usize] as i64;
                        }
                        self.csr[csr as usize] =
                            self.csr[csr as usize] | self.regfile.x[rs1 as usize] as u64;
                    }
                    Inst::CSRRWI => {
                        if rd > 0 {
                            self.regfile.x[rd as usize] = self.csr[csr as usize] as i64;
                        }
                        self.csr[csr as usize] = rs1 as u64;
                    }
                    _ => {
                        panic!();
                    }
                }
            }
        }
    }
}
