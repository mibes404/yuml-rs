use super::utils::Uids;
use super::*;
use crate::model::{
    activity::{as_note, ArrowProps, Element, ElementProps},
    shared::{ElementDetails, LabeledElement, Relation},
};

/*
Syntax as specified in yuml.me
Start	           (start)
End	               (end)
Activity           (Find Products)
Flow	           (start)->(Find Products)
Multiple Assoc.    (start)->(Find Products)->(end)
Decisions          (start)-><d1>
Decisions w/Label  (start)-><d1>logged in->(Show Dashboard), <d1>not logged in->(Show Login Page)
Parallel	       (Action1)->|a|,(Action 2)->|a|
Note               (Action1)-(note: A note message here)
Comment            // Comments
*/

pub fn parse_activity<'a, 'o>(yuml: &'a [u8], options: &'o Options) -> IResult<&'a [u8], DotFile> {
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
    let parse_line = many_till(parse_element, alt((eof, line_ending)));
    let mut parse_lines = many_till(parse_line, eof);

    let (rest, (lines, _)) = parse_lines(yuml)?;
    let elements: Vec<Element> = lines
        .into_iter()
        .flat_map(|(elements, _le)| elements.into_iter())
        .collect();

    let dots = as_dots(&elements);
    let activity_file = DotFile::new(dots, options);
    Ok((rest, activity_file))
}

fn as_dots(elements: &[Element]) -> Vec<DotElement> {
    let mut uids = Uids::default();

    // we must collect to borrow uids in subsequent iterator
    #[allow(clippy::needless_collect)]
    let element_details: Vec<ElementDetails<Element>> = elements
        .iter()
        .filter_map(|e| {
            if e.is_connection() {
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
    let arrow_details: Vec<ElementDetails<Element>> = elements
        .iter()
        .circular_tuple_windows::<(_, _, _)>()
        .filter(|(pre, _e, next)| !pre.is_connection() && !next.is_connection())
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
        let yuml = include_bytes!("../../test/activity.yuml");
        if let (rest, ParsedYuml::Activity(activity_file)) = parse_yuml(yuml).expect("invalid file") {
            assert!(rest.is_empty());
            println!("{}", activity_file);
        } else {
            panic!("Invalid file");
        }
    }

    #[test]
    fn test_parse_big_activity() {
        let yuml = include_bytes!("../../test/big_activity.yuml");
        if let (rest, ParsedYuml::Activity(activity_file)) = parse_yuml(yuml).expect("invalid file") {
            assert!(rest.is_empty());
            println!("{}", activity_file);
        } else {
            panic!("Invalid file");
        }
    }
}
