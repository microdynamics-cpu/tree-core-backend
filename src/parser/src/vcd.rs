use nom::character::complete::multispace0;
use nom::{
    bytes::complete::{tag, take_while_m_n},
    combinator::{map, map_res},
    sequence::tuple,
    IResult,
};

#[derive(Debug)]
enum TimeScaleUnit {
    PS,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Header<'a> {
    pub dat: &'a str,
    pub spa: &'a str,
    pub end: &'a str,
}

pub fn header(s: &str) -> IResult<&str, Header> {
    map(
        tuple((tag("$date"), multispace0, tag("$end"))),
        |(dat, spa, end)| Header { dat, spa, end },
    )(s)
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_header() {
        assert_eq!(
            header("$date\r\n $end"),
            Ok((
                "",
                Header {
                    dat: "$date",
                    spa: "\r\n ",
                    end: "$end"
                }
            ))
        );
    }
}
