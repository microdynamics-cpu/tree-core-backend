use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::Read;
use treecore_simu::core::Core;
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

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let args_len = args.len();
    if args_len < 2 {
        // eprint!("usage: treecore-simu file.bin\n");
        interactive_mode();
        return Ok(());
    } else {
        let filename = &args[1];
        let mut file = File::open(filename)?;
        let mut contents = vec![];
        file.read_to_end(&mut contents)?;
        // println!("file: {:?}", contents);
        println!("test name: {}", filename);
        if args_len == 2 {
            let mut core = Core::new(false);
            core.run_simu(contents);
        } else if args_len == 3 && &args[2] == "-d" {
            let mut core = Core::new(true);
            core.run_simu(contents);
        } else {
            panic!();
        }

        Ok(())
    }
}
