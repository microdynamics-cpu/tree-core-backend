use std::collections::HashMap;

pub struct Regfile {
    pub x: [i64; 32],
    pub alias: HashMap<String, u8>,
}

impl Regfile {
    pub fn new() -> Self {
        let mut res = Regfile {
            x: [0i64; 32],
            alias: HashMap::new(),
        };

        res.alias.insert("zero".to_string(), 0);
        res.alias.insert("ra".to_string(), 1);
        res.alias.insert("sp".to_string(), 2);
        res.alias.insert("gp".to_string(), 3);
        res.alias.insert("tp".to_string(), 4);
        res.alias.insert("t0".to_string(), 5);
        res.alias.insert("t1".to_string(), 6);
        res.alias.insert("t2".to_string(), 7);
        res.alias.insert("s0".to_string(), 8);
        res.alias.insert("fp".to_string(), 8);
        res.alias.insert("s1".to_string(), 9);
        res.alias.insert("a0".to_string(), 10);
        res.alias.insert("a1".to_string(), 11);
        res.alias.insert("a2".to_string(), 12);
        res.alias.insert("a3".to_string(), 13);
        res.alias.insert("a4".to_string(), 14);
        res.alias.insert("a5".to_string(), 15);
        res.alias.insert("a6".to_string(), 16);
        res.alias.insert("a7".to_string(), 17);
        res.alias.insert("s2".to_string(), 18);
        res.alias.insert("s3".to_string(), 19);
        res.alias.insert("s4".to_string(), 20);
        res.alias.insert("s5".to_string(), 21);
        res.alias.insert("s6".to_string(), 22);
        res.alias.insert("s7".to_string(), 23);
        res.alias.insert("s8".to_string(), 24);
        res.alias.insert("s9".to_string(), 25);
        res.alias.insert("s10".to_string(), 26);
        res.alias.insert("s11".to_string(), 27);
        res.alias.insert("t3".to_string(), 28);
        res.alias.insert("t4".to_string(), 29);
        res.alias.insert("t5".to_string(), 30);
        res.alias.insert("t6".to_string(), 31);
        res
    }

    pub fn reset(&mut self) {
        self.x = [0i64; 32];
    }

    pub fn val(&self, v: &str) -> i64 {
        match self.alias.get(&(v.to_string())) {
            Some(&idx) => self.x[idx as usize],
            _ => panic!(),
        }
    }
}
