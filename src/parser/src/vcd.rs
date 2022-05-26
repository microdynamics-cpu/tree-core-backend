use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{digit0, digit1, multispace0},
    combinator::{map, map_res},
    sequence::{delimited, pair, separated_pair, tuple},
    IResult,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimeScale<'a> {
    pub num: u8,
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
    pub var_type: &'a str,
    pub bw: u8,
    pub id: &'a str,
    pub refer: &'a str,
}

// ref to the verilog-std-1364-2005 LRM
// main entry
// pub fn value_change_dump_def(s: &str) -> IResult<&str, &str> {

// }

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

pub fn usc_decl_kw(s: &str) -> IResult<&str, &str> {
    delimited(multispace0, tag("$upscope"), multispace0)(s)
}

pub fn var_decl_kw(s: &str) -> IResult<&str, &str> {
    delimited(multispace0, tag("$var"), multispace0)(s)
}

pub fn ver_decl_kw(s: &str) -> IResult<&str, &str> {
    delimited(multispace0, tag("$version"), multispace0)(s)
}

// ====== Description of keyword commands ======
// $comment        $timescale $dumpall
// $date           $upscope   $dumpoff
// $enddefinitions $var       $dumpon
// $scope_decl_cmd $version   $dumpvars
pub fn comm_decl_cmd(s: &str) -> IResult<&str, &str> {
    delimited(comm_delc_kw, cmd_text, end_kw)(s)
}

pub fn dat_decl_cmd(s: &str) -> IResult<&str, &str> {
    delimited(dat_decl_kw, cmd_text, end_kw)(s)
}

