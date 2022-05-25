use std::str::from_utf8;

use nom::bits::bits;
use nom::bits::streaming::take;
use nom::bytes::streaming::tag;
use nom::combinator::{flat_map, map, map_res};
use nom::error::{Error, ErrorKind};
use nom::multi::{length_data, many0, many_m_n};
use nom::number::streaming::{be_f64, be_i16, be_i24, be_u16, be_u24, be_u32, be_u8};
use nom::sequence::{pair, terminated, tuple};
use nom::{Err, IResult, Needed};

#[cfg(test)]
mod test {
}