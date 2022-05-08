#[derive(Debug)]
pub enum PrivMode {
    User,
    Supervisor,
    Reserved,
    Machine,
}


pub enum ExceptionType {
    EnvCallFromMMode,
    EnvCallFromUMode,
    EnvCallFromSMode,
    IllegalInst,
    InstPageFault,
    LoadPageFault,
    StorePageFault,
}

pub struct Exception {
    pub excpt_type: ExceptionType,
    pub addr: u64,
}

pub fn get_priv_mode_name(mode: &PrivMode) -> &'static str {
    match mode {
        PrivMode::User => "User",
        PrivMode::Supervisor => "Supervisor",
        PrivMode::Reserved => "Reserved",
        PrivMode::Machine => "Machine",
    }
}

// bigger number is higher privilege level
pub fn get_priv_encoding(mode: &PrivMode) -> u8 {
    match mode {
        PrivMode::User => 0,
        PrivMode::Supervisor => 1,
        PrivMode::Reserved => panic!(),
        PrivMode::Machine => 3,
    }
}

pub fn get_exception_cause(exception: &Exception) -> u64 {
    match exception.excpt_type {
        ExceptionType::IllegalInst => 2,
        ExceptionType::EnvCallFromUMode => 8,
        ExceptionType::EnvCallFromSMode => 9,
        ExceptionType::EnvCallFromMMode => 11,
        ExceptionType::InstPageFault => 12,
        ExceptionType::LoadPageFault => 13,
        ExceptionType::StorePageFault => 15,
    }
}