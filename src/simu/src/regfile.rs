pub struct Regfile {
    pub x: [i32; 32],
}

impl Regfile {
    pub fn new() -> Self {
        Regfile { x: [0; 32] }
    }

    pub fn val(&self, v: &str) -> i32 {
        match v {
            "zero" => self.x[0],
            "ra" => self.x[1],
            "sp" => self.x[2],
            "gp" => self.x[3],
            "tp" => self.x[4],
            "t0" => self.x[5],
            "t1" => self.x[6],
            "t2" => self.x[7],
            "s0" | "fp" => self.x[8],
            "s1" => self.x[9],
            "a0" => self.x[10],
            "a1" => self.x[11],
            "a2" => self.x[12],
            "a3" => self.x[13],
            "a4" => self.x[14],
            "a5" => self.x[15],
            "a6" => self.x[16],
            "a7" => self.x[17],
            "s2" => self.x[18],
            "s3" => self.x[19],
            "s4" => self.x[20],
            "s5" => self.x[21],
            "s6" => self.x[22],
            "s7" => self.x[23],
            "s8" => self.x[24],
            "s9" => self.x[25],
            "s10" => self.x[26],
            "s11" => self.x[27],
            "t3" => self.x[28],
            "t4" => self.x[29],
            "t5" => self.x[30],
            "t6" => self.x[31],
            _ => panic!(),
        }
    }
}
