use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{digit0, multispace0},
    combinator::map,
    sequence::{delimited, tuple},
    IResult,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimeScale<'a> {
    pub num: &'a str,
    pub unit: &'a str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Header<'a> {
    pub dat: &'a str,
    pub ver: &'a str,
    pub tsc: &'a str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scope<'a> {
    pub sc_type: &'a str,
    pub sc_id: &'a str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Var<'a> {
    pub bw: &'a str,
}

pub fn end_kw(s: &str) -> IResult<&str, &str> {
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
pub fn mid(s: &str) -> IResult<&str, &str> {
    map(
        delimited(multispace0, take_until("$end"), multispace0),
        |s| trm_str(s),
    )(s)
}

// declaration_keyword
pub fn comm_delc_kw(s: &str) -> IResult<&str, &str> {
    delimited(multispace0, tag("$comment"), multispace0)(s)
}

pub fn dat_decl_kw(s: &str) -> IResult<&str, &str> {
    delimited(multispace0, tag("$date"), multispace0)(s)
}

pub fn enddef_decl_kw(s: &str) -> IResult<&str, &str> {
    delimited(multispace0, tag("$enddefinitions"), multispace0)(s)
}

pub fn scope_decl_kw(s: &str) -> IResult<&str, &str> {
    delimited(multispace0, tag("$scope"), multispace0)(s)
}

pub fn tsc_decl_kw(s: &str) -> IResult<&str, &str> {
    delimited(multispace0, tag("$timescale"), multispace0)(s)
}

pub fn upscope_decl_kw(s: &str) -> IResult<&str, &str> {
    delimited(multispace0, tag("$upscope"), multispace0)(s)
}

pub fn var_decl_kw(s: &str) -> IResult<&str, &str> {
    delimited(multispace0, tag("$var"), multispace0)(s)
}

pub fn ver_decl_kw(s: &str) -> IResult<&str, &str> {
    delimited(multispace0, tag("$version"), multispace0)(s)
}

// simulation_keyword
pub fn dumpall_simu_kw(s: &str) -> IResult<&str, &str> {
    delimited(multispace0, tag("$dumpall"), multispace0)(s)
}

pub fn dumpoff_simu_kw(s: &str) -> IResult<&str, &str> {
    delimited(multispace0, tag("$dumpoff"), multispace0)(s)
}

pub fn dumpon_simu_kw(s: &str) -> IResult<&str, &str> {
    delimited(multispace0, tag("$dumpon"), multispace0)(s)
}

pub fn dumpvars_simu_kw(s: &str) -> IResult<&str, &str> {
    delimited(multispace0, tag("$dumpvars"), multispace0)(s)
}

// declaration_command
pub fn ver_decl_cmd(s: &str) -> IResult<&str, &str> {
    delimited(ver_decl_kw, mid, end_kw)(s)
}

// ====== Description of keyword commands ======
// $comment        $timescale $dumpall
// $date           $upscope   $dumpoff
// $enddefinitions $var       $dumpon
// $scope          $version   $dumpvars
pub fn comment_simu_cmd(s: &str) -> IResult<&str, &str> {
    delimited(comm_delc_kw, mid, end_kw)(s)
}

pub fn dat_decl_cmd(s: &str) -> IResult<&str, &str> {
    delimited(dat_decl_kw, mid, end_kw)(s)
}

pub fn enddef(s: &str) -> IResult<&str, &str> {
    delimited(enddef_decl_kw, multispace0, end_kw)(s)
}

pub fn scope_type(s: &str) -> IResult<&str, &str> {
    delimited(
        multispace0,
        alt((
            tag("begin"),
            tag("fork"),
            tag("function"),
            tag("module"),
            tag("task"),
        )),
        multispace0,
    )(s)
}

pub fn scope_id(s: &str) -> IResult<&str, &str> {
    delimited(multispace0, take_until(" "), multispace0)(s)
}

pub fn scope(s: &str) -> IResult<&str, Scope> {
    map(
        tuple((scope_decl_kw, scope_type, scope_id, end_kw)),
        |(_, sc_type, sc_id, _)| Scope { sc_type, sc_id },
    )(s)
}

pub fn tsc_num(s: &str) -> IResult<&str, &str> {
    delimited(multispace0, digit0, multispace0)(s)
}

pub fn tsc_unit(s: &str) -> IResult<&str, &str> {
    delimited(
        multispace0,
        alt((
            tag("s"),
            tag("ms"),
            tag("us"),
            tag("ns"),
            tag("ps"),
            tag("fs"),
        )),
        multispace0,
    )(s)
}

pub fn tsc(s: &str) -> IResult<&str, TimeScale> {
    map(
        tuple((tsc_decl_kw, tsc_num, tsc_unit, end_kw)),
        |(_, num, unit, _)| TimeScale { num, unit },
    )(s)
}

pub fn tsc_decl_cmd(s: &str) -> IResult<&str, &str> {
    delimited(tsc_decl_kw, mid, end_kw)(s)
}

pub fn header(s: &str) -> IResult<&str, Header> {
    map(
        tuple((dat_decl_cmd, ver_decl_cmd, tsc_decl_cmd)),
        |(dat, ver, tsc)| Header { dat, ver, tsc },
    )(s)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_comment_simu_cmd() {
        assert_eq!(
            comment_simu_cmd("$comment This is a single-line comment    $end"),
            Ok(("", "This is a single-line comment")),
        );

        // assert_eq!(
        //     comment_simu_cmd("$comment This is a\r\n single-line comment\r\n$end"),
        //     Ok(("", "This is a single-line comment")),
        // );

        assert_eq!(
            comment_simu_cmd("$comment This is a single-line comment\r\n    $end\r\n"),
            Ok(("", "This is a single-line comment")),
        );
    }

    #[test]
    fn test_enddef() {
        assert_eq!(enddef("$enddefinitions $end"), Ok(("", "")));
        assert_eq!(enddef("$enddefinitions\r\n     $end"), Ok(("", "")))
    }

    #[test]
    fn test_tsc() {
        assert_eq!(
            tsc("$timescale 10ps $end"),
            Ok((
                "",
                TimeScale {
                    num: "10",
                    unit: "ps"
                }
            )),
        );

        assert_eq!(
            tsc("$timescale\r\n 1ns\r\n$end"),
            Ok((
                "",
                TimeScale {
                    num: "1",
                    unit: "ns"
                }
            )),
        );
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

    #[test]
    fn test_scope() {
        assert_eq!(
            scope("$scope module tinyriscv_soc_tb $end"),
            Ok((
                "",
                Scope {
                    sc_type: "module",
                    sc_id: "tinyriscv_soc_tb"
                }
            ))
        );

        assert_eq!(
            scope("$scope\r\n\t module tinyriscv_soc_tb \r\n$end"),
            Ok((
                "",
                Scope {
                    sc_type: "module",
                    sc_id: "tinyriscv_soc_tb"
                }
            ))
        );
    }
}
