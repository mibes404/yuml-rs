use crate::model::{
    activity::{ArrowProps, Element, ElementDetails, ElementProps, NoteProps, Relation},
    dot::{ActivityDotFile, ChartType, Directions, DotElement, Options},
};
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

fn as_note<'a>(note: (&'a [u8], Option<&'a [u8]>)) -> Element {
    let label = as_str(note.0);
    let attributes = note.1.map(as_str);
    Element::Note(NoteProps { label, attributes })
}

pub enum DotFile {
    Activity(ActivityDotFile),
    Unsupported,
}

impl std::fmt::Display for DotFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DotFile::Activity(af) => af.fmt(f),
            DotFile::Unsupported => f.write_str(""),
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

pub fn parse_yuml(yuml: &[u8]) -> IResult<&[u8], DotFile> {
    let alphanumeric_string = map(alphanumeric0, as_str);
    let alphanumeric_string_2 = map(alphanumeric0, as_str);
    let parse_key_value = separated_pair(alphanumeric_string, tag(":"), alphanumeric_string_2);
    let parse_header = delimited(tag("{"), parse_key_value, tag("}"));
    let parse_header = terminated(preceded(tag("// "), parse_header), newline);
    let parse_header = map(parse_header, as_header);
    let mut parse_headers = many0(parse_header);

    let (rest, headers) = parse_headers(yuml)?;
    let options = determine_file_options(&headers);

    let (rest, result) = match options.chart_type {
        Some(ChartType::Activity) => {
            let (rest, activity_file) = parse_activity(rest, &options)?;
            (rest, DotFile::Activity(activity_file))
        }
        _ => (rest, DotFile::Unsupported),
    };

    Ok((rest, result))
}

pub fn parse_activity<'a, 'o>(yuml: &'a [u8], options: &'o Options) -> IResult<&'a [u8], ActivityDotFile> {
    let start_tag = map(tag("(start)"), |_s: &[u8]| Element::StartTag);
    let end_tag = map(tag("(end)"), |_s: &[u8]| Element::EndTag);
    let note_string = take_until("}");
    let note_props = delimited(tag("{"), note_string, tag("}"));
    let note = take_until("{");
    let extract_attributes = map(tuple((note, opt(note_props))), as_note);
    let alphanumeric_string = take_until(")");
    let note = map_parser(
        delimited(tag("(note:"), alphanumeric_string, tag(")")),
        extract_attributes,
    );
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
    let alphanumeric_string = map(take_until("->"), as_str);
    let arrow_w_label = map(terminated(alphanumeric_string, tag("->")), |lbl| {
        Element::Arrow(ArrowProps::new(Some(lbl), &options.dir))
    });
    let arrow_wo_label = map(tag("->"), |_| Element::Arrow(ArrowProps::new(None, &options.dir)));
    let arrow = alt((arrow_wo_label, arrow_w_label));

    let parse_element = alt((start_tag, end_tag, decision, note, activity, parallel, arrow));
    let parse_line = many_till(parse_element, line_ending);
    let mut parse_lines = many_till(parse_line, eof);

    let (rest, (lines, _)) = parse_lines(yuml)?;
    let elements: Vec<Element> = lines
        .into_iter()
        .flat_map(|(elements, _le)| elements.into_iter())
        .collect();

    let dots = as_dots(&elements);
    let activity_file = ActivityDotFile::new(dots, options);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_activity() {
        let yuml = include_bytes!("../test/activity.yuml");
        if let (rest, DotFile::Activity(activity_file)) = parse_yuml(yuml).expect("invalid file") {
            assert!(rest.is_empty());
            println!("{}", activity_file);
        } else {
            panic!("Invalid file");
        }
    }
}
