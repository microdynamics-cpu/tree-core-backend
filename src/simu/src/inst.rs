pub enum Inst {
    // RV32I
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
    // RV32M
    MUL,
    MULH,
    MULHSU,
    MULHU,
    DIV,
    DIVU,
    REM,
    REMU,
    // RV64I addition
    LWU,
    LD,
    SD,
    ADDIW,
    SLLIW,
    SRLIW,
    SRAIW,
    ADDW,
    SUBW,
    SLLW,
    SRLW,
    SRAW,
}

pub enum InstType {
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

pub fn get_inst_name(inst: &Inst) -> &'static str {
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
        Inst::MUL => "MUL",
        Inst::MULH => "MULH",
        Inst::MULHSU => "MULHSU",
        Inst::MULHU => "MULHU",
        Inst::DIV => "DIV",
        Inst::DIVU => "DIVU",
        Inst::REM => "REM",
        Inst::REMU => "REMU",
        Inst::LWU => "LWU",
        Inst::LD => "LD",
        Inst::SD => "SD",
        Inst::ADDIW => "ADDIW",
        Inst::SLLIW => "SLLIW",
        Inst::SRLIW => "SRLIW",
        Inst::SRAIW => "SRAIW",
        Inst::ADDW => "ADDW",
        Inst::SUBW => "SUBW",
        Inst::SLLW => "SLLW",
        Inst::SRLW => "SRLW",
        Inst::SRAW => "SRAW",
    }
}

pub fn get_instruction_type(inst: &Inst) -> InstType {
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
        | Inst::MRET
        | Inst::MUL
        | Inst::MULH
        | Inst::MULHSU
        | Inst::MULHU
        | Inst::DIV
        | Inst::DIVU
        | Inst::REM
        | Inst::REMU 
        | Inst::ADDW
        | Inst::SUBW
        | Inst::SLLW
        | Inst::SRLW
        | Inst::SRAW => InstType::R,
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
        | Inst::FENCE
        | Inst::LWU
        | Inst::LD
        | Inst::ADDIW 
        | Inst::SLLIW
        | Inst::SRLIW
        | Inst::SRAIW => InstType::I,
        Inst::SB | Inst::SH | Inst::SW | Inst::SD => InstType::S,
        Inst::BEQ | Inst::BNE | Inst::BLT | Inst::BGE | Inst::BLTU | Inst::BGEU => InstType::B,
        Inst::LUI | Inst::AUIPC => InstType::U,
        Inst::JAL => InstType::J,
        Inst::CSRRS | Inst::CSRRW | Inst::CSRRWI => InstType::C,
    }
}
