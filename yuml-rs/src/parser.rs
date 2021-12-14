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
    borrow::{Borrow, Cow},
    cell::RefCell,
    collections::HashMap,
};

use crate::model::{ActivityDotFile, Arrow, Dot, DotElement, Style};

pub struct Header<'a> {
    pub key: Cow<'a, str>,
    pub value: Cow<'a, str>,
}

fn as_str(b: &[u8]) -> Cow<str> {
    String::from_utf8_lossy(b)
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

impl std::fmt::Display for DotFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DotFile::Activity(af) => f.write_str(&af.to_string()),
            DotFile::Unsupported => f.write_str(""),
        }
    }
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

#[derive(Debug)]
enum Element<'a> {
    StartTag,
    EndTag,
    Activity(ElementProps<'a>),
    Parallel(ElementProps<'a>),
    Decision(ElementProps<'a>),
    Arrow(ArrowProps<'a>),
    Note(Cow<'a, str>),
}

impl<'a> Element<'a> {
    pub fn label(&self) -> Cow<'a, str> {
        match self {
            Element::StartTag => Cow::from("start"),
            Element::EndTag => Cow::from("end"),
            Element::Activity(props) | Element::Parallel(props) | Element::Decision(props) => props.label.clone(),
            Element::Arrow(details) => details.label.clone().unwrap_or_default(),
            Element::Note(label) => label.clone(),
        }
    }

    pub fn is_arrow(&self) -> bool {
        matches!(self, Element::Arrow(_))
    }

    pub fn is_note(&self) -> bool {
        matches!(self, Element::Note(_))
    }
}

#[derive(Debug)]
struct ElementProps<'a> {
    label: Cow<'a, str>,
    incoming_connections: RefCell<u8>,
}

#[derive(Debug)]
struct ArrowProps<'a> {
    label: Option<Cow<'a, str>>,
    target_connection_id: RefCell<u8>,
    dashed: RefCell<bool>,
}

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
            target_connection_id: RefCell::new(0),
            dashed: RefCell::new(false),
        }
    }
}

#[derive(Debug)]
struct ElementDetails<'a> {
    id: Option<usize>,
    element: &'a Element<'a>,
    relation: Option<Relation>,
}

#[derive(Debug)]
struct Relation {
    previous_id: usize,
    next_id: usize,
}

pub fn parse_activity(yuml: &[u8]) -> IResult<&[u8], ActivityDotFile> {
    let start_tag = map(tag("(start)"), |_s: &[u8]| Element::StartTag);
    let end_tag = map(tag("(end)"), |_s: &[u8]| Element::EndTag);
    let alphanumeric_string = map(take_until(")"), as_str);
    let note = map(delimited(tag("(note:"), alphanumeric_string, tag(")")), |s| {
        Element::Note(s)
    });
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

    let parse_element = alt((start_tag, end_tag, decision, note, activity, parallel, arrow));
    let parse_line = many_till(parse_element, line_ending);
    let mut parse_lines = many_till(parse_line, eof);

    let (rest, (lines, _)) = parse_lines(yuml)?;
    let elements: Vec<Element> = lines
        .into_iter()
        .flat_map(|(elements, _le)| elements.into_iter())
        .collect();

    let dots = as_dots(&elements);
    let activity_file = ActivityDotFile::new(dots);
    Ok((rest, activity_file))
}

#[derive(Default)]
struct Uids<'a> {
    uids: HashMap<Cow<'a, str>, (usize, &'a Element<'a>)>,
    uid: usize,
}

impl<'a> Uids<'a> {
    fn insert_uid(&mut self, label: Cow<'a, str>, e: &'a Element<'a>) -> usize {
        self.uid += 1;
        self.uids.insert(label, (self.uid, e));
        self.uid
    }

    fn contains_key(&self, key: &str) -> bool {
        self.uids.contains_key(key)
    }

