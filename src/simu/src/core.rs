use crate::csr;
use crate::data::Word;
use crate::decode::Decode;
use crate::inst::{get_inst_name, get_instruction_type, Inst, InstType};
use crate::mmu::AddrMode;
use crate::mmu::MAType;
use crate::privilege::{get_exception_cause, get_priv_encoding, Exception, PrivMode};
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
            let end = match self.load_word(self.pc, true) {
                Ok(w) => w == 0x0000_0073u32,
                Err(e) => panic!(),
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
        match self.tick_wrap() {
            Ok(()) => {}
            Err(e) => self.handle_trap(e),
        };
        // regfile_trace(&self.regfile, "ra");
        // regfile_trace(&self.regfile, "sp");
        // regfile_trace(&self.regfile, "a4");
        // regfile_trace(&self.regfile, "t2");
    }

    fn tick_wrap(&mut self) -> Result<(), Exception> {
        let word = match self.fetch() {
            Ok(w) => w,
            Err(e) => return Err(e),
        };
        let inst = Decode::decode(self.pc, word);
        if self.debug {
            inst_trace(self.pc, word, &inst);
        }
        self.exec(word, inst);
        Ok(())
    }

    fn handle_trap(&mut self, excpt: Exception) {
        let cur_priv_encode = get_priv_encoding(&self.priv_mode) as u64;
        self.priv_mode =
            match (self.csr[csr::CSR_MEDELEG_ADDR as usize] >> get_exception_cause(&excpt)) & 1 {
                1u64 => PrivMode::Supervisor,
                0u64 => PrivMode::Machine,
                _ => panic!(),
            };
        match self.priv_mode {
            PrivMode::Supervisor => {
                self.csr[csr::CSR_SCAUSE_ADDR as usize] = get_exception_cause(&excpt);
                self.csr[csr::CSR_STVAL_ADDR as usize] = self.pc;
                self.pc = self.csr[csr::CSR_STVEC_ADDR as usize];
                // override SPP bit[8] with the current privilege mode encoding
                self.csr[csr::CSR_SSTATUS_ADDR as usize] =
                    (self.csr[csr::CSR_SSTATUS_ADDR as usize] & !0x100)
                        | ((cur_priv_encode & 1) << 8);
            }
            PrivMode::Machine => {
                self.csr[csr::CSR_MCAUSE_ADDR as usize] = get_exception_cause(&excpt);
                self.csr[csr::CSR_MTVAL_ADDR as usize] = self.pc;
                self.pc = self.csr[csr::CSR_MTVEC_ADDR as usize];
                // override MPP bits[12:11] with the current privilege mode encoding
                self.csr[csr::CSR_MSTATUS_ADDR as usize] =
                    (self.csr[csr::CSR_MSTATUS_ADDR as usize] & !0x1800)
                        | ((cur_priv_encode & 1) << 11);
            }
            _ => panic!(),
        }
    }

    fn fetch(&mut self) -> Result<u32, Exception> {
        let word = match self.load_word(self.pc, true) {
            Ok(w) => w,
            Err(e) => {
                self.pc = self.pc.wrapping_add(4);
                return Err(Exception::InstPageFault);
            }
        };
        self.pc = self.pc.wrapping_add(4);
        Ok(word)
    }

    fn load_byte(&self, addr: u64, trans: bool) -> Result<u8, Exception> {
        let phy_addr = match trans {
            true => addr, // HACK: fake
            false => addr,
        };
        // HACK: bound check!!
        Ok(self.mem[match self.xlen {
            XLen::X32 => addr & 0xFFFF_FFFF,
            XLen::X64 => addr,
        } as usize])
    }

    fn load_halfword(&self, addr: u64, trans: bool) -> Result<u16, Exception> {
        let mut res = 0u16;
        for i in 0..2 {
            match self.load_byte(addr.wrapping_add(i), trans) {
                Ok(v) => res |= (v as u16) << (8 * i),
                Err(e) => return Err(e),
            }
        }
        Ok(res)
    }

    fn load_word(&self, addr: u64, trans: bool) -> Result<u32, Exception> {
        let mut res = 0u32;
        for i in 0..2 {
            match self.load_halfword(addr.wrapping_add(i * 2), trans) {
                Ok(v) => res |= (v as u32) << (8 * i * 2),
                Err(e) => return Err(e),
            }
        }
        Ok(res)
    }

    fn load_doubleword(&self, addr: u64, trans: bool) -> Result<u64, Exception> {
        let mut res = 0u64;
        for i in 0..2 {
            match self.load_word(addr.wrapping_add(i * 4), trans) {
                Ok(v) => res |= (v as u64) << (8 * i * 4),
                Err(e) => return Err(e),
            }
        }
        Ok(res)
    }

    fn store_byte(&mut self, addr: u64, val: u8, trans: bool) -> Result<(), Exception> {
        let phy_addr = match trans {
            true => addr, // HACK: fake
            false => addr,
        };

        self.mem[match self.xlen {
            XLen::X32 => addr & 0xFFFF_FFFF,
            XLen::X64 => addr,
        } as usize] = val;
        Ok(())
    }

    fn store_halfword(&mut self, addr: u64, val: u16, trans: bool) -> Result<(), Exception> {
        for i in 0..2 {
            match self.store_byte(
                addr.wrapping_add(i),
                (val >> (8 * i) & 0xFFu16) as u8,
                trans,
            ) {
                Ok(()) => {}
                Err(e) => return Err(e),
            };
        }
        Ok(())
    }

    fn store_word(&mut self, addr: u64, val: u32, trans: bool) -> Result<(), Exception> {
        for i in 0..2 {
            match self.store_halfword(
                addr.wrapping_add(i * 2),
                (val >> (8 * i * 2) & 0xFFFFu32) as u16,
                trans,
            ) {
                Ok(()) => {}
                Err(e) => return Err(e),
            };
        }
        Ok(())
    }

    fn store_doubleword(&mut self, addr: u64, val: u64, trans: bool) -> Result<(), Exception> {
        for i in 0..2 {
            match self.store_word(
                addr.wrapping_add(i * 4),
                (val >> (8 * i * 4) & 0xFFFF_FFFFFu64) as u32,
                trans,
            ) {
                Ok(()) => {}
                Err(e) => return Err(e),
            };
        }
        Ok(())
    }

    fn trans_addr(&mut self, addr: u64, ma_type: MAType) -> Result<u64, ()> {
        match self.addr_mode {
            AddrMode::None => Ok(addr),
            AddrMode::SV32 => match self.priv_mode {
                PrivMode::User | PrivMode::Supervisor => {
                    let vpns = [(addr >> 12) & 0x3FF, (addr >> 22) & 0x3FF];
                    // self.trav_page(addr, 2 - 1, self.ppn, &vpns, ma_type)
                    Ok(addr)
                }
                _ => Ok(addr),
            },
            _ => panic!(),
        }
    }

    // fn trav_page(&self, ) -> Result<u64, ()> {

    // }

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

    fn exec(&mut self, word: u32, inst: Inst) -> Result<(), Exception> {
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
                        self.regfile.x[rd as usize] = match self
                            .load_byte(self.regfile.x[rs1 as usize].wrapping_add(imm) as u64, true)
                        {
                            Ok(v) => v as i8 as i64,
                            Err(e) => return Err(e),
                        };
                        // NOTE: convert to i8 is important!!! different from 'LBU'
                        // println!("val: {}", self.regfile.x[rd as usize]);
                    }
                    Inst::LH => {
                        self.regfile.x[rd as usize] = match self.load_halfword(
                            self.regfile.x[rs1 as usize].wrapping_add(imm) as u64,
                            true,
                        ) {
                            Ok(v) => v as i16 as i64,
                            Err(e) => return Err(e),
                        }
                    }
                    Inst::LW => {
                        self.regfile.x[rd as usize] = match self
                            .load_word(self.regfile.x[rs1 as usize].wrapping_add(imm) as u64, true)
                        {
                            Ok(v) => v as i32 as i64,
                            Err(e) => return Err(e),
                        }
                    }
                    Inst::LD => {
                        self.regfile.x[rd as usize] = match self.load_doubleword(
                            self.regfile.x[rs1 as usize].wrapping_add(imm) as u64,
                            true,
                        ) {
                            Ok(v) => v as i64,
                            Err(e) => return Err(e),
                        }
                    }
                    Inst::LBU => {
                        self.regfile.x[rd as usize] = match self
                            .load_byte(self.regfile.x[rs1 as usize].wrapping_add(imm) as u64, true)
                        {
                            Ok(v) => v as i64,
                            Err(e) => return Err(e),
                        }
                    }
                    Inst::LHU => {
                        self.regfile.x[rd as usize] = match self.load_halfword(
                            self.regfile.x[rs1 as usize].wrapping_add(imm) as u64,
                            true,
                        ) {
                            Ok(v) => v as i64,
                            Err(e) => return Err(e),
                        }
                    }
                    Inst::LWU => {
                        self.regfile.x[rd as usize] = match self
                            .load_word(self.regfile.x[rs1 as usize].wrapping_add(imm) as u64, true)
                        {
                            Ok(v) => v as u32 as i64,
                            Err(e) => return Err(e),
                        }
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
                        match self.store_byte(
                            self.regfile.x[rs1 as usize].wrapping_add(offset) as u64,
                            (self.regfile.x[rs2 as usize] as u8) & 0xFFu8,
                            true,
                        ) {
                            Ok(()) => {}
                            Err(e) => return Err(e),
                        };
                    }
                    Inst::SH => {
                        match self.store_halfword(
                            self.regfile.x[rs1 as usize].wrapping_add(offset) as u64,
                            (self.regfile.x[rs2 as usize] as u64 as u16) & 0xFFFFu16,
                            true,
                        ) {
                            Ok(()) => {}
                            Err(e) => return Err(e),
                        };
                    }
                    Inst::SW => {
                        match self.store_word(
                            self.regfile.x[rs1 as usize].wrapping_add(offset) as u64,
                            self.regfile.x[rs2 as usize] as u32,
                            true,
                        ) {
                            Ok(()) => {}
                            Err(e) => return Err(e),
                        };
                    }
                    Inst::SD => {
                        match self.store_doubleword(
                            self.regfile.x[rs1 as usize].wrapping_add(offset) as u64,
                            self.regfile.x[rs2 as usize] as u64,
                            true,
                        ) {
                            Ok(()) => {}
                            Err(e) => return Err(e),
                        };
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
        Ok(())
    }
}
