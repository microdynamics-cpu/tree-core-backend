use nom::{
    branch::alt,
    bytes::complete::{is_a, tag, take_until},
    character::complete::{digit0, digit1, multispace0, one_of},
    combinator::{map, map_res},
    multi::many0,
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
    pub tsc: TimeScale<'a>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scope<'a> {
    pub sc_type: &'a str,
    pub sc_id: &'a str,
    pub var_list: Vec<Var<'a>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Var<'a> {
    pub var_type: &'a str,
    pub bw: u8,
    pub id: &'a str,
    pub refer: &'a str,
    pub idx: (u8, u8),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Val<'a> {
    pub val: u64,
    pub vld: char,
    pub id: &'a str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VcdMeta<'a> {
    pub hdr: Header<'a>,
    pub sc_list: Vec<Scope<'a>>,
    pub rt_scope: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VcdTimeVal<'a> {
    pub st_flag: u32,
    pub tv_list: Vec<Val<'a>>,
}

// ref to the verilog-std-1364-2005 LRM
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
        |(_, sc_type, sc_id, _)| Scope {
            sc_type,
            sc_id,
            var_list: Vec::new(),
        },
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

pub fn usc_decl_cmd(s: &str) -> IResult<&str, Scope> {
    map(tuple((usc_decl_kw, multispace0, end_kw)), |(_, _, _)| {
        println!("[upscope]");
        Scope {
            sc_type: "no",
            sc_id: "no",
            var_list: Vec::new(),
        }
    })(s)
}

pub fn variable_type(s: &str) -> IResult<&str, &str> {
    delimited(
        multispace0,
        alt((
            tag("event"),
            tag("integer"),
            tag("parameter"),
            tag("realtime"),
            tag("real"),
            tag("reg"),
            tag("supply0"),
            tag("supply1"),
            tag("time"),
            tag("triand"),
            tag("trior"),
            tag("trireg"),
            tag("tri0"),
            tag("tri1"),
            tag("tri"),
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

//BUG: is right?
pub fn variable_id(s: &str) -> IResult<&str, &str> {
    delimited(
        multispace0,
        alt((take_until(" "), take_until("\r\n"))),
        multispace0,
    )(s)
}

pub fn variable_idx(s: &str) -> IResult<&str, u8> {
    map_res(digit0, |s: &str| s.parse::<u8>())(s)
}

pub fn variable_vector(s: &str) -> IResult<&str, (u8, u8)> {
    delimited(
        tag("["),
        separated_pair(variable_idx, tag(":"), variable_idx),
        tag("]"),
    )(s)
}

pub fn variable_vec_ref(s: &str) -> IResult<&str, (&str, (u8, u8))> {
    delimited(multispace0, pair(variable_id, variable_vector), multispace0)(s)
}

pub fn variable_sca_ref(s: &str) -> IResult<&str, (&str, (u8, u8))> {
    map(delimited(multispace0, variable_id, multispace0), |s| {
        (s, (0, 0))
    })(s)
}
pub fn var_decl_cmd(s: &str) -> IResult<&str, Var> {
    map(
        tuple((
            var_decl_kw,
            variable_type,
            variable_bw,
            variable_id,
            alt((variable_vec_ref, variable_sca_ref)),
            end_kw,
        )),
        |(_, var_type, bw, id, refer, _)| {
            let res = Var {
                var_type,
                bw,
                id,
                refer: refer.0,
                idx: refer.1,
            };
            // if res.bw > 1 {
            // println!("bw > 1");
            // }
            res
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

pub fn simu_kw(s: &str) -> IResult<&str, &str> {
    alt((
        dumpall_simu_kw,
        dumpoff_simu_kw,
        dumpon_simu_kw,
        dumpvars_simu_kw,
    ))(s)
}

pub fn simu_time_val(s: &str) -> IResult<&str, u32> {
    map_res(digit1, |s: &str| s.parse::<u32>())(s)
}

pub fn simu_time(s: &str) -> IResult<&str, u32> {
    map(tuple((tag("#"), simu_time_val)), |(_, v)| v)(s)
}

// NOTE: pay attention to the order of two sub parser
pub fn val_chg(s: &str) -> IResult<&str, Val> {
    alt((vec_val_chg, sec_val_chg))(s)
}

// NOTE: use the greedy method to match the longest pattern!
pub fn val_id(s: &str) -> IResult<&str, &str> {
    delimited(
        multispace0,
        alt((take_until("\r\n"), take_until("\n"))),
        multispace0,
    )(s)
}

pub fn sec_val_chg(s: &str) -> IResult<&str, Val> {
    map(
        tuple((multispace0, sec_val, val_id, multispace0)),
        |(_, val, id, _)| {
            let mut res = Val {
                val: 0u64,
                vld: '0',
                id: id,
            };
            if val == 'x' || val == 'X' || val == 'z' || val == 'Z' {
                res.vld = val;
            } else {
                res.val = val.to_digit(2).unwrap() as u64; //HACK: maybe right?
            }
            res
        },
    )(s)
}

fn check_undef_val(val: &str) -> bool {
    for v in val.chars() {
        if v == 'x' || v == 'X' || v == 'z' || v == 'Z' {
            return true;
        }
    }
    false
}

fn bin_str_to_oct(val: &str) -> u64 {
    let mut res = 0u64;
    let mut mul = 1u64;
    for v in val.chars().rev() {
        if v == '1' {
            res += mul;
        }
        mul = mul.wrapping_mul(2);
    }
    res
}

pub fn vec_val_chg(s: &str) -> IResult<&str, Val> {
    map(
        tuple((
            multispace0,
            vec_flag_val,
            alt((is_a("0xXzZ"), digit1)),
            val_id,
            multispace0,
        )),
        |(_, _ch, val, id, _)| {
            let mut res = Val {
                val: 0u64,
                vld: '0',
                id: id,
            };

            if check_undef_val(val) {
                let mut ch = val.chars();
                if val.chars().count() == 1 {
                    res.vld = ch.next().unwrap();
                } else {
                    res.vld = ch.next().unwrap();
                    res.vld = ch.next().unwrap();
                }
            } else {
                res.val = bin_str_to_oct(val);
            }
            res
        },
    )(s)
}

pub fn sec_val(s: &str) -> IResult<&str, char> {
    one_of("01xXzZ")(s)
}

pub fn vec_flag_val(s: &str) -> IResult<&str, char> {
    one_of("bBrR")(s)
}

// high level parser
pub fn vcd_header(s: &str) -> IResult<&str, Header> {
    map(
        tuple((dat_decl_cmd, ver_decl_cmd, tsc_decl_cmd)),
        |(dat, ver, tsc)| Header { dat, ver, tsc },
    )(s)
}

pub fn vcd_scope(s: &str) -> IResult<&str, Scope> {
    map(tuple((scope_decl_cmd, vcd_var)), |(mut scope, var_list)| {
        println!("[scope] id: {}", scope.sc_id);
        scope.var_list = var_list;
        scope
    })(s)
}

// scope upscope
pub fn vcd_scope_multi(s: &str) -> IResult<&str, Vec<Scope>> {
    many0(vcd_scope)(s)
}

pub fn vcd_end_multi(s: &str) -> IResult<&str, Vec<Scope>> {
    many0(usc_decl_cmd)(s)
}

pub fn vcd_def(s: &str) -> IResult<&str, Vec<Scope>> {
    many0(alt((vcd_scope, usc_decl_cmd)))(s)
}

pub fn vcd_var(s: &str) -> IResult<&str, Vec<Var>> {
    many0(var_decl_cmd)(s)
}

pub fn vcd_meta(s: &str) -> IResult<&str, VcdMeta> {
    map(
        tuple((vcd_header, vcd_def, enddef_decl_cmd)),
        |(hdr, sc_list, _)| VcdMeta {
            hdr,
            sc_list,
            rt_scope: 0u32,
        },
    )(s)
}

pub fn vcd_init(s: &str) -> IResult<&str, VcdTimeVal> {
    map(
        tuple((simu_time, dumpvars_simu_kw, many0(val_chg), end_kw)),
        |(st_flag, _, tv_list, _)| VcdTimeVal { st_flag, tv_list },
    )(s)
}

pub fn vcd_timeval(s: &str) -> IResult<&str, VcdTimeVal> {
    map(tuple((simu_time, many0(val_chg))), |(st_flag, tv_list)| {
        VcdTimeVal { st_flag, tv_list }
    })(s)
}

pub fn vcd_body(s: &str) -> IResult<&str, Vec<VcdTimeVal>> {
    many0(vcd_timeval)(s)
}

// main entry
pub fn vcd_main(s: &str) -> IResult<&str, (VcdMeta, VcdTimeVal, Vec<VcdTimeVal>)> {
    map(
        tuple((vcd_meta, vcd_init, vcd_body)),
        |(meta, init, body)| (meta, init, body),
    )(s)
}

#[cfg(test)]
mod unit_test {
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
        assert_eq!(scope_id("tinyriscv_soc_tb "), Ok(("", "tinyriscv_soc_tb")));
        assert_eq!(scope_id("hello \r\n     "), Ok(("", "hello")));
        assert_eq!(scope_id(" top \r\n     "), Ok(("", "top")));
    }

    #[test]
    fn test_scope_decl_cmd() {
        assert_eq!(
            scope_decl_cmd("$scope module tinyriscv_soc_tb $end"),
            Ok((
                "",
                Scope {
                    sc_type: "module",
                    sc_id: "tinyriscv_soc_tb",
                    var_list: Vec::new(),
                }
            ))
        );

        assert_eq!(
            scope_decl_cmd("$scope\r\n\t module tinyriscv_soc_tb \r\n$end"),
            Ok((
                "",
                Scope {
                    sc_type: "module",
                    sc_id: "tinyriscv_soc_tb",
                    var_list: Vec::new(),
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
        assert_eq!(
            usc_decl_cmd("$upscope $end"),
            Ok((
                "",
                Scope {
                    sc_type: "no",
                    sc_id: "no",
                    var_list: Vec::new(),
                }
            )),
        );
        assert_eq!(
            usc_decl_cmd("\r\n  \t $upscope   $end\r\n \t  "),
            Ok((
                "",
                Scope {
                    sc_type: "no",
                    sc_id: "no",
                    var_list: Vec::new(),
                }
            )),
        );
    }

    #[test]
    #[rustfmt::skip]
    fn test_variable_type() {
        assert_eq!(variable_type("event "), Ok(("", "event")));
        assert_eq!(variable_type("integer \r\n     "), Ok(("", "integer")));
        assert_eq!(variable_type("parameter \r\n     "), Ok(("", "parameter")));
        assert_eq!(variable_type(" real \r\n     "), Ok(("", "real")));
        assert_eq!(variable_type("\r\n realtime \r\n     "), Ok(("", "realtime")));
        assert_eq!(variable_type("\r\n  \r reg \r\n     "), Ok(("", "reg")));
        assert_eq!(variable_type("\r\n  \nsupply0 \r\n     "), Ok(("", "supply0")));
        assert_eq!(variable_type("\r\n  \nsupply1 \r\n     "), Ok(("", "supply1")));
        assert_eq!(variable_type("\r\n  time \r\n     "), Ok(("", "time")));
        assert_eq!(variable_type("\r\n  tri\t \r\n     "), Ok(("", "tri")));
        assert_eq!(variable_type("\r\n  triand\n\t \r\n     "), Ok(("", "triand")));
        assert_eq!(variable_type("\r\n  trior \n\t \r\n     "), Ok(("", "trior")));
        assert_eq!(variable_type("\r\n trireg \n\t \r\n     "), Ok(("", "trireg")));
        assert_eq!(variable_type("\r\n tri0 \n\t \r\n     "), Ok(("", "tri0")));
        assert_eq!(variable_type("\r\n tri1 \n\t \r\n     "), Ok(("", "tri1")));
        assert_eq!(variable_type("\r\n wand \n\t  \r\n     "), Ok(("", "wand")));
        assert_eq!(variable_type("\r\n wire  \n\t  \r\n     "), Ok(("", "wire")));
        assert_eq!(variable_type("\r\n wor    \n\t  \r\n     "), Ok(("", "wor")));
    }

    #[test]
    fn test_variable_bw() {
        assert_eq!(variable_bw("1 "), Ok((" ", 1)));
        assert_eq!(variable_bw("10 \r\n     "), Ok((" \r\n     ", 10)));
        assert_eq!(variable_bw("100 \r\n     "), Ok((" \r\n     ", 100)));
    }

    #[test]
    fn test_variable_id() {
        assert_eq!(variable_id("! "), Ok(("", "!")));
        assert_eq!(variable_id("% \r\n     "), Ok(("", "%")));
        assert_eq!(variable_id("4 \r\n     "), Ok(("", "4")));
        assert_eq!(variable_id("@ \r\n   "), Ok(("", "@")));
        assert_eq!(variable_id("] \r\n   \t"), Ok(("", "]")));
        assert_eq!(variable_id("z@\r\n"), Ok(("", "z@")));
    }

    #[test]
    fn test_variable_idx() {
        assert_eq!(variable_idx("1 "), Ok((" ", 1)));
        assert_eq!(variable_idx("23 \r\n     "), Ok((" \r\n     ", 23)));
        assert_eq!(variable_idx("134 \r\n     "), Ok((" \r\n     ", 134)));
    }

    #[test]
    fn test_variable_vector() {
        assert_eq!(variable_vector("[13:0] "), Ok((" ", (13, 0))));
        assert_eq!(variable_vector("[31:0] \r"), Ok((" \r", (31, 0))));
        assert_eq!(variable_vector("[0:0] \r"), Ok((" \r", (0, 0))));
    }

    #[test]
    fn test_variable_ref() {
        assert_eq!(variable_vec_ref(" r [31:0] "), Ok(("", ("r", (31, 0)))));
        assert_eq!(variable_vec_ref(" ab [0:0] "), Ok(("", ("ab", (0, 0)))));
        assert_eq!(
            variable_vec_ref("\r\n ab [0:0]\r\n "),
            Ok(("", ("ab", (0, 0))))
        );
        // assert_eq!(variable_vec_ref("\r\n rst \r\n "), Ok(("", ("rst", (0, 0)))));
    }

    #[test]
    fn test_variable_sca_ref() {
        assert_eq!(variable_sca_ref("clk "), Ok(("", ("clk", (0, 0)))));
        assert_eq!(
            variable_sca_ref("\r\n rst \r\n "),
            Ok(("", ("rst", (0, 0))))
        );
    }

    #[test]
    fn test_var_decl_cmd() {
        assert_eq!(
            var_decl_cmd("$var wire 32 ! x26 [31:0] $end"),
            Ok((
                "",
                Var {
                    var_type: "wire",
                    bw: 32,
                    id: "!",
                    refer: "x26",
                    idx: (31, 0),
                }
            ))
        );

        assert_eq!(
            var_decl_cmd("$var reg 1 $ clk $end"),
            Ok((
                "",
                Var {
                    var_type: "reg",
                    bw: 1,
                    id: "$",
                    refer: "clk",
                    idx: (0, 0),
                }
            ))
        );
    }

    #[test]
    fn test_ver_decl_cmd() {
        assert_eq!(ver_decl_cmd("$version hello $end"), Ok(("", "hello")),);
        assert_eq!(
            ver_decl_cmd("$version\r\n\t Icarus Verilog\r\n$end"),
            Ok(("", "Icarus Verilog")),
        );
    }

    #[test]
    fn test_end_kw() {
        assert_eq!(end_kw("$end"), Ok(("", "$end")));
        assert_eq!(end_kw("\r\n $end\t "), Ok(("", "$end")));
    }

    #[test]
    fn test_simu_kw() {
        assert_eq!(simu_kw(" $dumpall"), Ok(("", "$dumpall")));
        assert_eq!(simu_kw("  $dumpoff"), Ok(("", "$dumpoff")));
        assert_eq!(simu_kw("$dumpon   "), Ok(("", "$dumpon")));
        assert_eq!(simu_kw("$dumpvars     "), Ok(("", "$dumpvars")));
    }

    #[test]
    fn test_simu_time_val() {
        assert_eq!(simu_time_val("1000"), Ok(("", 1000u32)));
        assert_eq!(simu_time_val("1324"), Ok(("", 1324u32)));
    }

    #[test]
    fn test_simu_time() {
        assert_eq!(simu_time("#1234"), Ok(("", 1234u32)));
    }

    #[test]
    fn test_val_chg() {
        assert_eq!(
            val_chg("b101011 #%\r\n"),
            Ok((
                "",
                Val {
                    val: 0b101011u64,
                    vld: '0',
                    id: "#%"
                }
            ))
        );
    }

    #[test]
    fn test_sec_val_chg() {
        assert_eq!(
            sec_val_chg("1#%\r\n"),
            Ok((
                "",
                Val {
                    val: 1u64,
                    vld: '0',
                    id: "#%"
                }
            ))
        );
        assert_eq!(
            sec_val_chg("0:'\r\n"),
            Ok((
                "",
                Val {
                    val: 0u64,
                    vld: '0',
                    id: ":'"
                }
            ))
        );
        assert_eq!(
            sec_val_chg("10@'\r\n"),
            Ok((
                "",
                Val {
                    val: 1u64,
                    vld: '0',
                    id: "0@'"
                }
            ))
        );

        assert_eq!(
            sec_val_chg("x4@\r\n"),
            Ok((
                "",
                Val {
                    val: 0u64,
                    vld: 'x',
                    id: "4@"
                }
            ))
        );
        assert_eq!(
            sec_val_chg("zx4@\r\n"),
            Ok((
                "",
                Val {
                    val: 0u64,
                    vld: 'z',
                    id: "x4@"
                }
            ))
        );
    }

    #[test]
    fn test_vec_val_chg() {
        assert_eq!(
            vec_val_chg("b0xxxxxxxx s\n"),
            Ok((
                "",
                Val {
                    val: 0u64,
                    vld: 'x',
                    id: "s"
                }
            ))
        );
        assert_eq!(
            vec_val_chg("b101011 #%\r\n"),
            Ok((
                "",
                Val {
                    val: 0b101011u64,
                    vld: '0',
                    id: "#%"
                }
            ))
        );
        assert_eq!(
            vec_val_chg("b101000001100001 v\"\n"),
            Ok((
                "",
                Val {
                    val: 0b101000001100001u64,
                    vld: '0',
                    id: "v\""
                }
            ))
        );
        assert_eq!(
            vec_val_chg("b11111111111111111111111001101010 D%\r\n"),
            Ok((
                "",
                Val {
                    val: 0b11111111111111111111111001101010u64,
                    vld: '0',
                    id: "D%"
                }
            ))
        );

        assert_eq!(
            vec_val_chg("bx #%\r\n"),
            Ok((
                "",
                Val {
                    val: 0u64,
                    vld: 'x',
                    id: "#%"
                }
            ))
        );

        assert_eq!(
            vec_val_chg("bz x#%\r\n"),
            Ok((
                "",
                Val {
                    val: 0u64,
                    vld: 'z',
                    id: "x#%"
                }
            ))
        );
    }

    #[test]
    fn test_sec_val() {
        assert_eq!(sec_val("0"), Ok(("", '0')));
        assert_eq!(sec_val("1"), Ok(("", '1')));
        assert_eq!(sec_val("x"), Ok(("", 'x')));
        assert_eq!(sec_val("X"), Ok(("", 'X')));
        assert_eq!(sec_val("z"), Ok(("", 'z')));
        assert_eq!(sec_val("Z"), Ok(("", 'Z')));
    }

    #[test]
    fn test_vec_flag_val() {
        assert_eq!(vec_flag_val("b"), Ok(("", 'b')));
        assert_eq!(vec_flag_val("B"), Ok(("", 'B')));
        assert_eq!(vec_flag_val("r"), Ok(("", 'r')));
        assert_eq!(vec_flag_val("R"), Ok(("", 'R')));
        assert_eq!(vec_flag_val("bx o'\n$end"), Ok(("x o'\n$end", 'b')));
    }

    #[test]
    fn test_header() {
        assert_eq!(
            vcd_header("$date\r\n\t Mon Feb 22 19:49:29 2021\r\n $end\r\n $version\r\n Icarus Verilog\r\n $end\r\n $timescale\r\n 1ps\r\n $end"),
            Ok((
                "",
                Header {
                    dat: "Mon Feb 22 19:49:29 2021",
                    ver: "Icarus Verilog",
                    tsc: TimeScale {
                        num: 1,
                        unit: "ps",
                    },
                }
            ))
        );
    }

    #[test]
    fn test_vcd_init() {
        assert_eq!(
            vcd_init("#0\r\n$dumpvars\r\nbx o'\r\nbx n'\r\n$end"),
            Ok((
                "",
                VcdTimeVal {
                    st_flag: 0u32,
                    tv_list: vec![
                        Val {
                            val: 0u64,
                            vld: 'x',
                            id: "o'"
                        },
                        Val {
                            val: 0u64,
                            vld: 'x',
                            id: "n'"
                        }
                    ],
                }
            ))
        );
    }

    #[test]
    fn test_vcd_timeval() {
        assert_eq!(
            vcd_timeval("#0\r\nbx ok\r\n"),
            Ok((
                "",
                VcdTimeVal {
                    st_flag: 0u32,
                    tv_list: vec![Val {
                        val: 0u64,
                        vld: 'x',
                        id: "ok"
                    }],
                }
            ))
        );
    }

    #[test]
    fn test_vcd_body() {
        assert_eq!(
            vcd_body("#0\r\nbx ok\r\n#110000\r\nb10 R%\r\n"),
            Ok((
                "",
                vec![
                    VcdTimeVal {
                        st_flag: 0u32,
                        tv_list: vec![Val {
                            val: 0u64,
                            vld: 'x',
                            id: "ok"
                        }],
                    },
                    VcdTimeVal {
                        st_flag: 110000u32,
                        tv_list: vec![Val {
                            val: 2u64,
                            vld: '0',
                            id: "R%"
                        }],
                    }
                ]
            ))
        );
    }
}
