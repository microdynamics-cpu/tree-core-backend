use crate::data::Word;
use crate::inst::Inst;
use crate::trace;

pub struct Decode {}

impl Decode {
    pub fn decode(pc: u32, word: u32) -> Inst {
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
                        trace::execpt_handle(pc, word);
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
                        0x01 => Inst::MUL,
                        0x20 => Inst::SUB,
                        _ => panic!(),
                    },
                    1 => match func7 {
                        0x00 => Inst::SLL,
                        0x01 => Inst::MULH,
                        _ => panic!(),
                    },
                    2 => match func7 {
                        0x00 => Inst::SLT,
                        0x01 => Inst::MULHSU,
                        _ => panic!(),
                    },
                    3 => match func7 {
                        0x00 => Inst::SLTU,
                        0x01 => Inst::MULHU,
                        _ => panic!(),
                    },
                    4 => match func7 {
                        0x00 => Inst::XOR,
                        0x01 => Inst::DIV,
                        _ => panic!(),
                    },
                    5 => match func7 {
                        0x00 => Inst::SRL,
                        0x01 => Inst::DIVU,
                        0x20 => Inst::SRA,
                        _ => panic!(),
                    },
                    6 => match func7 {
                        0x00 => Inst::OR,
                        0x01 => Inst::REM,
                        _ => panic!(),
                    },
                    7 => match func7 {
                        0x00 => Inst::AND,
                        0x01 => Inst::REMU,
                        _ => panic!(),
                    },
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
                        trace::execpt_handle(pc, word);
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
                        trace::execpt_handle(pc, word);
                        panic!()
                    }
                };
            }
            _ => {
                trace::execpt_handle(pc, word);
                panic!()
            }
        }
    }
}
