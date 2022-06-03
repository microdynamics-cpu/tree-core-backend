use std::env;
use std::fs;
use treecore_parser::vcd::{vcd_def, vcd_scope_multi, vcd_var};

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    println!("In file {}", filename);
    let contents = fs::read_to_string(filename).expect("[error]read the file");
    // println!("With text:\n{}", contents);
    // let res = vcd_scope_multi(contents.as_str());
    let res = vcd_def(contents.as_str());
    println!("res: {:?}", res);
    // for v in res {
    // println!("{:?}", v);
    // }
}
