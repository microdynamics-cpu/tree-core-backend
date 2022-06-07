use crate::core::Core;
use std::fs::File;
use std::io::{stdin, stdout, Read, Write};

// like nemu
// 0x00000297,  // auipc t0,0
// 0x0002b823,  // sd  zero,16(t0)
// 0x0102b503,  // ld  a0,16(t0)
// 0x0000006b,  // treecore_trap
// 0xdeadbeef,  // some data

enum CliCmd {
    NONE,
    HELP,
    QUIT,
    RUN,
    LOAD,
    TDB,
    TDB_C,
    TDB_SI,
    TDB_INFO,
}

pub struct Cmd<'a> {
    name: &'a str,
    info: &'a str,
}

pub struct Cli<'a> {
    prompt: &'a str,
    cmd_list: [Cmd<'a>; 6],
}

impl Cli<'_> {
    pub fn new() -> Self {
        Cli {
            prompt: ">>>",
            cmd_list: [
                Cmd {
                    name: "help",
                    info: "help info",
                },
                Cmd {
                    name: "quit",
                    info: "quit from the interactive mode",
                },
                Cmd {
                    name: "run",
                    info: " run loaded binary program",
                },
                Cmd {
                    name: "load",
                    info: "load binary program",
                },
                Cmd {
                    name: "tdb",
                    info: " start a debugger",
                },
                Cmd {
                    name: "info",
                    info: "[r|w]: print [register|watchpoint] info",
                },
            ],
        }
    }

    fn flush(&self) {
        match stdout().flush() {
            Ok(()) => {}
            Err(_e) => panic!(),
        }
    }

    fn map_cmd(&self, val: &str) -> CliCmd {
        match val {
            "help" => CliCmd::HELP,
            "quit" => CliCmd::QUIT,
            "run" => CliCmd::RUN,
            "load" => CliCmd::LOAD,
            "tdb" => CliCmd::TDB,
            "info" => CliCmd::TDB_INFO,
            _ => panic!(),
        }
    }

    fn cmd_parser<'a>(&mut self, val: &'a String) -> (CliCmd, Option<&'a str>) {
        let mut val_list = val.split_whitespace();
        let first_args = val_list.next();
        let sec_args = val_list.next();

        match first_args {
            Some(va) => {
                for vb in &self.cmd_list {
                    if va == vb.name {
                        return (self.map_cmd(vb.name), sec_args);
                    }
                }
            }
            None => {}
        }

        (CliCmd::NONE, None)
        // self.cmd_deduce(val);
    }

    // fn cmd_comp(&self) {

    // }
    // fn cmd_deduce(&self, val: &String) {
    // let mut sim = 0;
    // for v in self.cmd_list.iter() {
    // for va in val.char
    // }
    // }

    fn print_help(&self) {
        for v in self.cmd_list.iter() {
            println!("{}: {}", v.name, v.info);
        }
    }

    pub fn inter_mode(&mut self, core: &mut Core) {
        println!("\x1b[92mTreeCore RISCV ISA Simulator 0.0.1\x1b[0m");
        println!("\x1b[92m[last-release] on Ubuntu 20.04 LTS\x1b[0m");
        println!("\x1b[92mType 'help' for more information.\x1b[0m");

        let dummy_bin: Vec<u8> = vec![
            0x97, 0x02, 0x00, 0x00, 0x23, 0xb8, 0x02, 0x00, 0x03, 0xb5, 0x02, 0x01, 0x6b, 0x00,
            0x00, 0x00, 0xef, 0xbe, 0xad, 0xde,
        ];
        core.load_bin_file(dummy_bin);

        let mut input_dat = String::new();
        loop {
            print!("{}", self.prompt);
            self.flush();
            match stdin().read_line(&mut input_dat) {
                Ok(_v) => {
                    // print!("[debug]{}", input_dat);
                    let (fir_cmd, sec_cmd) = self.cmd_parser(&input_dat);
                    match fir_cmd {
                        CliCmd::NONE => {
                            println!(
                                "\x1b[93m[Warn] no support cmd, Type 'help' to get all legal cmds\x1b[0m"
                            );
                        }
                        CliCmd::HELP => {
                            self.print_help();
                        }
                        CliCmd::QUIT => break,
                        CliCmd::RUN => {
                            core.reset();
                            core.run_simu(None, None); // NOTE: now just for cmd binary
                        }
                        CliCmd::LOAD => match sec_cmd {
                            Some(v) => {
                                println!("\x1b[93m[binary loading]...\x1b[0m");
                                match File::open(v) {
                                    Ok(mut file) => {
                                        let mut contents = vec![];
                                        match file.read_to_end(&mut contents) {
                                            Ok(_v) => {
                                                println!("\x1b[92m[Loading Success]...\x1b[0m");
                                                core.load_bin_file(contents);
                                            }
                                            Err(_e) => panic!(),
                                        }
                                    }
                                    Err(_e) => panic!(),
                                }
                            }
                            None => println!(
                                "\x1b[93m[Warn] none binary path, please type right one\x1b[0m"
                            ),
                        },
                        CliCmd::TDB => {
                            println!("run tdb..."); // NOTE: no impl
                        }
                        CliCmd::TDB_INFO => {
                            println!("run info..."); // NOTE: no impl
                        }
                    }
                    input_dat.clear();
                }
                Err(e) => {
                    println!("[err]: {}", e);
                    panic!()
                }
            }
        }
    }
}