    fn get(&'a self, key: &str) -> Option<&'a (usize, &'a Element<'a>)> {
        self.uids.get(key)
    }
}

fn as_dots(elements: &[Element]) -> Vec<DotElement> {
    let mut uids = Uids::default();

    // we must collect to borrow uids in subsequent iterator
    #[allow(clippy::needless_collect)]
    let element_details: Vec<ElementDetails> = elements
        .iter()
        .filter_map(|e| {
            if let Element::Arrow(_) = &e {
                // ignore arrows for now
                None
            } else {
                let lbl = e.label();
                if uids.contains_key(&lbl) {
                    None
                } else {
                    let id = uids.insert_uid(lbl, e);
                    Some((id, e))
                }
            }
        })
        .map(|(id, element)| ElementDetails {
            id: Some(id),
            element,
            relation: None,
        })
        .collect();

    // we must collect to ensure the incoming connections are all processed, before creating the dot file
    #[allow(clippy::needless_collect)]
    let arrow_details: Vec<ElementDetails> = elements
        .iter()
        .circular_tuple_windows::<(_, _, _)>()
        .filter(|(pre, _e, next)| !pre.is_arrow() && !next.is_arrow())
        .filter_map(|(pre, e, next)| {
            if let Element::Arrow(props) = e {
                Some((pre, e, props, next))
            } else {
                None
            }
        })
        .filter_map(|(pre, e, props, next)| {
            // if I am an arrow
            if pre.is_note() || next.is_note() {
                let mut dashed = props.dashed.borrow_mut();
                *dashed = true;
            }

            let previous_id = uids.get(&pre.label()).map(|(idx, _e)| *idx).unwrap_or_default();
            let (next_id, next_e) = match uids.get(&next.label()) {
                Some((idx, e)) => (*idx, e),
                None => {
                    // arrow pointing in the void
                    return None;
                }
            };

            let target_connection = if let Element::Parallel(props) = next_e {
                let mut incoming_connections = props.incoming_connections.borrow_mut();
                *incoming_connections += 1;
                *incoming_connections
            } else {
                0
            };

            let mut target_connection_id = props.target_connection_id.borrow_mut();
            *target_connection_id = target_connection;

            let r = Relation { previous_id, next_id };
            Some(ElementDetails {
                id: None,
                element: e,
                relation: Some(r),
            })
        })
        .collect();

    element_details
        .into_iter()
        .chain(arrow_details.into_iter())
        .map(|e| DotElement::from(e.borrow()))
        .collect()
}

impl<'a> From<&ElementDetails<'a>> for DotElement {
    fn from(e: &ElementDetails<'a>) -> Self {
        match e.element {
            Element::StartTag | Element::EndTag => DotElement {
                dot: Dot::from(e.element),
                uid: format!("A{}", e.id.unwrap_or_default()),
                uid2: None,
            },
            Element::Activity(_) | Element::Parallel(_) | Element::Decision(_) | Element::Note(_) => DotElement {
                dot: Dot::from(e.element),
                uid: format!("A{}", e.id.unwrap_or_default()),
                uid2: None,
            },
            Element::Arrow(props) => {
                let target_connection_id = *(props.target_connection_id.borrow());
                let (uid1, uid2) = if let Some(relation) = &e.relation {
                    let uid1 = format!("A{}", relation.previous_id);
                    let uid2 = if target_connection_id > 0 {
                        format!("A{}:f{}:n", relation.next_id, target_connection_id)
                    } else {
                        format!("A{}", relation.next_id)
                    };
                    (uid1, uid2)
                } else {
                    ("A0".to_string(), "A0".to_string())
                };

                DotElement {
                    dot: Dot::from(e.element),
                    uid: uid1,
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
            Element::Activity(props) => Dot {
                shape: crate::model::DotShape::Rectangle,
                height: Some(0.5),
                margin: Some("0.20,0.05".to_string()),
                label: Some(props.label.clone().into_owned()),
                style: vec![Style::Rounded],
                fontsize: Some(10),
                ..Dot::default()
            },
            Element::Parallel(props) => {
                let incoming_connections = *props.incoming_connections.borrow();
                let label = (1..=incoming_connections).map(|i| format!("<f{}>", i)).join("|");

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
            Element::Decision(props) => Dot {
                shape: crate::model::DotShape::Diamond,
                height: Some(0.5),
                width: Some(0.5),
                label: Some(props.label.clone().into_owned()),
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
            // A1 [shape="note" , margin="0.20,0.05" , label="You can stick notes on diagrams too!\\{bg:cornsilk\\}" , style="filled" , fillcolor="cornsilk" , fontcolor="black" , arrowtail="none" , arrowhead="none" , height=0.5 , fontsize=10 , ]
            Element::Note(label) => Dot {
                shape: crate::model::DotShape::Note,
                height: Some(0.5),
                margin: Some("0.20,0.05".to_string()),
                label: Some(label.clone().into_owned()),
                fontsize: Some(10),
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
