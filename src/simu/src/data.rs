
pub struct Word {
    value: u32
}

impl Word {
    pub fn new(val: u32) -> Self {
        Word {
            value: val
        }
    }

    // word[lhs, rhs] 0 <= lhs <= rhs <= 31
    pub fn val(&self, rhs: usize, lhs: usize) -> u32 {
        if lhs > rhs || rhs >= 32 {
            panic!()
        }

        let len = rhs - lhs + 1; // len: [1, 32]
        (self.value >> lhs) & (((1u64 << len) - 1)  as u32)// [1, 0]
    }

    pub fn pos(&self, rhs: usize, lhs: usize, pos: u32) -> u32 {
        let dat = self.val(rhs, lhs);
        dat << pos
    }
}

#[cfg(test)]
mod tests {
    use crate::data::Word;
    #[test]
    #[should_panic]
    fn word_panic() {
        let val: u32 = 0b1111_0000_1111_0110__1111_0000_1111_0110;
        let dut = Word::new(val);
        assert_eq!(0, dut.val(23, 30));
        assert_eq!(0, dut.val(0, 42324));
        assert_eq!(0, dut.val(32, 0));
        assert_eq!(0, dut.val(24234, 0));
    }

    #[test]
    // #[should_panic]
    fn word_normal() {
        let val: u32 = 0b1111_0000_1111_0110__1111_0000_1111_0110;
        let dut = Word::new(val);
        // invalid param
        // assert_eq!(0, dut.val(23, 33));
        assert_eq!(0, dut.val(0, 0));
        assert_eq!(1, dut.val(1, 1));
        assert_eq!(15, dut.val(7, 4));
        assert_eq!(6, dut.val(3, 0));
        assert_eq!(6, dut.val(2, 0));
        assert_eq!(15, dut.val(31, 28));
        assert_eq!(4042715382, dut.val(31, 0));
    }
}