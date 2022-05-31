use std::env;
use std::fs;
use treecore_parser::vcd::{header, Header, TimeScale};

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    println!("In file {}", filename);
    let contents = fs::read_to_string(filename).expect("[error]read the file");
    // println!("With text:\n{}", contents);

    assert_eq!(
        header(contents.as_str()),
        Ok((
            "",
            Header {
                dat: "Mon Feb 22 19:49:29 2021",
                ver: "Icarus Verilog",
                tsc: TimeScale { num: 1, unit: "ps" },
            }
        ))
    );
}
