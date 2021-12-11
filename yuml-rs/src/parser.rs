use itertools::Itertools;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::{
        complete::{alphanumeric0, newline},
        streaming::line_ending,
    },
    combinator::{eof, map, opt},
    multi::{many0, many_till},
    sequence::{delimited, preceded, separated_pair, terminated, tuple},
    IResult,
};
use std::{
    borrow::{BorrowMut, Cow},
    cell::RefCell,
    collections::HashMap,
    hash::Hash,
};

use crate::model::{ActivityDotFile, Arrow, Dot, DotElement, Style};

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

pub enum DotFile {
    Activity(ActivityDotFile),
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

pub fn parse_file(yuml: &[u8]) -> IResult<&[u8], DotFile> {
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

    let (rest, result) = match file_type {
        FileType::Activity => {
            let (rest, activity_file) = parse_activity(rest)?;
            (rest, DotFile::Activity(activity_file))
        }
        FileType::Unsupported => (rest, DotFile::Unsupported),
    };

    Ok((rest, result))
}

#[derive(Debug, Hash, PartialEq, Eq)]
enum Element<'a> {
    StartTag,
    EndTag,
    Activity(ElementProps<'a>),
    Parallel(ElementProps<'a>),
    Decision(ElementProps<'a>),
    Arrow(ArrowProps<'a>),
}

#[derive(Debug)]
struct ElementProps<'a> {
    label: Cow<'a, str>,
    incoming_connections: RefCell<u8>,
}

#[derive(Debug)]
struct ArrowProps<'a> {
    label: Option<Cow<'a, str>>,
    outgoing_connection_id: RefCell<u8>,
}

impl<'a> Hash for ElementProps<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.label.hash(state);
    }
}

impl<'a> PartialEq for ElementProps<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.label == other.label
    }
}

impl<'a> Eq for ElementProps<'a> {}

impl<'a> Hash for ArrowProps<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.label.hash(state);
    }
}

impl<'a> PartialEq for ArrowProps<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.label == other.label
    }
}

impl<'a> Eq for ArrowProps<'a> {}

impl<'a> ElementProps<'a> {
    pub fn new(label: Cow<'a, str>) -> Self {
        Self {
            label,
            incoming_connections: RefCell::new(0),
        }
    }
}

impl<'a> ArrowProps<'a> {
    pub fn new(label: Option<Cow<'a, str>>) -> Self {
        Self {
            label,
            outgoing_connection_id: RefCell::new(0),
        }
    }
}

#[derive(Debug)]
struct ElementRelation<'a> {
    id: usize,
    previous_id: usize,
    next_id: usize,
    element: &'a Element<'a>,
    previous: &'a Element<'a>,
    next: &'a Element<'a>,
}

pub fn parse_activity(yuml: &[u8]) -> IResult<&[u8], ActivityDotFile> {
    let start_tag = map(tag("(start)"), |_s: &[u8]| Element::StartTag);
    let end_tag = map(tag("(end)"), |_s: &[u8]| Element::EndTag);
    let alphanumeric_string = map(take_until(">"), as_str);
    let decision = map(delimited(tag("<"), alphanumeric_string, tag(">")), |s| {
        Element::Decision(ElementProps::new(s))
    });
    let alphanumeric_string = map(take_until(")"), as_str);
    let activity = map(delimited(tag("("), alphanumeric_string, tag(")")), |s| {
        Element::Activity(ElementProps::new(s))
    });
    let alphanumeric_string = map(take_until("|"), as_str);
    let parallel = map(delimited(tag("|"), alphanumeric_string, tag("|")), |s| {
        Element::Parallel(ElementProps::new(s))
    });
    let alphanumeric_string = map(take_until("]"), as_str);
    let label = map(delimited(tag("["), alphanumeric_string, tag("]")), |s| s);
    let arrow = map(tuple((opt(label), tag("->"))), |(lbl, _)| {
        Element::Arrow(ArrowProps::new(lbl))
    });

    let parse_element = alt((start_tag, end_tag, decision, activity, parallel, arrow));
    let parse_line = many_till(parse_element, line_ending);
    let mut parse_lines = many_till(parse_line, eof);

    let (rest, (lines, _)) = parse_lines(yuml)?;
    let elements: Vec<Element> = lines
        .into_iter()
        .flat_map(|(elements, _le)| elements.into_iter())
        .collect();

    let dots = as_dots(elements);
    let activity_file = ActivityDotFile::new(dots);
    Ok((rest, activity_file))
}

