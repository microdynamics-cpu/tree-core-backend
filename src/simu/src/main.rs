use clap::Parser;
use std::fs::File;
use std::io::Read;
use treecore_simu::core::{Core};
use treecore_simu::config::XLen;
use treecore_simu::cli::cli_mode;

/// RISCV ISA Simulator Component
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path of the bin file to simulate
    #[clap(short, long, default_value = "none")]
    bin: String,

    /// Output the trace info
    #[clap(short, long)]
    debug: bool,

    /// Bit width of the processor
    #[clap(short, long, default_value = "x64")]
    xlen: String,

    /// Start addr of the processor
    #[clap(short, long, default_value = "0x80000000")]
    start_addr: String,

    /// End inst
    #[clap(short, long, default_value = "0x0000006b")]
    end_inst: String,

    /// Interactive mode
    #[clap(short, long)]
    inter: bool,

    /// RPC request(http) for simulating keyboard and gpu in server mode
    #[clap(short, long)]
    rpc: bool,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    if args.inter {
        cli_mode();
        return Ok(());
    }

    let mut file = File::open(args.bin)?;
    let mut contents = vec![];
    file.read_to_end(&mut contents)?;
    let mut core = Core::new(
        args.debug,
        match args.xlen.as_str() {
            "x32" => XLen::X32,
            "x64" => XLen::X64,
            _ => panic!(),
        },
        match u64::from_str_radix(args.start_addr.as_str().trim_start_matches("0x"), 16) {
            Ok(v) => v,
            Err(_e) => panic!("need to set the right format!, the right format: 0xXXXX"),
        },
        match u32::from_str_radix(args.end_inst.as_str().trim_start_matches("0x"), 16) {
            Ok(v) => v,
            Err(_e) => panic!("need to set the right format!, the right format: 0xXXXX"),
        },
    );

    // NOTE: launch cmd and server simulator simultaneously can lead to stack overflow bug
    if args.rpc {
        core.run_simu(contents);
    } else {
        core.run_simu(contents);
    }

    Ok(())
}
