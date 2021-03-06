use super::utils::populate_uids;
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

pub fn note_or_actvity(yuml: &str) -> IResult<&str, Element> {
    let note_string = take_until("}");
    let note_props = delimited(tag("{"), note_string, tag("}"));
    let note_text = alt((take_until("{"), rest));
    let extract_attributes = map(tuple((note_text, opt(note_props))), as_note);
    let extract_note = map_parser(preceded(tag("note:"), rest), extract_attributes);
    let extract_activity = map(rest, |s| Element::Activity(ElementProps::new(s)));
    let mut n_or_a = alt((extract_note, extract_activity));

    n_or_a(yuml)
}

fn parse_activity_elem(yuml: &str) -> IResult<&str, Element> {
    let activity = preceded(tag("("), parse_until_end_of_activity);
    let mut activity = map_res(activity, |s| note_or_actvity(s).map(|(_, b)| b));
    activity(yuml)
}

pub fn parse_activity<'a, 'o>(yuml: &'a str, options: &'o Options) -> IResult<&'a str, DotFile> {
    let start_tag = map(tag("(start)"), |_s: &str| Element::StartTag);
    let end_tag = map(tag("(end)"), |_s: &str| Element::EndTag);
    let alphanumeric_string = take_until(">");
    let decision = map(delimited(tag("<"), alphanumeric_string, tag(">")), |s| {
        Element::Decision(ElementProps::new(s))
    });
    let alphanumeric_string = take_until("|");
    let parallel = map(delimited(tag("|"), alphanumeric_string, tag("|")), |s| {
        Element::Parallel(ElementProps::new(s))
    });
    let alphanumeric_string = take_until("->");
    let arrow_w_label = map(terminated(alphanumeric_string, tag("->")), |lbl| {
        Element::Arrow(ArrowProps::new(Some(lbl), &options.dir, true))
    });
    let arrow_wo_label = map(tag("->"), |_| Element::Arrow(ArrowProps::new(None, &options.dir, true)));
    let no_tail_arrow_wo_label = map(tag("-"), |_| Element::Arrow(ArrowProps::new(None, &options.dir, false)));

    let arrow = alt((arrow_wo_label, arrow_w_label, no_tail_arrow_wo_label));

    let parse_element = alt((start_tag, end_tag, decision, parse_activity_elem, parallel, arrow));
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
    let (uids, element_details) = populate_uids(elements);

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

            let previous_id = uids.get(pre.label()).map(|(idx, _e)| *idx).unwrap_or_default();
            let (next_id, next_e) = match uids.get(next.label()) {
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

fn parse_until_end_of_activity(yuml: &str) -> IResult<&str, &str> {
    let mut last_char: Option<char> = None;
    for (idx, c) in yuml.char_indices() {
        if c == ')' {
            if let Some(lc) = last_char.as_ref() {
                if *lc != '\\' {
                    return Ok((&yuml[idx + 1..], &yuml[..idx]));
                }
            } else {
                return Ok((&yuml[idx + 1..], &yuml[..idx]));
            }
        }

        last_char = Some(c)
    }

    Err(nom::Err::Error(nom::error::Error::new(
        yuml,
        nom::error::ErrorKind::RegexpFind,
    )))
}
#[cfg(test)]
mod tests {
    use super::*;

    fn parse(yuml: &str) -> DotFile {
        if let (rest, ParsedYuml::Activity(dot_file)) = parse_yuml(yuml).expect("invalid file") {
            assert!(rest.is_empty());
            println!("{dot_file}");
            dot_file
        } else {
            panic!("Invalid file");
        }
    }

    const HEADER: &str = "// {type:activity}\n";
    fn insert_header(yuml: &str) -> String {
        format!("{HEADER}{yuml}")
    }

    fn contains_all(parts: &[&str], full: &str) -> bool {
        for part in parts {
            if !full.contains(part) {
                return false;
            }
        }

        true
    }

    fn validate(yuml: &str, parts: &[&str]) {
        let yuml = insert_header(yuml);
        let result = parse(&yuml).to_string();
        assert!(contains_all(parts, &result));
    }

    #[test]
    fn parse_empty_activity() {
        const YUML: &str = r#"()"#;
        const A1: &str = r#"A1 [shape="rectangle" , margin="0.20,0.05" , label="" , style="rounded" , arrowtail="none" , arrowhead="none" , height=0.5 , fontsize=10 , ]"#;
        validate(YUML, &[A1]);
    }

    #[test]
    fn parse_single_activity() {
        const YUML: &str = "(Hello)";
        const A1: &str = r#"A1 [shape="rectangle" , margin="0.20,0.05" , label="Hello" , style="rounded" , arrowtail="none" , arrowhead="none" , height=0.5 , fontsize=10 , ]"#;
        validate(YUML, &[A1]);
    }

    #[test]
    fn parse_single_decision() {
        const YUML: &str = "<a>";
        const A1: &str = r#"A1 [shape="diamond" , label="a" , style="" , arrowtail="none" , arrowhead="none" , height=0.5 , width=0.5 , fontsize=0 , ]"#;
        validate(YUML, &[A1]);
    }

    #[test]
    fn parse_single_connector() {
        const YUML: &str = "|a|";
        const A1: &str = r#"A1 [shape="record" , label="" , style="filled" , arrowtail="none" , arrowhead="none" , height=0.05 , width=0.5 , fontsize=1 , penwidth=4 , ]"#;
        validate(YUML, &[A1]);
    }

    #[test]
    fn parse_single_note_with_attr() {
        const YUML: &str = "(note:Hello{bg:cornsilk})";
        const A1: &str = r#"A1 [shape="note" , margin="0.20,0.05" , label="Hello" , style="filled" , fillcolor="cornsilk" , arrowtail="none" , arrowhead="none" , height=0.5 , fontsize=10 , ]"#;
        validate(YUML, &[A1]);
    }

    #[test]
    fn parse_single_note() {
        const YUML: &str = "(note:Hello)";
        const A1: &str = r#"A1 [shape="note" , margin="0.20,0.05" , label="Hello" , style="" , arrowtail="none" , arrowhead="none" , height=0.5 , fontsize=10 , ]"#;
        validate(YUML, &[A1]);
    }

    #[test]
    fn parse_single_note_with_escaped_chars() {
        const YUML: &str = r#"(note: V1 \(vdest\): 99999{bg:cornsilk})"#;
        const A1: &str = r#"A1 [shape="note" , margin="0.20,0.05" , label=" V1 \(vdest\): 99999" , style="filled" , fillcolor="cornsilk" , arrowtail="none" , arrowhead="none" , height=0.5 , fontsize=10 ,"#;
        validate(YUML, &[A1]);
    }

    #[test]
    fn parse_single_connection() {
        const YUML: &str = "(a)-(b)";
        const A1: &str = r#"A1 [shape="rectangle" , margin="0.20,0.05" , label="a" , style="rounded" , arrowtail="none" , arrowhead="none" , height=0.5 , fontsize=10 , ]"#;
        const A2: &str = r#"A2 [shape="rectangle" , margin="0.20,0.05" , label="b" , style="rounded" , arrowtail="none" , arrowhead="none" , height=0.5 , fontsize=10 , ]"#;
        const CON: &str = r#"A1 -> A2 [shape="edge" , label="" , style="solid" , dir="both" , arrowtail="none" , arrowhead="none" , labeldistance=1 , fontsize=10 , ]"#;
        validate(YUML, &[A1, A2, CON]);
    }

    #[test]
    fn parse_double_connection() {
        const YUML: &str = "(a)-(b)-(c)";
        const A1: &str = r#"A1 [shape="rectangle" , margin="0.20,0.05" , label="a" , style="rounded" , arrowtail="none" , arrowhead="none" , height=0.5 , fontsize=10 , ]"#;
        const A2: &str = r#"A2 [shape="rectangle" , margin="0.20,0.05" , label="b" , style="rounded" , arrowtail="none" , arrowhead="none" , height=0.5 , fontsize=10 , ]"#;
        const A3: &str = r#"A3 [shape="rectangle" , margin="0.20,0.05" , label="c" , style="rounded" , arrowtail="none" , arrowhead="none" , height=0.5 , fontsize=10 , ]"#;
        const CON: &str = r#"A1 -> A2 [shape="edge" , label="" , style="solid" , dir="both" , arrowtail="none" , arrowhead="none" , labeldistance=1 , fontsize=10 , ]"#;
        const CON2: &str = r#"A2 -> A3 [shape="edge" , label="" , style="solid" , dir="both" , arrowtail="none" , arrowhead="none" , labeldistance=1 , fontsize=10 , ]"#;
        validate(YUML, &[A1, A2, A3, CON, CON2]);
    }

    #[test]
    fn parse_connection_with_note() {
        const YUML: &str = "(a)-(note:Hello)-(b)";
        const A1: &str = r#"A1 [shape="rectangle" , margin="0.20,0.05" , label="a" , style="rounded" , arrowtail="none" , arrowhead="none" , height=0.5 , fontsize=10 , ]"#;
        const A2: &str = r#"A2 [shape="note" , margin="0.20,0.05" , label="Hello" , style="" , arrowtail="none" , arrowhead="none" , height=0.5 , fontsize=10 , ]"#;
        const A3: &str = r#"A3 [shape="rectangle" , margin="0.20,0.05" , label="b" , style="rounded" , arrowtail="none" , arrowhead="none" , height=0.5 , fontsize=10 , ]"#;
        const CON: &str = r#"A1 -> A2 [shape="edge" , label="" , style="solid" , dir="both" , arrowtail="none" , arrowhead="none" , labeldistance=1 , fontsize=10 , ]"#;
        const CON2: &str = r#"A2 -> A3 [shape="edge" , label="" , style="solid" , dir="both" , arrowtail="none" , arrowhead="none" , labeldistance=1 , fontsize=10 , ]"#;
        validate(YUML, &[A1, A2, A3, CON, CON2]);
    }

    #[test]
    fn parse_single_arrow_connection() {
        const YUML: &str = "(a)->(b)";
        const A1: &str = r#"A1 [shape="rectangle" , margin="0.20,0.05" , label="a" , style="rounded" , arrowtail="none" , arrowhead="none" , height=0.5 , fontsize=10 , ]"#;
        const A2: &str = r#"A2 [shape="rectangle" , margin="0.20,0.05" , label="b" , style="rounded" , arrowtail="none" , arrowhead="none" , height=0.5 , fontsize=10 , ]"#;
        const CON: &str = r#"A1 -> A2 [shape="edge" , label="" , style="solid" , dir="both" , arrowtail="none" , arrowhead="vee" , labeldistance=1 , fontsize=10 , ]"#;
        validate(YUML, &[A1, A2, CON]);
    }

    #[test]
    fn test_parse_activity() {
        let yuml = include_str!("../../test/activity.yuml");
        parse(yuml);
    }

    #[test]
    fn test_parse_activity_2() {
        let yuml = include_str!("../../test/activity_2.yuml");
        parse(yuml);
    }

    #[test]
    fn test_parse_activity_w_note() {
        let yuml = include_str!("../../test/activity_w_note.yuml");
        parse(yuml);
    }

    #[test]
    fn test_parse_big_activity() {
        let yuml = include_str!("../../test/big_activity.yuml");
        parse(yuml);
    }
}
