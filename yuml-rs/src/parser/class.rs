use super::utils::Uids;
use super::*;
use crate::model::{
    class::{as_note, Connection, Connector, Element, RelationProps},
    shared::{ElementDetails, LabeledElement, Relation},
};
use nom::bytes::complete::{is_not, take_until1, take_while};

/*
Syntax as specified in yuml.me
Class           [Customer]
Directional     [Customer]->[Order]
Bidirectional   [Customer]<->[Order]
Aggregation     [Customer]+-[Order] or [Customer]<>-[Order]
Composition     [Customer]++-[Order]
Inheritance     [Customer]^[Cool Customer], [Customer]^[Uncool Customer]
Dependencies    [Customer]uses-.->[PaymentStrategy]
Cardinality     [Customer]<1-1..2>[Address]
Labels          [Person]customer-billingAddress[Address]
Notes           [Person]-[Address],[Address]-[note: Value Object]
Full Class      [Customer|Forename;Surname;Email|Save()]
Color splash    [Customer{bg:orange}]<>1->*[Order{bg:green}]
Comment         // Comments
*/

fn as_connector<'a>((arrow, label): (Cow<'a, str>, Option<Cow<'a, str>>)) -> Connector<'a> {
    match arrow.as_ref() {
        "<>" | "+" => Connector::Aggregation(RelationProps { label }),
        "++" => Connector::Composition(RelationProps { label }),
        _ => Connector::Directional(RelationProps { label }),
    }
}

pub fn parse_class<'a, 'o>(yuml: &'a [u8], options: &'o Options) -> IResult<&'a [u8], DotFile> {
    let note_string = take_until("}");
    let note_props = delimited(tag("{"), note_string, tag("}"));
    let note = take_until("{");
    let extract_attributes = map(tuple((note, opt(note_props))), as_note);
    let alphanumeric_string = take_until("]");
    let note = map_parser(
        delimited(tag("[note:"), alphanumeric_string, tag("]")),
        extract_attributes,
    );

    let alphanumeric_string = map(take_until("]"), as_str);
    let class = map(delimited(tag("["), alphanumeric_string, tag("]")), |lbl| {
        Element::Class(lbl)
    });

    let right_label = map(is_not("<>+"), as_str);
    let left_label = map(take_until1("-"), as_str);
    let left_arrow = map(alt((tag("<>"), tag("++"), tag("<"), tag("+"))), as_str);
    let left_arrow_w_label = map(tuple((left_arrow, opt(left_label))), as_connector);
    let right_arrow = map(alt((tag("<>"), tag("++"), tag(">"), tag("+"))), as_str);
    let right_arrow_w_label = map(tuple((opt(right_label), right_arrow)), |(lbl, arrow)| {
        as_connector((arrow, lbl))
    });
    let connection = map(alt((tag("-.-"), tag("-"))), as_str);
    let connector = map(tuple((left_arrow_w_label, connection, right_arrow_w_label)), |t| {
        let dotted = t.1.as_ref() == "-.-";
        println!("Found: {:?}", t);
        Element::Connection(Connection {
            dotted,
            ..Connection::default()
        })
    });
    let inheritance = map(tag("^"), |_| Element::Inheritance);

    // let directional = map(tag("->"), |_| Element::Directional(RelationProps::default()));

    let parse_element = alt((note, class, connector, inheritance));
    let parse_line = many_till(parse_element, line_ending);
    let mut parse_lines = many_till(parse_line, eof);

    let (rest, (lines, _)) = parse_lines(yuml)?;
    let elements: Vec<Element> = lines
        .into_iter()
        .flat_map(|(elements, _le)| elements.into_iter())
        .collect();

    let dots = as_dots(&elements);
    let class_file = DotFile::new(dots, options);
    Ok((rest, class_file))
}

fn as_dots(elements: &[Element]) -> Vec<DotElement> {
    let mut uids = Uids::default();

    // we must collect to borrow uids in subsequent iterator
    #[allow(clippy::needless_collect)]
    let element_details: Vec<ElementDetails<Element>> = elements
        .iter()
        .filter_map(|e| {
            if e.is_connection() {
                // ignore connections for now
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
            if let Element::Connection(props) = e {
                Some((pre, e, props, next))
            } else {
                None
            }
        })
        .filter_map(|(pre, e, _props, next)| {
            // if I am a connection
            let previous_id = uids.get(&pre.label()).map(|(idx, _e)| *idx).unwrap_or_default();
            let (next_id, _next_e) = match uids.get(&next.label()) {
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
        let yuml = include_bytes!("../../test/class.yuml");
        if let (rest, ParsedYuml::Class(activity_file)) = parse_yuml(yuml).expect("invalid file") {
            assert!(rest.is_empty());
            println!("{}", activity_file);
        } else {
            panic!("Invalid file");
        }
    }
}
