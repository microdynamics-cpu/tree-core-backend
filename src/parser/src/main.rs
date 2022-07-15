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
    match res {
        Ok(v) => {
            println!("hdr: {:?}", v.1.hdr);
            println!("rt scope: {:?}", v.1.rt_scope);
            for vv in v.1.sc_list {
                println!("scope: {:?}\n", vv);
            }
        }
        Err(_v) => panic!(),
    }
}
