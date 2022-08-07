use std::io::{BufRead, BufReader};
use std::time::Instant;
use std::{env, fs, fs::File};
use treecore_parser::vcd::{vcd_main, vcd_build_tree};

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
        Ok(v) => {
            // println!("hdr: {:?}", v.1 .0.hdr);
            println!("rt scope: {:?}", v.1 .0.rt_scope);
            let res = vcd_build_tree(&v.1 .0.sc_list);
            println!("res's id: {}", res.id);
            // for vv in &v.1 .0.sc_list {
            //     if vv.sc_cnt > 0 {
            //         println!("scope id: {}, hier: {}", vv.sc_id, vv.sc_cnt);
            //     }
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
