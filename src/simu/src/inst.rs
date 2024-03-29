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
    FENCE,
    ECALL,
    EBREAK,
    CSRRW,
    CSRRS,
    CSRRWI,
    URET,
    SRET,
    MRET,
    SFENCEVMA,
    
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
    // RV64M addition
    MULW,
    DIVW,
    DIVUW,
    REMW,
    REMUW,
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
        Inst::URET => "URET",
        Inst::SRET => "SRET",
        Inst::MRET => "MRET",
        Inst::SFENCEVMA => "SFENCE_VMA",
        Inst::FENCE => "FENCE",
        Inst::ECALL => "ECALL",
        Inst::EBREAK => "EBREAK",
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
        Inst::MULW => "MULW",
        Inst::DIVW => "DIVW",
        Inst::DIVUW => "DIVUW",
        Inst::REMW => "REMW",
        Inst::REMUW => "REMUW",
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
        | Inst::URET
        | Inst::SRET
        | Inst::MRET
        | Inst::SFENCEVMA
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
        | Inst::SRAW
        | Inst::MULW
        | Inst::DIVW
        | Inst::DIVUW
        | Inst::REMW
        | Inst::REMUW => InstType::R,
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
        | Inst::SRAIW
        | Inst::ECALL
        | Inst::EBREAK => InstType::I,
        Inst::SB | Inst::SH | Inst::SW | Inst::SD => InstType::S,
        Inst::BEQ | Inst::BNE | Inst::BLT | Inst::BGE | Inst::BLTU | Inst::BGEU => InstType::B,
        Inst::LUI | Inst::AUIPC => InstType::U,
        Inst::JAL => InstType::J,
        Inst::CSRRS | Inst::CSRRW | Inst::CSRRWI => InstType::C,
    }
}
