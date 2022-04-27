pub enum AddrMode {
    None,
    SV32,
    SV39,
    SV48,
}

pub enum MAType {
    Read,
    Write,
}

pub fn get_addr_mode_name(mode: &AddrMode) -> &'static str {
    match mode {
        AddrMode::None => "None",
        AddrMode::SV32 => "SV32",
        AddrMode::SV39 => "SV39",
        AddrMode::SV48 => "SV48",
    }
}
