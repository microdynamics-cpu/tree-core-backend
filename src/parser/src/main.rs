use std::env;
use std::fs;
use treecore_parser::vcd::vcd_main;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    println!("In file {}", filename);
    let contents = fs::read_to_string(filename).expect("[error]read the file");
    // println!("With text:\n{}", contents);
    let res = vcd_main(contents.as_str());
    println!("res: {:?}", res);
    // for v in res {
    // println!("{:?}", v);
    // }
}
