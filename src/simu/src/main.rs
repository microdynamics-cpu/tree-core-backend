use std::env;
use std::fs::File;
use std::io::Read;
use treecore_simu::core::Core;

fn interactive_mode() {
    println!("TreeCore RISCV ISA Simulator 0.0.1");
    println!("[last-release] on Ubuntu 20.04 LTS");
    println!("Type 'help' for more information.");
    println!(">>>");
    // loop {
    //     println!(">>>");
    // }
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        // eprint!("usage: treecore-simu file.bin\n");
        interactive_mode();
        return Ok(());
    }

    let filename = &args[1];
    let mut file = File::open(filename)?;
    let mut contents = vec![];
    file.read_to_end(&mut contents)?;
    // println!("file: {:?}", contents);
    let mut core = Core::new();
    core.run_simu(contents);
    Ok(())
}
