use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::Read;
use clap::Parser;
use treecore_simu::core::{Core, XLen};
use treecore_simu::shell::{Shell, ShellIO};

fn interactive_mode() {
    println!("TreeCore RISCV ISA Simulator 0.0.1");
    println!("[last-release] on Ubuntu 20.04 LTS");
    println!("Type 'help' for more information.");
    let mut shell = Shell::new(());
    shell.new_command_noargs("hello", "Say 'hello' to the world", |io, _| {
        writeln!(io, "Hello World !!!")?;
        Ok(())
    });

    shell.run_loop(&mut ShellIO::default());
}


/// RISCV ISA Simulator Component
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Name of the bin file to simulate
    #[clap(short, long)]
    bin: String,

    /// Output the trace info
    #[clap(short, long)]
    debug: bool,

    /// Bit width of the processor
    #[clap(short, long)]
    xlen: String,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    // println!("path: {}", args.bin);
    // println!("xlen: {}", args.xlen);
    // println!("debug: {}", args.debug);
    let mut file = File::open(args.bin)?;
    let mut contents = vec![];
    file.read_to_end(&mut contents)?;
    let mut core = Core::new(args.debug, match args.xlen.as_str() {
        "x32" => XLen::X32,
        "x64" => XLen::X64,
        _ => panic!(),
    });
    core.run_simu(contents);

    Ok(())
}
