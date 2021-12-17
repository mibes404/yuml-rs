use super::utils::Uids;
use super::*;
use crate::model::{
    class::{as_note, Connection, Connector, Element, ElementDetails, RelationProps},
    shared::LabeledElement,
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
    // let left_diamond = tuple((tag("<>"), opt(left_label)));
    // let right_diamond = tuple((opt(right_label), tag("<>")));
    let left_arrow = map(alt((tag("<>"), tag("++"), tag("<"), tag("+"))), as_str);
    let left_arrow_w_label = map(tuple((left_arrow, opt(left_label))), as_connector);
    let right_arrow = map(alt((tag("<>"), tag("++"), tag(">"), tag("+"))), as_str);
    let right_arrow_w_label = map(tuple((opt(right_label), right_arrow)), |(lbl, arrow)| {
        as_connector((arrow, lbl))
    });
    let connection = map(alt((tag("-.-"), tag("-"))), as_str);
    // let composition = tag("++");
    // let aggregation = tag("+");
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

    todo! {}
}

fn as_dots(elements: &[Element]) -> Vec<DotElement> {
    let mut uids = Uids::default();

    // we must collect to borrow uids in subsequent iterator
    #[allow(clippy::needless_collect)]
    let element_details: Vec<ElementDetails> = elements
        .iter()
        .filter_map(|e| {
            if let Element::Connection(_) = &e {
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

    todo! {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_class() {
        let yuml = r#"
[Category]<left_label->[Product]
[Category]<>left_label->[Product]
[Category]++->[Product]
[Category]<-.->[Product]
[Category]<-<>[Product]
[Category]<-right_label>[Product]
[Customer]^[Cool Customer]
[Customer]<>1-orders 0..*>[Order]
        "#;

        parse_class(yuml.as_bytes(), &Options::default());
    }
}
