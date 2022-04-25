pub enum AddrMode {
    None,
    SV32,
    SV39,
    SV48,
}

pub enum MemoryAccessType {
    Read,
    Write,
}

pub fn get_addr_mode_name(mode: &AddressingMode) -> &'static str {
    match mode {
        AddressingMode::None => "None",
        AddressingMode::SV32 => "SV32",
        AddressingMode::SV39 => "SV39",
        AddressingMode::SV48 => "SV48",
    }
}
