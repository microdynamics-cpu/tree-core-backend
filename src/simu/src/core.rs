use crate::config::XLen;
use crate::csr;
use crate::data::Word;
use crate::decode::Decode;
use crate::device::Device;
use crate::inst::{get_inst_name, get_instruction_type, Inst, InstType};
use crate::mmu::{AddrMode, MAType};
use crate::privilege::{
    get_exception_cause, get_priv_encoding, Exception, ExceptionType, PrivMode,
};
use crate::regfile::Regfile;
use crate::trace::{itrace, log, rtrace, FTrace};
use std::sync::mpsc;

// const self.start_addr: u64 = 0x1000u64;
const MEM_CAPACITY: usize = 60 * 1024 * 1024;
const CSR_CAPACITY: usize = 4096;
const PERIF_START_ADDR: u64 = 0xa1000000u64;
const PERIF_ADDR_SIZE: u64 = 0x1000u64;
const SERIAL_START_OFFSET: u64 = 0x3F8u64;
// const SERIAL_ADDR_SIZE: u64 = 0x4u64;
const RTC_START_OFFSET: u64 = 0x48u64;
const RTC_ADDR_SIZE: u64 = 0x08u64; // HACK: addr is surplus?
const KDB_START_OFFSET: u64 = 0x60u64;
const KDB_ADDR_SIZE: u64 = 0x02u64; // only device -> core
const VGA_VGACTL_START_OFFSET: u64 = 0x100u64;
const VGA_SYNC_START_OFFSET: u64 = VGA_VGACTL_START_OFFSET + 4u64;
const VGA_SYNC_ADDR_SIZE: u64 = 0x4u64;
const VGA_FRAME_BUF_ADDR_START: u64 = 0xa0000000u64;
const VGA_FRAME_BUF_ADDR_SIZE: u64 = 0x200000u64;

pub struct Core {
    regfile: Regfile,
    pc: u64,
    start_addr: u64,
    end_inst: u32,
    ppn: u64,
    priv_mode: PrivMode,
    addr_mode: AddrMode,
    csr: [u64; CSR_CAPACITY],
    mem: Vec<u8>,
    dev: Device,
    inst_num: u64,
    xlen: XLen,
    dbg_level: String,
    trace_type: String,
    ftr: FTrace,
}

impl Core {
    pub fn new(
        dbg_level: String,
        trace_type: String,
        xlen_val: XLen,
        start_addr: u64,
        end_inst: u32,
    ) -> Self {
        Core {
            regfile: Regfile::new(),
            pc: 0u64,
            ppn: 0u64,
            start_addr: start_addr,
            end_inst: end_inst,
            priv_mode: PrivMode::Machine,
            addr_mode: AddrMode::None,
            csr: [0; CSR_CAPACITY], // NOTE: need to prepare specific val for reg, such as mhardid
            mem: Vec::with_capacity(MEM_CAPACITY),
            dev: Device::new(),
            inst_num: 0u64,
            xlen: xlen_val,
            dbg_level: dbg_level,
            trace_type: trace_type,
            ftr: FTrace::new("test"),
        }
    }

    // NOTE: like 'new' oper, but dont reset mem
    pub fn reset(&mut self) {
        self.regfile.reset();
        self.pc = 0u64;
        self.ppn = 0u64;
        self.priv_mode = PrivMode::Machine;
        self.addr_mode = AddrMode::None;
        self.csr = [0; CSR_CAPACITY];
        self.dev.reset();
        self.inst_num = 0u64;
    }

    pub fn load_bin_file(&mut self, data: Vec<u8>) {
        // clear memory
        for _i in 0..MEM_CAPACITY {
            // BUG: error if reset!
            self.mem.push(0);
        }
        for i in 0..data.len() {
            // HACK: 0x8000_0000 need mem map
            self.mem[i] = data[i];
        }
    }

