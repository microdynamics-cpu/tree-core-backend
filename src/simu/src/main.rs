use clap::Parser;
use std::fs::File;
use std::io::Read;
use std::sync::mpsc;
use std::thread;
use treecore_simu::cli::cli_mode;
use treecore_simu::config::XLen;
use treecore_simu::core::Core;
use treecore_simu::web::web_init;

/// RISCV ISA Simulator Component
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path of the bin file to simulate
    #[clap(short, long, default_value = "none")]
    bin: String,

    /// Debug level[err, warn, trace, none]
    #[clap(short, long, default_value = "none")]
    debug: String,

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

    /// Web server(http) for simulating keyboard and gpu online
    #[clap(short, long)]
    web: bool,
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
    if args.web {
        println!("web");
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            web_init(tx);
        });

        loop {
            match rx.try_recv() {
                Ok(v) => {
                    println!("Got: {}", v)
                }
                Err(_e) => {}
            }
        }
    } else {
        core.run_simu(contents);
    }

    Ok(())
}
