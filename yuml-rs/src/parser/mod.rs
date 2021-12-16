use crate::model::dot::{ChartType, Directions, DotElement, DotFile, Options};
use itertools::Itertools;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::{
        complete::{alphanumeric0, newline},
        streaming::line_ending,
    },
    combinator::{eof, map, map_parser, opt},
    multi::{many0, many_till},
    sequence::{delimited, preceded, separated_pair, terminated, tuple},
    IResult,
};
use std::{
    borrow::{Borrow, Cow},
    collections::HashMap,
};

use self::{activity::parse_activity, utils::as_str};

mod activity;
mod class;
pub mod utils;

pub enum ParsedYuml {
    Activity(DotFile),
    Unsupported,
}

pub struct Header<'a> {
    pub key: Cow<'a, str>,
    pub value: Cow<'a, str>,
}

fn as_header<'a>(kv: (Cow<'a, str>, Cow<'a, str>)) -> Header<'a> {
    Header { key: kv.0, value: kv.1 }
}

impl std::fmt::Display for ParsedYuml {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParsedYuml::Activity(af) => af.fmt(f),
            ParsedYuml::Unsupported => f.write_str(""),
        }
    }
}

fn determine_file_options(headers: &[Header]) -> Options {
    let mut options = Options::default();

    for h in headers.iter() {
        match h.key.as_ref() {
            "type" => options.chart_type = ChartType::try_from(h.value.as_ref()).ok(),
            "direction" => options.dir = Directions::try_from(h.value.as_ref()).unwrap_or_default(),
            _ => { /* ignore unsupported headers */ }
        }
    }

    options
}

pub fn parse_yuml(yuml: &[u8]) -> IResult<&[u8], ParsedYuml> {
    let alphanumeric_string = map(alphanumeric0, as_str);
    let alphanumeric_string_2 = map(alphanumeric0, as_str);
    let parse_key_value = separated_pair(alphanumeric_string, tag(":"), alphanumeric_string_2);
    let parse_header = delimited(tag("{"), parse_key_value, tag("}"));
    let parse_header = terminated(preceded(tag("// "), parse_header), newline);
    let parse_header = map(parse_header, as_header);
    let prefix_empty_lines = many0(line_ending);
    let mut parse_headers = tuple((prefix_empty_lines, many0(parse_header)));

    let (rest, (_, headers)) = parse_headers(yuml)?;
    let options = determine_file_options(&headers);

    let (rest, result) = match options.chart_type {
        Some(ChartType::Activity) => {
            let (rest, activity_file) = parse_activity(rest, &options)?;
            (rest, ParsedYuml::Activity(activity_file))
        }
        _ => (rest, ParsedYuml::Unsupported),
    };

    Ok((rest, result))
}
