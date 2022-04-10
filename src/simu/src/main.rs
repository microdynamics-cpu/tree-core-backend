use std::env;
use std::fs::File;
use std::io::Read;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprint!("Usage: treecore-simu file.bin\n");
        return Ok(());
    }

    let filename = &args[1];
    let mut file = File::open(filename)?;
    let mut contents = vec![];
    file.read_to_end(&mut contents)?;
    println!("file: {:?}", contents);
    Ok(())
}
