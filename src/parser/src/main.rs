use std::io::{BufRead, BufReader};
use std::time::Instant;
use std::{env, fs, fs::File};
use treecore_parser::vcd::vcd_main;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    println!("In file {}", filename);
    let file = BufReader::new(File::open(filename).expect("Unable to open file"));
    let mut file_line_cnt = 0;
    for _ in file.lines() {
        file_line_cnt += 1;
    }

    let start = Instant::now();
    let contents = fs::read_to_string(filename).expect("[error]read the file");
    // println!("With text:\n{}", contents);
    let res = vcd_main(contents.as_str());
    let duration = start.elapsed();

    match res {
        Ok(_v) => {
            // println!("hdr: {:?}", v.1 .0.hdr);
            // println!("rt scope: {:?}", v.1 .0.rt_scope);
            // for vv in v.1 .0.sc_list {
            // println!("scope: {:?}\n", vv);
            // }

            // println!("init val: {:?}", v.1 .1);
            println!(
                "\x1b[92mTime elapsed in vcd_main() is: {:?}\x1b[0m",
                duration
            );
            println!(
                "\x1b[92mTotal lines: {}, parse speed is about: {:?}K lines/s\x1b[0m",
                file_line_cnt,
                file_line_cnt / duration.as_secs() / 1000
            );
        }
        Err(e) => panic!("{}", e),
    }
}
