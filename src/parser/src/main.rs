use std::env;
use std::fs;
use treecore_parser::vcd::{vcd_header, vcd_scope_multi, vcd_var, Header, TimeScale};

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    println!("In file {}", filename);
    let contents = fs::read_to_string(filename).expect("[error]read the file");
    // println!("With text:\n{}", contents);

    // assert_eq!(
    //     vcd_header(contents.as_str()),
    //     Ok((
    //         "",
    //         Header {
    //             dat: "Mon Feb 22 19:49:29 2021",
    //             ver: "Icarus Verilog",
    //             tsc: TimeScale { num: 1, unit: "ps" },
    //         }
    //     ))
    // );
    // let res = vcd_var(contents.as_str());
    let res = vcd_scope_multi(contents.as_str());
    for v in res {
        println!("{:?}", v);
    }
}