    pub fn check_bound(&self, val: u8) -> u8 {
        if val >= 80 {
            80u8
        } else {
            val
        }
    }

    pub fn run_simu(
        &mut self,
        kdb_rx: Option<mpsc::Receiver<(u8, u8)>>,
        vga_tx: Option<mpsc::Sender<String>>,
    ) {
        self.pc = self.start_addr;

        loop {
            match kdb_rx {
                Some(ref v) => {
                    match v.try_recv() {
                        Ok(mut vv) => {
                            // println!("Got: {:?}", v)
                            // HACK: trim because not support all key detect
                            vv.0 = self.check_bound(vv.0);
                            vv.1 = self.check_bound(vv.1);
                            self.dev.kdb.det(vv.0, vv.1);
                        }
                        Err(_e) => {}
                    }
                }
                None => {}
            }
            // println!("val: {:08x}", self.load_word(self.pc));
            let end = match self.load_word(self.pc, true) {
                Ok(w) => w == self.end_inst,
                Err(_e) => panic!(),
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
            // log!(self.pc);
            if self.dev.vga.sync {
                match vga_tx {
                    Some(ref v) => v.send(self.dev.vga.send_dat()).unwrap(),
                    None => {}
                }
            }
        }
    }

    fn tick(&mut self) {
        match self.tick_wrap() {
            Ok(()) => {}
            Err(e) => self.handle_trap(e),
        };
        // rtrace(&self.regfile, "ra");
        // rtrace(&self.regfile, "sp");
        // rtrace(&self.regfile, "a4");
        // rtrace(&self.regfile, "t2");
    }

    fn tick_wrap(&mut self) -> Result<(), Exception> {
        let word = match self.fetch() {
            Ok(w) => w,
            Err(e) => return Err(e),
        };
        let inst = Decode::decode(self.pc, word, &self.xlen);
        match self.dbg_level.as_str() {
            "trace" => {
                if self.trace_type == "itrace" {
                    itrace(self.pc, word, &inst);
                }
            }
            "err" => {
                if self.trace_type == "rtrace" {
                    rtrace(&self.regfile, "a0");
                }
            } // HACK:
            _ => {}
        }
        self.exec(word, inst)
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
                self.csr[csr::CSR_SEPC_ADDR as usize] = self.pc.wrapping_sub(4);
                self.csr[csr::CSR_SCAUSE_ADDR as usize] = get_exception_cause(&excpt);
                self.csr[csr::CSR_STVAL_ADDR as usize] = excpt.addr;
                self.pc = self.csr[csr::CSR_STVEC_ADDR as usize];
                // override SPP bit[8] with the current privilege mode encoding
                self.csr[csr::CSR_SSTATUS_ADDR as usize] =
                    (self.csr[csr::CSR_SSTATUS_ADDR as usize] & !0x100)
                        | ((cur_priv_encode & 1) << 8);
            }
            PrivMode::Machine => {
                self.csr[csr::CSR_MEPC_ADDR as usize] = self.pc.wrapping_sub(4);
                self.csr[csr::CSR_MCAUSE_ADDR as usize] = get_exception_cause(&excpt);
                self.csr[csr::CSR_MTVAL_ADDR as usize] = excpt.addr;
                self.pc = self.csr[csr::CSR_MTVEC_ADDR as usize];
                // override MPP bits[12:11] with the current privilege mode encoding
                self.csr[csr::CSR_MSTATUS_ADDR as usize] =
                    (self.csr[csr::CSR_MSTATUS_ADDR as usize] & !0x1800)
                        | ((cur_priv_encode & 0x3) << 11);
            }
            _ => panic!(),
        }
    }

    fn fetch(&mut self) -> Result<u32, Exception> {
        let word = match self.load_word(self.pc, true) {
            Ok(w) => w,
            Err(_e) => {
                self.pc = self.pc.wrapping_add(4);
                return Err(Exception {
                    excpt_type: ExceptionType::InstPageFault,
                    addr: self.pc.wrapping_sub(4),
                }); // NOTE: coverage the LoadPageFault
            }
        };
        self.pc = self.pc.wrapping_add(4);
        Ok(word)
    }

    fn mmap_load_oper(&mut self, addr: u64) -> u8 {
        // println!("addr: {:016x}", addr);
        // HACK: range addr check
        if addr == PERIF_START_ADDR + SERIAL_START_OFFSET {
            panic!();
        } else if addr >= PERIF_START_ADDR + RTC_START_OFFSET
            && addr <= PERIF_START_ADDR + RTC_START_OFFSET + RTC_ADDR_SIZE
        {
            self.dev.rtc.val()
        } else if addr >= PERIF_START_ADDR + KDB_START_OFFSET
            && addr < PERIF_START_ADDR + KDB_START_OFFSET + KDB_ADDR_SIZE
        {
            if addr == PERIF_START_ADDR + KDB_START_OFFSET {
                self.dev.kdb.val(false)
            } else if addr == PERIF_START_ADDR + KDB_START_OFFSET + KDB_ADDR_SIZE - 1u64 {
                self.dev.kdb.val(true)
            } else {
                panic!("[kdb] error addr space")
            }
        } else {
            panic!();
        }
    }

    fn mmap_store_oper(&mut self, addr: u64, val: u8) {
        // println!("addr: {:016x}", addr);
        if addr == PERIF_START_ADDR + SERIAL_START_OFFSET {
            self.dev.uart.out(val);
        } else if addr == PERIF_START_ADDR + RTC_START_OFFSET {
            panic!();
        } else if addr >= VGA_FRAME_BUF_ADDR_START
            && addr <= VGA_FRAME_BUF_ADDR_START + VGA_FRAME_BUF_ADDR_SIZE
        {
            //NOTE: need to guard data transfer by use sync flag
            self.dev.vga.store(addr, val);
        } else if addr >= PERIF_START_ADDR + VGA_SYNC_START_OFFSET
            && addr <= PERIF_START_ADDR + VGA_SYNC_START_OFFSET + VGA_SYNC_ADDR_SIZE
        {
            self.dev.vga.set_sync(val);
        } else {
            panic!();
        }
    }

    fn load_phy_mem(&mut self, addr: u64) -> u8 {
        // HACK: boundery check
        if addr < self.start_addr {
            panic!("[load]mem out of boundery");
        }

        if addr >= PERIF_START_ADDR && addr <= PERIF_START_ADDR + PERIF_ADDR_SIZE {
            self.mmap_load_oper(addr) // BUG: bit width!
        } else {
            // HACK: need to dbg_level seek for season
            match self.start_addr {
                0x8000_0000u64 => self.mem[(addr - self.start_addr) as usize],
                _ => self.mem[addr as usize],
            }
        }
    }

    fn load_byte(&mut self, addr: u64, trans: bool) -> Result<u8, Exception> {
        let phy_addr = match trans {
            true => match self.trans_addr(addr, MAType::Read) {
                Ok(v) => v,
                Err(_e) => {
                    return Err(Exception {
                        excpt_type: ExceptionType::LoadPageFault,
                        addr: addr,
                    })
                }
            },
            false => addr,
        };

        Ok(self.load_phy_mem(match self.xlen {
            XLen::X32 => phy_addr & 0xFFFF_FFFF,
            XLen::X64 => phy_addr,
        }))
    }

    fn load_halfword(&mut self, addr: u64, trans: bool) -> Result<u16, Exception> {
        let mut res = 0u16;
        for i in 0..2 {
            match self.load_byte(addr.wrapping_add(i), trans) {
                Ok(v) => res |= (v as u16) << (8 * i),
                Err(e) => return Err(e),
            }
        }
        Ok(res)
    }

    fn load_word(&mut self, addr: u64, trans: bool) -> Result<u32, Exception> {
        let mut res = 0u32;
        for i in 0..2 {
            match self.load_halfword(addr.wrapping_add(i * 2), trans) {
                Ok(v) => res |= (v as u32) << (8 * i * 2),
                Err(e) => return Err(e),
            }
        }
        Ok(res)
    }

    fn load_doubleword(&mut self, addr: u64, trans: bool) -> Result<u64, Exception> {
        let mut res = 0u64;
        for i in 0..2 {
            match self.load_word(addr.wrapping_add(i * 4), trans) {
                Ok(v) => res |= (v as u64) << (8 * i * 4),
                Err(e) => return Err(e),
            }
        }
        Ok(res)
    }

    fn store_phy_mem(&mut self, addr: u64, val: u8) {
        if addr < self.start_addr {
            panic!("[store]mem out of boundery");
        }

        if (addr >= PERIF_START_ADDR && addr <= PERIF_START_ADDR + PERIF_ADDR_SIZE)
            || (addr >= VGA_FRAME_BUF_ADDR_START
                && addr <= VGA_FRAME_BUF_ADDR_START + VGA_FRAME_BUF_ADDR_SIZE)
        {
            self.mmap_store_oper(addr, val);
        } else {
            // HACK: need to dbg_level seek for season
            match self.start_addr {
                0x8000_0000u64 => self.mem[(addr - self.start_addr) as usize] = val,
                _ => self.mem[addr as usize] = val,
            }
        }
    }

    fn store_byte(&mut self, addr: u64, val: u8, trans: bool) -> Result<(), Exception> {
        let phy_addr = match trans {
            true => match self.trans_addr(addr, MAType::Write) {
                Ok(v) => v,
                Err(_e) => {
                    return Err(Exception {
                        excpt_type: ExceptionType::StorePageFault,
                        addr: addr,
                    })
                }
            },
            false => addr,
        };

        self.store_phy_mem(
            match self.xlen {
                XLen::X32 => phy_addr & 0xFFFF_FFFF,
                XLen::X64 => phy_addr,
            },
            val,
        );
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
                    self.trav_page(addr, 2 - 1, self.ppn, &vpns, ma_type)
                }
                _ => Ok(addr),
            },
            AddrMode::SV39 => match self.priv_mode {
                PrivMode::User | PrivMode::Supervisor => {
                    let vpns = [
                        (addr >> 12) & 0x1FF,
                        (addr >> 21) & 0x1FF,
                        (addr >> 30) & 0x1FF,
                    ];
                    self.trav_page(addr, 3 - 1, self.ppn, &vpns, ma_type)
                }
                _ => Ok(addr),
            },
            _ => panic!(), // HACK: not impl for sv48
        }
    }

    fn trav_page(
        &mut self,
        virt_addr: u64,
        level: u8,
        parent_ppn: u64,
        vpns: &[u64],
        ma_type: MAType,
    ) -> Result<u64, ()> {
        let pagesize = 4096;
        let ptesize = match self.addr_mode {
            AddrMode::SV32 => 4,
            _ => 8, // sv39 and sv48
        };

        let pte_addr = parent_ppn * pagesize + vpns[level as usize] * ptesize;
        let pte = match self.addr_mode {
            AddrMode::SV32 => match self.load_word(pte_addr, false) {
                Ok(v) => v as u64,
                Err(_e) => panic!(),
            },
            _ => match self.load_doubleword(pte_addr, false) {
                Ok(v) => v as u64,
                Err(_e) => panic!(),
            },
        };

        let ppn = match self.addr_mode {
            AddrMode::SV32 => (pte >> 10) & 0x3FFFFF,
            AddrMode::SV39 => (pte >> 10) & 0xFFF_FFFF_FFFF,
            _ => panic!(),
        };

        let ppns = match self.addr_mode {
            AddrMode::SV32 => [(pte >> 10) & 0x3FF, (pte >> 20) & 0xFFF, 0],
            AddrMode::SV39 => [
                (pte >> 10) & 0x1FF,
                (pte >> 19) & 0x1FF,
                (pte >> 28) & 0x3FF_FFFF,
            ],
            _ => panic!(),
        };

        let _rsw = (pte >> 8) & 0x3;
        let d = (pte >> 7) & 1;
        let a = (pte >> 6) & 1;
        let _g = (pte >> 5) & 1;
        let _u = (pte >> 4) & 1;
        let x = (pte >> 3) & 1;
        let w = (pte >> 2) & 1;
        let r = (pte >> 1) & 1;
        let v = pte & 1;

        // println!("VA:{:X} Level:{:X} PTE_AD:{:X} PTE:{:X} PPN:{:X} PPN1:{:X} PPN0:{:X}", v_address, level, pte_addr, pte, ppn, ppns[1], ppns[0]);

        if v == 0 || (r == 0 && w == 1) {
            return Err({});
        }

        if r == 0 && x == 0 {
            return match level {
                0 => Err(()),
                _ => self.trav_page(virt_addr, level - 1, ppn, vpns, ma_type),
            };
        }

        if a == 0 {
            return Err(());
        }

        match ma_type {
            MAType::Exec => {
                if x == 0 {
                    return Err({});
                }
            }
            MAType::Read => {
                if r == 0 {
                    return Err({});
                }
            }
            MAType::Write => {
                if d == 0 || w == 0 {
                    return Err(());
                }
            }
        };

        let offset = virt_addr & 0xFFF; // [11:0]
        let phy_addr = match self.addr_mode {
            AddrMode::SV32 => match level {
                1 => {
                    if ppns[0] != 0 {
                        return Err(());
                    }
                    (ppns[1] << 22) | (vpns[0] << 12) | offset
                }
                0 => (ppn << 12) | offset,
                _ => panic!(), // Shouldn't happen
            },
            _ => match level {
                2 => {
                    if ppns[1] != 0 || ppns[0] != 0 {
                        return Err(());
                    }
                    (ppns[2] << 30) | (vpns[1] << 21) | (vpns[0] << 12) | offset
                }
                1 => {
                    if ppns[0] != 0 {
                        return Err(());
                    }
                    (ppns[2] << 30) | (ppns[1] << 21) | (vpns[0] << 12) | offset
                }
                0 => (ppn << 12) | offset,
                _ => panic!(), // Shouldn't happen
            },
        };
        // println!("PA:{:X}", phy_addr);
        Ok(phy_addr)
    }

    fn get_csr_access_priv(&self, addr: u16) -> bool {
        let priv_val = (addr >> 8) & 0x3;
        (priv_val as u8) <= get_priv_encoding(&self.priv_mode)
    }

    fn read_csr(&self, addr: u16) -> Result<u64, Exception> {
        match self.get_csr_access_priv(addr) {
            true => Ok(self.csr[addr as usize]),
            false => Err(Exception {
                excpt_type: ExceptionType::IllegalInst,
                addr: self.pc.wrapping_sub(4),
            }),
        }
    }

    fn write_csr(&mut self, addr: u16, val: u64) -> Result<(), Exception> {
        match self.get_csr_access_priv(addr) {
            true => {
                self.csr[addr as usize] = val;
                if addr == csr::CSR_SATP_ADDR {
                    self.update_addr_mode(val);
                }
                Ok(())
            }
            false => Err(Exception {
                excpt_type: ExceptionType::IllegalInst,
                addr: self.pc.wrapping_sub(4),
            }),
        }
    }
    fn update_addr_mode(&mut self, val: u64) {
        self.addr_mode = match self.xlen {
            XLen::X32 => match val >> 31 {
                0 => AddrMode::None,
                1 => AddrMode::SV32,
                _ => panic!(),
            },
            XLen::X64 => match val >> 60 {
                0 => AddrMode::None,
                8 => AddrMode::SV39,
                9 => AddrMode::SV48,
                _ => panic!(),
            },
        };

        self.ppn = match self.xlen {
            XLen::X32 => val & 0x3FFFFF,
            XLen::X64 => val & 0xFFFFFFFFFFF,
        }
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

                        if self.dbg_level == "trace" && self.trace_type == "ftrace" {
                            match self.ftr.ftrace(tmp_pc.wrapping_sub(4), self.pc) {
                                Ok(()) => {}
                                Err(_e) => panic!(),
                            }
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
                        let excpt_type = match self.priv_mode {
                            PrivMode::User => ExceptionType::EnvCallFromUMode,
                            PrivMode::Supervisor => ExceptionType::EnvCallFromSMode,
                            PrivMode::Machine => ExceptionType::EnvCallFromMMode,
                            _ => panic!(),
                        };

                        return Err(Exception {
                            excpt_type: excpt_type,
                            addr: self.pc.wrapping_sub(4),
                        });
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
                    Inst::SRET => {
                        self.pc = match self.read_csr(csr::CSR_SEPC_ADDR) {
                            Ok(v) => v,
                            Err(e) => return Err(e),
                        };

                        self.priv_mode = match self.csr[csr::CSR_SSTATUS_ADDR as usize] & 0x100u64 {
                            0 => PrivMode::User,
                            _ => {
                                self.csr[csr::CSR_SSTATUS_ADDR as usize] &= !0x100;
                                PrivMode::Supervisor
                            }
                        }
                    }
                    Inst::SFENCEVMA => {}
                    Inst::MRET => {
                        self.pc = match self.read_csr(csr::CSR_MEPC_ADDR) {
                            Ok(v) => v,
                            Err(e) => return Err(e),
                        };

                        self.priv_mode =
                            match (self.csr[csr::CSR_MSTATUS_ADDR as usize] >> 11) & 0x3 {
                                0 => PrivMode::User,
                                1 => PrivMode::Supervisor,
                                3 => PrivMode::Machine,
                                _ => panic!(),
                            };
                    }
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
                        let tmp_pc = self.pc;
                        self.pc = self.pc.wrapping_sub(4).wrapping_add(imm as u64);
                        if self.dbg_level == "trace" && self.trace_type == "ftrace" {
                            match self.ftr.ftrace(tmp_pc.wrapping_sub(4), self.pc) {
                                Ok(()) => {}
                                Err(_e) => panic!(),
                            }
                        }
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
                let csr = inst_wrap.val(31, 20) as u16;

                match inst {
                    Inst::CSRRW => {
                        let dat = match self.read_csr(csr) {
                            Ok(v) => v,
                            Err(e) => return Err(e),
                        };
                        if rd > 0 {
                            self.regfile.x[rd as usize] = dat as i64;
                        }
                        match self.write_csr(csr, self.regfile.x[rs1 as usize] as u64) {
                            Ok(()) => {}
                            Err(e) => return Err(e),
                        };
                    }
                    Inst::CSRRS => {
                        let dat = match self.read_csr(csr) {
                            Ok(v) => v,
                            Err(e) => return Err(e),
                        };
                        if rd > 0 {
                            self.regfile.x[rd as usize] = dat as i64;
                        }
                        match self.write_csr(
                            csr,
                            (self.regfile.x[rd as usize] | self.regfile.x[rs1 as usize]) as u64,
                        ) {
                            Ok(()) => {}
                            Err(e) => return Err(e),
                        };
                    }
                    Inst::CSRRWI => {
                        let dat = match self.read_csr(csr) {
                            Ok(v) => v,
                            Err(e) => return Err(e),
                        };
                        if rd > 0 {
                            self.regfile.x[rd as usize] = dat as i64;
                        }
                        match self.write_csr(csr, rs1 as u64) {
                            Ok(()) => {}
                            Err(e) => return Err(e),
                        };
                        // self.csr[csr as usize] = rs1 as u64;
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
