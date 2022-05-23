use nom::character::complete::multispace0;
use nom::{
    bytes::complete::{tag, take_until},
    combinator::map,
    sequence::{delimited, tuple},
    IResult,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Header<'a> {
    pub dat: &'a str,
    pub ver: &'a str,
    pub tsc: &'a str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scope<'a> {
    pub par: &'a str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Var<'a> {
    pub bw: &'a str,
}

pub fn end_kw_par(s: &str) -> IResult<&str, &str> {
    delimited(multispace0, tag("$end"), multispace0)(s)
}

pub fn trm_str(s: &str) -> &str {
    // let iter: Vec<_> = s.trim().split_whitespace().collect();
    // println!("[trm str]: {}", iter.join(" ").as_str());
    // let mut res = "abc adfasd";
    // res
    s.trim()
}

//HACK: perf improve?
pub fn data_par(s: &str) -> IResult<&str, &str> {
    map(
        delimited(multispace0, take_until("$end"), multispace0),
        |s| trm_str(s),
    )(s)
}

// declaration_keyword
pub fn comm_delc_kw_par(s: &str) -> IResult<&str, &str> {
    delimited(multispace0, tag("$comment"), multispace0)(s)
}

pub fn dat_decl_kw_par(s: &str) -> IResult<&str, &str> {
    delimited(multispace0, tag("$date"), multispace0)(s)
}

pub fn enddef_decl_kw_par(s: &str) -> IResult<&str, &str> {
    delimited(multispace0, tag("$enddefinitions"), multispace0)(s)
}

pub fn scope_decl_kw_par(s: &str) -> IResult<&str, &str> {
    delimited(multispace0, tag("$scope"), multispace0)(s)
}

pub fn tsc_decl_kw_par(s: &str) -> IResult<&str, &str> {
    delimited(multispace0, tag("$timescale"), multispace0)(s)
}

pub fn upscope_decl_kw_par(s: &str) -> IResult<&str, &str> {
    delimited(multispace0, tag("$upscope"), multispace0)(s)
}

pub fn var_decl_kw_par(s: &str) -> IResult<&str, &str> {
    delimited(multispace0, tag("$var"), multispace0)(s)
}

pub fn ver_decl_kw_par(s: &str) -> IResult<&str, &str> {
    delimited(multispace0, tag("$version"), multispace0)(s)
}

// simulation_keyword
pub fn dumpall_simu_kw_par(s: &str) -> IResult<&str, &str> {
    delimited(multispace0, tag("$dumpall"), multispace0)(s)
}

pub fn dumpoff_simu_kw_par(s: &str) -> IResult<&str, &str> {
    delimited(multispace0, tag("$dumpoff"), multispace0)(s)
}

pub fn dumpon_simu_kw_par(s: &str) -> IResult<&str, &str> {
    delimited(multispace0, tag("$dumpon"), multispace0)(s)
}

pub fn dumpvars_simu_kw_par(s: &str) -> IResult<&str, &str> {
    delimited(multispace0, tag("$dumpvars"), multispace0)(s)
}

// declaration_command
pub fn dat_decl_cmd_par(s: &str) -> IResult<&str, &str> {
    delimited(dat_decl_kw_par, data_par, end_kw_par)(s)
}

pub fn ver_decl_cmd_par(s: &str) -> IResult<&str, &str> {
    delimited(ver_decl_kw_par, data_par, end_kw_par)(s)
}

pub fn tsc_decl_cmd_par(s: &str) -> IResult<&str, &str> {
    delimited(tsc_decl_kw_par, data_par, end_kw_par)(s)
}

// simulation_command
pub fn comment_simu_cmd_par(s: &str) -> IResult<&str, &str> {
    delimited(comm_delc_kw_par, data_par, end_kw_par)(s)
}

pub fn enddef_par(s: &str) -> IResult<&str, &str> {
    delimited(enddef_decl_kw_par, multispace0, end_kw_par)(s)
}

pub fn header(s: &str) -> IResult<&str, Header> {
    map(
        tuple((dat_decl_cmd_par, ver_decl_cmd_par, tsc_decl_cmd_par)),
        |(dat, ver, tsc)| Header { dat, ver, tsc },
    )(s)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_comment_simu_cmd_par() {
        assert_eq!(
            comment_simu_cmd_par("$comment This is a single-line comment    $end"),
            Ok(("", "This is a single-line comment")),
        );

        // assert_eq!(
        //     comment_simu_cmd_par("$comment This is a\r\n single-line comment\r\n$end"),
        //     Ok(("", "This is a single-line comment")),
        // );

        assert_eq!(
            comment_simu_cmd_par("$comment This is a single-line comment\r\n    $end\r\n"),
            Ok(("", "This is a single-line comment")),
        );
    }

    #[test]
    fn test_enddef_par() {
        assert_eq!(enddef_par("$enddefinitions $end"), Ok(("", "")));

        assert_eq!(enddef_par("$enddefinitions\r\n     $end"), Ok(("", "")))
    }

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
