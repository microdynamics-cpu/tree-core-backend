use nom::{
    bytes::complete::{tag, take_while_m_n},
    combinator::map_res,
    sequence::tuple,
    IResult,
};

#[derive(Debug)]
enum TimeScaleUnit {
    PS,
}

#[derive(Debug)]
pub struct Header {
    date: String,
    ver: String,
    ts_val: u8,
    ts_unit: TimeScaleUnit,
}

pub fn header(s: &str) -> IResult<&str, &str> {
    tag("$date")(s)
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_header() {
        assert_eq!(
            header("$date hello"),
            Ok((
                " hello", "$date"
            ))
        );
    }
}