fn as_dots(elements: Vec<Element>) -> Vec<DotElement> {
    let mut flattened: HashMap<&Element, usize> = HashMap::new();

    elements.iter().enumerate().for_each(|e| match e.1 {
        Element::StartTag => {
            flattened.insert(e.1, e.0);
        }
        Element::EndTag => {
            flattened.insert(e.1, e.0);
        }
        Element::Activity(_lbl) | Element::Parallel(_lbl) | Element::Decision(_lbl) => {
            flattened.entry(e.1).or_insert(e.0);
        }
        Element::Arrow(_lbl) => {
            flattened.insert(e.1, e.0);
        }
    });

    let lookup = |(idx, e): (usize, &Element)| flattened.get(e).copied().unwrap_or(idx);

    elements
        .iter()
        .enumerate()
        .circular_tuple_windows::<(_, _, _)>()
        .map(|(prev, me, next)| ((lookup(prev), prev.1), (lookup(me), me.1), (lookup(next), next.1)))
        .map(|(prev, me, next)| {
            let next_incoming_connection_count = match &next.1 {
                Element::StartTag | Element::EndTag => 1,
                Element::Activity(props) | Element::Parallel(props) | Element::Decision(props) => {
                    if let Ok(mut incoming_connections) = props.incoming_connections.try_borrow_mut() {
                        *incoming_connections += 1;
                        *incoming_connections
                    } else {
                        1
                    }
                }
                Element::Arrow(_) => 1,
            };

            match &me.1 {
                Element::StartTag | Element::EndTag => {}
                Element::Activity(_) | Element::Parallel(_) | Element::Decision(_) => {}
                Element::Arrow(props) => {
                    if let Ok(mut outgoing_connection_count) = props.outgoing_connection_id.try_borrow_mut() {
                        println!("{}", next_incoming_connection_count);
                        *outgoing_connection_count = next_incoming_connection_count
                    }
                }
            }

            ElementRelation {
                id: me.0,
                previous_id: prev.0,
                next_id: next.0,
                element: me.1,
                previous: prev.1,
                next: next.1,
            }
        })
        .map(|e| DotElement::from(&e))
        .collect()
}

impl<'a> From<&ElementRelation<'a>> for DotElement {
    fn from(e: &ElementRelation<'a>) -> Self {
        match e.element {
            Element::StartTag | Element::EndTag => DotElement {
                dot: Dot::from(e.element),
                uid: format!("A{}", e.id),
                uid2: None,
            },
            Element::Activity(_lbl) | Element::Parallel(_lbl) | Element::Decision(_lbl) => DotElement {
                dot: Dot::from(e.element),
                uid: format!("A{}", e.id),
                uid2: None,
            },
            Element::Arrow(props) => {
                let outgoing_connection_count = *(props.outgoing_connection_id.borrow());
                let uid2 = if outgoing_connection_count > 1 {
                    format!("A{}:f{}:n", e.next_id, outgoing_connection_count + 1)
                } else {
                    format!("A{}", e.next_id)
                };

                DotElement {
                    dot: Dot::from(e.element),
                    uid: format!("A{}", e.previous_id),
                    uid2: Some(uid2),
                }
            }
        }
    }
}

impl<'a> From<&Element<'a>> for Dot {
    fn from(e: &Element<'a>) -> Self {
        match e {
            Element::StartTag => Dot {
                shape: crate::model::DotShape::Circle,
                height: Some(0.3),
                width: Some(0.3),
                ..Dot::default()
            },
            Element::EndTag => Dot {
                shape: crate::model::DotShape::DoubleCircle,
                height: Some(0.3),
                width: Some(0.3),
                ..Dot::default()
            },
            Element::Activity(ElementProps {
                label,
                incoming_connections: _incoming_connections,
            }) => Dot {
                shape: crate::model::DotShape::Rectangle,
                height: Some(0.5),
                margin: Some("0.20,0.05".to_string()),
                label: Some(label.clone().into_owned()),
                style: vec![Style::Rounded],
                fontsize: Some(10),
                ..Dot::default()
            },
            Element::Parallel(props) => {
                let incoming_connections = *props.incoming_connections.borrow();
                let label = (0..=incoming_connections).map(|i| format!("<f{}>", i + 1)).join("|");

                Dot {
                    shape: crate::model::DotShape::Record,
                    height: Some(0.05),
                    width: Some(0.5),
                    penwidth: Some(4),
                    label: Some(label),
                    style: vec![Style::Filled],
                    fontsize: Some(1),
                    ..Dot::default()
                }
            }
            Element::Decision(ElementProps {
                label,
                incoming_connections: _incoming_connections,
            }) => Dot {
                shape: crate::model::DotShape::Diamond,
                height: Some(0.5),
                width: Some(0.5),
                label: Some(label.clone().into_owned()),
                fontsize: Some(0),
                ..Dot::default()
            },
            Element::Arrow(props) => Dot {
                shape: crate::model::DotShape::Edge,
                style: vec![Style::Solid],
                dir: Some("both".to_string()),
                arrowhead: Some(Arrow::Vee),
                fontsize: Some(10),
                labeldistance: Some(1),
                label: props.label.as_ref().map(|s| s.clone().into_owned()),
                ..Dot::default()
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_activity() {
        let yuml = include_bytes!("../test/activity.yuml");
        if let (rest, DotFile::Activity(activity_file)) = parse_file(yuml).expect("invalid file") {
            assert!(rest.is_empty());
            println!("{}", activity_file);
        } else {
            panic!("Invalid file");
        }
    }
}
