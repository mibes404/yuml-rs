use std::borrow::Cow;

use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::{
        complete::{alphanumeric0, anychar, newline, not_line_ending},
        is_newline,
        streaming::line_ending,
    },
    combinator::{cond, eof, map, opt},
    multi::{many0, many_till},
    sequence::{delimited, preceded, separated_pair, terminated},
    IResult,
};

use crate::activity::Activity;

pub struct Header<'a> {
    pub key: Cow<'a, str>,
    pub value: Cow<'a, str>,
}

fn as_str(b: &[u8]) -> Cow<str> {
    String::from_utf8_lossy(b)
}

fn vec_as_str(v: Vec<char>) -> Option<String> {
    if v.is_empty() {
        None
    } else {
        Some(v.iter().collect())
    }
}

fn as_header<'a>(kv: (Cow<'a, str>, Cow<'a, str>)) -> Header<'a> {
    Header { key: kv.0, value: kv.1 }
}

#[derive(Debug, PartialEq)]
enum FileType {
    Activity,
    Unsupported,
}

impl From<&Cow<'_, str>> for FileType {
    fn from(c: &Cow<str>) -> Self {
        match c.as_ref() {
            "activity" => FileType::Activity,
            _ => FileType::Unsupported,
        }
    }
}

fn determine_filetype(headers: &[Header]) -> FileType {
    headers
        .iter()
        .filter(|h| h.key.as_ref() == "type")
        .map(|h| FileType::from(&h.value))
        .next()
        .unwrap_or(FileType::Unsupported)
}

pub fn parse_file(yuml: &[u8]) -> IResult<&[u8], ()> {
    let alphanumeric_string = map(alphanumeric0, as_str);
    let alphanumeric_string_2 = map(alphanumeric0, as_str);
    let parse_key_value = separated_pair(alphanumeric_string, tag(":"), alphanumeric_string_2);
    let parse_header = delimited(tag("{"), parse_key_value, tag("}"));
    let parse_header = terminated(preceded(tag("// "), parse_header), newline);
    let parse_header = map(parse_header, as_header);
    let mut parse_headers = many0(parse_header);

    let (rest, headers) = parse_headers(yuml)?;

    assert_eq!(headers.len(), 2);
    let file_type = determine_filetype(&headers);
    assert_eq!(file_type, FileType::Activity);

    match file_type {
        FileType::Activity => parse_activity(rest)?,
        FileType::Unsupported => todo!(),
    };

    Ok((rest, ()))
}

enum Element<'a> {
    StartTag,
    EndTag,
    Activity(Cow<'a, str>),
    Parallel(Cow<'a, str>),
    Decision(Cow<'a, str>),
    Arrow,
    Label(Cow<'a, str>),
}

pub fn parse_activity(yuml: &[u8]) -> IResult<&[u8], ()> {
    let start_tag = map(tag("(start)"), |_s: &[u8]| Element::StartTag);
    let end_tag = map(tag("(end)"), |_s: &[u8]| Element::EndTag);
    let alphanumeric_string = map(take_until(">"), as_str);
    let decision = map(delimited(tag("<"), alphanumeric_string, tag(">")), |s| {
        Element::Decision(s)
    });
    let alphanumeric_string = map(take_until(")"), as_str);
    let activity = map(delimited(tag("("), alphanumeric_string, tag(")")), |s| {
        Element::Decision(s)
    });
    let alphanumeric_string = map(take_until("|"), as_str);
    let parallel = map(delimited(tag("|"), alphanumeric_string, tag("|")), |s| {
        Element::Decision(s)
    });
    let alphanumeric_string = map(take_until("]"), as_str);
    let label = map(delimited(tag("["), alphanumeric_string, tag("]")), |s| {
        Element::Label(s)
    });

    // let label = map(take_until("->"), |s| Element::Label(as_str(s)));
    let arrow = map(tag("->"), |_| Element::Arrow);

    let parse_element = alt((start_tag, decision, activity, parallel, arrow, label, end_tag));
    let parse_line = many_till(parse_element, line_ending);
    let mut parse_lines = many_till(parse_line, eof);

    let (rest, (lines, _)) = parse_lines(yuml)?;
    println!("{}", String::from_utf8_lossy(rest));

    let elements: Vec<Element> = lines
        .into_iter()
        .flat_map(|(elements, _le)| elements.into_iter())
        .collect();

    assert_eq!(elements.len(), 28);

    Ok((rest, ()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_activity() {
        let yuml = include_bytes!("../test/activity.yuml");
        parse_file(yuml).expect("invalid file");
    }
}
