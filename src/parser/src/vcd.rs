use nom::character::complete::multispace0;
use nom::{
    bytes::complete::{tag, take_until},
    combinator::{map, map_res},
    sequence::{delimited, tuple},
    IResult,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Header<'a> {
    pub dat: &'a str,
    pub ver: &'a str,
    pub tsc: &'a str,
}

pub fn end_kw_par(s: &str) -> IResult<&str, &str> {
    delimited(multispace0, tag("$end"), multispace0)(s)
}

pub fn data_par(s: &str) -> IResult<&str, &str> {
    delimited(multispace0, take_until("\r\n"), multispace0)(s)
}

pub fn dat_kw_par(s: &str) -> IResult<&str, &str> {
    delimited(multispace0, tag("$date"), multispace0)(s)
}

pub fn ver_kw_par(s: &str) -> IResult<&str, &str> {
    delimited(multispace0, tag("$version"), multispace0)(s)
}

pub fn tsc_kw_par(s: &str) -> IResult<&str, &str> {
    delimited(multispace0, tag("$timescale"), multispace0)(s)
}

pub fn dat_par(s: &str) -> IResult<&str, &str> {
    delimited(dat_kw_par, data_par, end_kw_par)(s)
}

pub fn ver_par(s: &str) -> IResult<&str, &str> {
    delimited(ver_kw_par, data_par, end_kw_par)(s)
}

pub fn tsc_par(s: &str) -> IResult<&str, &str> {
    delimited(tsc_kw_par, data_par, end_kw_par)(s)
}

pub fn header(s: &str) -> IResult<&str, Header> {
    map(tuple((dat_par, ver_par, tsc_par)), |(dat, ver, tsc)| {
        Header { dat, ver, tsc }
    })(s)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_header() {
        assert_eq!(
            header("$date\r\n\t Mon Feb 22 19:49:29 2021\r\n $end\r\n $version\r\n Icarus Verilog\r\n $end\r\n $timescale\r\n 1ps\r\n $end"),
            Ok((
                "",
                Header {
                    dat: "Mon Feb 22 19:49:29 2021",
                    ver: "Icarus Verilog",
                    tsc: "1ps",
                }
            ))
        );
    }
}
