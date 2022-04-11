const MEMORY_CAPACITY: usize = 1024 * 16;
const CSR_CAPACITY: usize = 4096;

pub struct Core {
    x: [i32; 32],
    pc: u32,
    csr: [u32; CSR_CAPACITY],
    memory: [u8; MEMORY_CAPACITY],
}

enum Instruction {
    ADDI,
}

enum InstructionFormat {
    I,
}

fn get_instruction_name(instruction: &Instruction) -> &'static str {
    match instruction {
        Instruction::ADDI => "ADDI",
    }
}

fn get_instruction_format(instruction: &Instruction) -> InstructionFormat {
    match instruction {
        Instruction::ADDI => InstructionFormat::I,
    }
}

impl Core {
    pub fn new() -> Self {
        Core {
            x: [0; 32],
            pc: 0,
            csr: [0; CSR_CAPACITY],
            memory: [0; MEMORY_CAPACITY],
        }
    }

    pub fn run_simu(&mut self, data: Vec<u8>) {
        for i in 0..data.len() {
            self.memory[i] = data[i];
        }

        self.pc = 0;
        loop {
            let terminate = match self.load_word(self.pc) {
                0x00000073 => true,
                _ => false,
            };

            self.tick();
            if terminate {
                match self.x[10] {
                    0 => println!("Test Passed"),
                    _ => println!("Test Failed"),
                };
                break;
            }
        }
    }

    pub fn tick(&mut self) {
        let word = self.fetch();
        let instruction = self.decode(word);
        println!("PC:{:08x}, Word:{:08x}, Inst:{}", self.pc.wrapping_sub(4), word, get_instruction_name(&instruction));
        self.exec(word, instruction);
    }

    fn fetch(&mut self) -> u32 {
        let word = self.load_word(self.pc);
        self.pc = self.pc.wrapping_add(4);
        word
    }

    fn load_word(&mut self, addr: u32) -> u32 {
        ((self.memory[addr as usize + 3] as u32) << 24) |
        ((self.memory[addr as usize + 2] as u32) << 16) |
        ((self.memory[addr as usize + 1] as u32) << 8) |
        (self.memory[addr as usize] as u32)
    }

    fn decode(&mut self, word: u32) -> Instruction {
        let opcode = word & 0x7F;
        let func3 = (word >> 12) & 0x7F;
        let func7 = (word >> 25) & 0x7F;

        if opcode == 0x13 {
            return match func3 {
                0 => Instruction::ADDI,
                _ => {
                    println!("unkown func3: {:03b}", func3);
                    panic!();
                }
            }
        }
        println!("Unknown Instruction type.");
        panic!();
    }

    fn exec(&mut self, word: u32, inst: Instruction) {
        let format = get_instruction_format(&inst);
        match format {
            InstructionFormat::I => {
                let rd = (word >> 7) & 0x1F; // [11:7]
                let rs1 = (word >> 15) & 0x1F; // [19:15]
                let imm = (
                    match word & 0x80000000 { // imm[31:11] = [31]
                        0x80000000 => 0xfffff800,
                        _ => 0
                    } |
                    ((word >> 20) & 0x000007ff) // imm[10:0] = [30:20]
                    ) as i32;

                match inst {
                    Instruction::ADDI => {
                        self.x[rd as usize] = self.x[rs1 as usize].wrapping_add(imm);
                    },
                    _ => {
                        println!("{}", get_instruction_name(&inst).to_owned() + " instruction is not supported yet.");
                        panic!();
					}
                }
            }
        }
    }
}
