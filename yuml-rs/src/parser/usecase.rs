use super::utils::populate_uids;
use super::*;
use crate::model::{
    shared::{ElementDetails, LabeledElement, Relation},
    usecase::{as_note, ConnectionProps, Element},
};

/*
Syntax as specified in yuml.me
Use Case	        (Login)
Actor	            [Customer]
<<Extend>>	        (Login)<(Forgot Password)
<<Include>>	        (Register)>(Confirm Email)
Actor Inheritance	[Admin]^[User]
Notes	            [Admin]^[User],[Admin]-(note: Most privileged user)
*/

pub fn parse_usecase<'a, 'o>(yuml: &'a str, options: &'o Options) -> IResult<&'a str, DotFile> {
    let note_string = take_until("}");
    let note_props = delimited(tag("{"), note_string, tag("}"));
    let note = take_until("{");
    let extract_attributes = map(tuple((note, opt(note_props))), as_note);
    let alphanumeric_string = take_until(")");
    let note = map_parser(
        delimited(tag("(note:"), alphanumeric_string, tag(")")),
        extract_attributes,
    );
    let alphanumeric_string = take_until(")");
    let use_case = map(delimited(tag("("), alphanumeric_string, tag(")")), |lbl| {
        Element::UseCase(lbl)
    });
    let alphanumeric_string = take_until("]");
    let actor = map(delimited(tag("["), alphanumeric_string, tag("]")), |lbl| {
        Element::Actor(lbl)
    });
    let inheritance = map(tag("^"), |_| Element::Inheritance);
    let connection = map(alt((tag("<"), tag(">"))), |found| {
        if found == "<" {
            Element::Connection(ConnectionProps::Extend)
        } else {
            Element::Connection(ConnectionProps::Include)
        }
    });

    let parse_element = alt((note, actor, use_case, connection, inheritance));
    let parse_line = many_till(parse_element, alt((eof, line_ending)));
    let mut parse_lines = many_till(parse_line, eof);

    let (rest, (lines, _)) = parse_lines(yuml)?;
    let elements: Vec<Element> = lines
        .into_iter()
        .flat_map(|(elements, _le)| elements.into_iter())
        .collect();

    let dots = as_dots(&elements);
    let class_file = DotFile::new(dots, options).sep(0.7);
    Ok((rest, class_file))
}

fn as_dots(elements: &[Element]) -> Vec<DotElement> {
    let (uids, element_details) = populate_uids(elements);

    // we must collect to ensure the incoming connections are all processed, before creating the dot file
    #[allow(clippy::needless_collect)]
    let arrow_details: Vec<ElementDetails<Element>> = elements
        .iter()
        .circular_tuple_windows::<(_, _, _)>()
        .filter(|(pre, _e, next)| !pre.is_connection() && !next.is_connection())
        .filter_map(|(pre, e, next)| match e {
            Element::Connection(_props) => Some((pre, e, next)),
            Element::Inheritance => Some((pre, e, next)),
            _ => None,
        })
        .filter_map(|(pre, e, next)| {
            // if I am a connection
            let previous_id = uids.get(pre.label()).map(|(idx, _e)| *idx).unwrap_or_default();
            let (next_id, _next_e) = match uids.get(next.label()) {
                Some((idx, e)) => (*idx, e),
                None => {
                    // arrow pointing in the void
                    return None;
                }
            };

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
    fn test_parse_class() {
        let yuml = include_str!("../../test/class.yuml");
        if let (rest, ParsedYuml::Class(activity_file)) = parse_yuml(yuml).expect("invalid file") {
            assert!(rest.is_empty());
            println!("{}", activity_file);
        } else {
            panic!("Invalid file");
        }
    }
}
