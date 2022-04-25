pub enum PrivMode {
    User,
    Supervisor,
    Reserved,
    Machine,
}

pub enum Exception {
    EnvCallFromMMode,
    EnvCallFromUMode,
    EnvCallFromSMode,
    IllegalInstruction,
    InstPageFault,
    LoadPageFault,
    StorePageFault,
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
fn get_priv_encoding(mode: &PrivilegeMode) -> u8 {
    match mode {
        PrivMode::User => 0,
        PrivMode::Supervisor => 1,
        PrivMode::Reserved => panic!(),
        PrivMode::Machine => 3,
    }
}

pub fn get_exception_cause(exception: &Exception) -> u64 {
    match exception {
        Exception::IllegalInstruction => 2,
        Exception::EnvCallFromUMode => 8,
        Exception::EnvCallFromSMode => 9,
        Exception::EnvCallFromMMode => 11,
        Exception::InstPageFault => 12,
        Exception::LoadPageFault => 13,
        Exception::StorePageFault => 15,
    }
}