pub fn enddef_decl_cmd(s: &str) -> IResult<&str, &str> {
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

pub fn scope_decl_cmd(s: &str) -> IResult<&str, Scope> {
    map(
        tuple((scope_decl_kw, scope_type, scope_id, end_kw)),
        |(_, sc_type, sc_id, _)| Scope { sc_type, sc_id },
    )(s)
}

pub fn tsc_num(s: &str) -> IResult<&str, u8> {
    map_res(digit1, |s: &str| s.parse::<u8>())(s)
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

pub fn tsc_decl_cmd(s: &str) -> IResult<&str, TimeScale> {
    map(
        tuple((tsc_decl_kw, tsc_num, tsc_unit, end_kw)),
        |(_, num, unit, _)| TimeScale { num, unit },
    )(s)
}

pub fn usc_decl_cmd(s: &str) -> IResult<&str, &str> {
    delimited(usc_decl_kw, multispace0, end_kw)(s)
}

pub fn variable_type(s: &str) -> IResult<&str, &str> {
    delimited(
        multispace0,
        alt((
            tag("event"),
            tag("integer"),
            tag("parameter"),
            tag("real"),
            tag("realtime"),
            tag("reg"),
            tag("supply0"),
            tag("supply1"),
            tag("time"),
            tag("tri"),
            tag("triand"),
            tag("trior"),
            tag("trireg"),
            tag("tri0"),
            tag("tri1"),
            tag("wand"),
            tag("wire"),
            tag("wor"),
        )),
        multispace0,
    )(s)
}

pub fn variable_bw(s: &str) -> IResult<&str, u8> {
    map_res(digit1, |s: &str| s.parse::<u8>())(s)
}

pub fn variable_id(s: &str) -> IResult<&str, &str> {
    delimited(multispace0, take_until(" "), multispace0)(s)
}

pub fn variable_vector(s: &str) -> IResult<&str, (u8, u8)> {
    delimited(
        tag("["),
        separated_pair(variable_idx, tag(":"), variable_idx),
        tag("]"),
    )(s)
}

pub fn variable_idx(s: &str) -> IResult<&str, u8> {
    map_res(digit0, |s: &str| s.parse::<u8>())(s)
}
pub fn variable_ref(s: &str) -> IResult<&str, (&str, (u8, u8))> {
    delimited(multispace0, pair(variable_id, variable_vector), multispace0)(s)
}

pub fn var_decl_cmd(s: &str) -> IResult<&str, Var> {
    map(
        tuple((
            var_decl_kw,
            variable_type,
            variable_bw,
            variable_id,
            variable_id,
            variable_vector,
            end_kw,
        )),
        |(_, var_type, bw, id, refer, _, _)| Var {
            var_type,
            bw,
            id,
            refer,
        },
    )(s)
}

pub fn ver_decl_cmd(s: &str) -> IResult<&str, &str> {
    delimited(ver_decl_kw, cmd_text, end_kw)(s)
}

pub fn trm_str(s: &str) -> &str {
    // let iter: Vec<_> = s.trim().split_whitespace().collect();
    // println!("[trm str]: {}", iter.join(" ").as_str());
    // let mut res = "abc adfasd";
    // res
    s.trim()
}

pub fn cmd_text(s: &str) -> IResult<&str, &str> {
    map(
        delimited(multispace0, take_until("$end"), multispace0),
        |s| trm_str(s),
    )(s)
}

pub fn end_kw(s: &str) -> IResult<&str, &str> {
    delimited(multispace0, tag("$end"), multispace0)(s)
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

// pub fn tsc_decl_cmd(s: &str) -> IResult<&str, &str> {
//     delimited(tsc_decl_kw, cmd_text, end_kw)(s)
// }

// pub fn header(s: &str) -> IResult<&str, Header> {
//     map(
//         tuple((dat_decl_cmd, ver_decl_cmd, tsc_decl_cmd)),
//         |(dat, ver, tsc)| Header { dat, ver, tsc },
//     )(s)
// }

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_comm_delc_kw() {
        assert_eq!(comm_delc_kw("$comment"), Ok(("", "$comment")),);
        assert_eq!(
            comm_delc_kw("\r\n  \t $comment\r\n \t  "),
            Ok(("", "$comment")),
        );
    }

    #[test]
    fn test_dat_delc_kw() {
        assert_eq!(dat_decl_kw("$date"), Ok(("", "$date")),);
        assert_eq!(dat_decl_kw("\r\n  \t $date\r\n \t  "), Ok(("", "$date")),);
    }

    #[test]
    fn test_enddef_delc_kw() {
        assert_eq!(
            enddef_decl_kw("$enddefinitions"),
            Ok(("", "$enddefinitions")),
        );

        assert_eq!(
            enddef_decl_kw("\r\n  \t $enddefinitions\r\n \t  "),
            Ok(("", "$enddefinitions")),
        );
    }

    #[test]
    fn test_scope_delc_kw() {
        assert_eq!(scope_decl_kw("$scope"), Ok(("", "$scope")),);
        assert_eq!(
            scope_decl_kw("\r\n  \t $scope\r\n \t  "),
            Ok(("", "$scope")),
        );
    }

    #[test]
    fn test_tsc_delc_kw() {
        assert_eq!(tsc_decl_kw("$timescale"), Ok(("", "$timescale")),);
        assert_eq!(
            tsc_decl_kw("\r\n  \t $timescale\r\n \t  "),
            Ok(("", "$timescale")),
        );
    }

    #[test]
    fn test_usc_delc_kw() {
        assert_eq!(usc_decl_kw("$upscope"), Ok(("", "$upscope")),);
        assert_eq!(
            usc_decl_kw("\r\n  \t $upscope\r\n \t  "),
            Ok(("", "$upscope")),
        );
    }

    #[test]
    fn test_var_delc_kw() {
        assert_eq!(var_decl_kw("$var"), Ok(("", "$var")),);
        assert_eq!(var_decl_kw("\r\n  \t $var\r\n \t  "), Ok(("", "$var")),);
    }

    #[test]
    fn test_ver_delc_kw() {
        assert_eq!(ver_decl_kw("$version"), Ok(("", "$version")),);
        assert_eq!(
            ver_decl_kw("\r\n  \t $version\r\n \t  "),
            Ok(("", "$version")),
        );
    }

    #[test]
    fn test_comm_decl_cmd() {
        assert_eq!(
            comm_decl_cmd("$comment This is a single-line comment    $end"),
            Ok(("", "This is a single-line comment")),
        );

        // assert_eq!(
        //     comm_decl_cmd("$comment This is a\r\n single-line comment\r\n$end"),
        //     Ok(("", "This is a single-line comment")),
        // );

        assert_eq!(
            comm_decl_cmd("$comment This is a single-line comment\r\n    $end\r\n"),
            Ok(("", "This is a single-line comment")),
        );
    }

    #[test]
    fn test_dat_decl_cmd() {
        assert_eq!(
            dat_decl_cmd("$date Mon Feb 22 19:49:29 2021    $end"),
            Ok(("", "Mon Feb 22 19:49:29 2021")),
        );
        assert_eq!(
            dat_decl_cmd("$date Mon Feb 22 19:49:29 2021\r\n    $end\r\n"),
            Ok(("", "Mon Feb 22 19:49:29 2021")),
        );
    }

    #[test]
    fn test_enddef_decl_cmd() {
        assert_eq!(enddef_decl_cmd("$enddefinitions $end"), Ok(("", "")));
        assert_eq!(
            enddef_decl_cmd("$enddefinitions\r\n     $end"),
            Ok(("", ""))
        )
    }
    #[test]
    fn test_scope_type() {
        assert_eq!(scope_type("begin"), Ok(("", "begin")));
        assert_eq!(scope_type("fork\r\n     "), Ok(("", "fork")));
        assert_eq!(scope_type("function\r\n     "), Ok(("", "function")));
        assert_eq!(scope_type("module\r\n   "), Ok(("", "module")));
        assert_eq!(scope_type("task \r\n   \t"), Ok(("", "task")));
    }

    #[test]
    fn test_scope_id() {
        assert_eq!(scope_id("! "), Ok(("", "!")));
        assert_eq!(scope_id("% \r\n     "), Ok(("", "%")));
        assert_eq!(scope_id("4 \r\n     "), Ok(("", "4")));
        assert_eq!(scope_id("@ \r\n   "), Ok(("", "@")));
        assert_eq!(scope_id("] \r\n   \t"), Ok(("", "]")));
    }

    #[test]
    fn test_scope_decl_cmd() {
        assert_eq!(
            scope_decl_cmd("$scope module tinyriscv_soc_tb $end"),
            Ok((
                "",
                Scope {
                    sc_type: "module",
                    sc_id: "tinyriscv_soc_tb"
                }
            ))
        );

        assert_eq!(
            scope_decl_cmd("$scope\r\n\t module tinyriscv_soc_tb \r\n$end"),
            Ok((
                "",
                Scope {
                    sc_type: "module",
                    sc_id: "tinyriscv_soc_tb"
                }
            ))
        );
    }

    #[test]
    fn test_tsc_num() {
        assert_eq!(tsc_num("1 "), Ok((" ", 1)));
        assert_eq!(tsc_num("10 \r\n     "), Ok((" \r\n     ", 10)));
        assert_eq!(tsc_num("100 \r\n     "), Ok((" \r\n     ", 100)));
    }

    #[test]
    fn test_tsc_unit() {
        assert_eq!(tsc_unit("s "), Ok(("", "s")));
        assert_eq!(tsc_unit("ms \r\n     "), Ok(("", "ms")));
        assert_eq!(tsc_unit("us \r\n     "), Ok(("", "us")));
        assert_eq!(tsc_unit(" ps \r\n     "), Ok(("", "ps")));
        assert_eq!(tsc_unit("\r\n fs \r\n     "), Ok(("", "fs")));
    }

    #[test]
    fn test_tsc_decl_cmd() {
        assert_eq!(
            tsc_decl_cmd("$timescale 10ps $end"),
            Ok((
                "",
                TimeScale {
                    num: 10,
                    unit: "ps"
                }
            )),
        );

        assert_eq!(
            tsc_decl_cmd("$timescale\r\n 1ns\r\n$end"),
            Ok(("", TimeScale { num: 1, unit: "ns" })),
        );
    }

    #[test]
    fn test_usc_decl_cmd() {
        assert_eq!(usc_decl_cmd("$upscope $end"), Ok(("", "")),);
        assert_eq!(
            usc_decl_cmd("\r\n  \t $upscope   $end\r\n \t  "),
            Ok(("", "")),
        );
    }

    // #[test]
    // fn test_header() {
    //     assert_eq!(
    //         header("$date\r\n\t Mon Feb 22 19:49:29 2021\r\n $end\r\n $version\r\n Icarus Verilog\r\n $end\r\n $timescale\r\n 1ps\r\n $end"),
    //         Ok((
    //             "",
    //             Header {
    //                 dat: "Mon Feb 22 19:49:29 2021",
    //                 ver: "Icarus Verilog",
    //                 tsc: "1ps",
    //             }
    //         ))
    //     );
    // }
}
