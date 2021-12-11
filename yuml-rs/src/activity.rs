//! Syntax as specified in yuml.me
//!
//! Start              (start)
//! End                (end)
//! Activity           (Find Products)
//! Flow               (start)->(Find Products)
//! Multiple Assoc.    (start)->(Find Products)->(end)
//! Decisions          (start)-><d1>
//! Decisions w/Label  (start)-><d1>logged in->(Show Dashboard), <d1>not logged in->(Show Login Page)
//! Parallel           (Action1)->|a|,(Action 2)->|a|
//! Note               (Action1)-(note: A note message here)
//! Comment            // Comments

use crate::diagram::Diagram;
use crate::error::{YumlError, YumlResult};
use crate::model::{
    Arrow, Directions, Dot, DotElement, DotShape, EdgeProps, Options, Style, YumlExpression, YumlProps,
};
use crate::utils::{
    add_bar_facet, escape_label, extract_bg_from_regex, record_name, serialize_dot_elements, split_yuml_expr, EMPTY,
};
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::fmt::Write;

lazy_static! {
    static ref R_ACTIVITY: Regex = Regex::new(r"(?m)^\(.*\)$").unwrap();
    static ref R_DECISION: Regex = Regex::new(r"(?m)^<.*>$").unwrap();
    static ref R_BAR: Regex = Regex::new(r"(?m)^\|.*\|$").unwrap();
    static ref R_ARROW: Regex = Regex::new(r"(?m).*->$").unwrap();
    static ref R_BG_PARTS: Regex = Regex::new(r"(?m)^(.*)\{ *bg *: *([a-zA-Z]+\d*|#[0-9a-fA-F]{6}) *}$").unwrap();
    static ref R_LABEL: Regex = Regex::new(r"(?m)^<.+>(|<.+>)*$").unwrap();
}

pub struct Activity {}

impl Diagram for Activity {
    fn compose_dot_expr(&self, lines: &[&str], options: &Options) -> YumlResult<String> {
        let mut uids: HashMap<String, String> = HashMap::new();
        let mut len = 0;
        let mut elements: Vec<DotElement> = vec![];

        let expressions: Vec<Vec<YumlExpression>> =
            lines.iter().map(|line| self.parse_yuml_expr(line)).try_collect()?;

        for expression in expressions {
            for elem in &expression {
                let label = &elem.label;
                let uid_label = record_name(label).to_string();

                match &elem.props {
                    YumlProps::Diamond => {
                        if uids.contains_key(&uid_label) {
                            continue;
                        }

                        len += 1;
                        let uid = format!("A{}", len);
                        uids.insert(uid_label, uid.clone());

                        let node = Dot {
                            shape: DotShape::Diamond,
                            height: Some(0.5),
                            width: Some(0.5),
                            margin: Some("0,0".to_string()),
                            label: None,
                            fontsize: Some(0),
                            style: vec![],
                            fillcolor: None,
                            fontcolor: None,
                            penwidth: None,
                            dir: None,
                            arrowtail: None,
                            arrowhead: None,
                            taillabel: None,
                            headlabel: None,
                            labeldistance: None,
                        };

                        elements.push(DotElement::new(&uid, node));
                    }
                    YumlProps::MRecord => {
                        if uids.contains_key(&uid_label) {
                            continue;
                        }

                        len += 1;
                        let uid = format!("A{}", len);
                        uids.insert(uid_label, uid.clone());

                        let node = Dot {
                            shape: DotShape::Record,
                            height: Some(if options.dir == Directions::TopDown { 0.05 } else { 0.5 }),
                            width: Some(if options.dir == Directions::TopDown { 0.5 } else { 0.05 }),
                            margin: Some("0,0".to_string()),
                            style: vec![Style::Filled],
                            fillcolor: None,
                            label: None,
                            fontsize: Some(1),
                            penwidth: Some(4),
                            dir: None,
                            arrowtail: None,
                            arrowhead: None,
                            taillabel: None,
                            headlabel: None,
                            fontcolor: None,
                            labeldistance: None,
                        };

                        elements.push(DotElement::new(&uid, node));
                    }
                    YumlProps::Edge(_) | YumlProps::Signal(_) => {
                        // ignore for now
                    }
                    YumlProps::NoteOrRecord(is_note, fillcolor, fontcolor) => {
                        if uids.contains_key(&uid_label) {
                            continue;
                        }

                        len += 1;
                        let uid = format!("A{}", len);
                        uids.insert(uid_label, uid.clone());

                        let node = if !*is_note && (label == "start" || label == "end") {
                            Dot {
                                shape: if label == "start" {
                                    DotShape::Circle
                                } else {
                                    DotShape::DoubleCircle
                                },
                                height: Some(0.3),
                                width: Some(0.3),
                                margin: Some("0,0".to_string()),
                                label: None,
                                fontsize: None,
                                style: vec![],
                                fillcolor: None,
                                fontcolor: None,
                                penwidth: None,
                                dir: None,
                                arrowtail: None,
                                arrowhead: None,
                                taillabel: None,
                                headlabel: None,
                                labeldistance: None,
                            }
                        } else {
                            let mut node = Dot {
                                shape: YumlProps::note_or_record_shape(*is_note),
                                height: Some(0.5),
                                fontsize: Some(10),
                                margin: Some("0.20,0.05".to_string()),
                                label: Some(escape_label(label)),
                                style: vec![Style::Rounded],
                                fillcolor: None,
                                width: None,
                                penwidth: None,
                                dir: None,
                                arrowtail: None,
                                arrowhead: None,
                                taillabel: None,
                                headlabel: None,
                                fontcolor: None,
                                labeldistance: None,
                            };

                            if !fillcolor.is_empty() {
                                node.style.push(Style::Filled);
                                node.fillcolor = Some(fillcolor.clone());
                            }

                            if !fontcolor.is_empty() {
                                node.fontcolor = Some(fontcolor.clone());
                            }

                            node
                        };

                        elements.push(DotElement::new(&uid, node));
                    }
                }
            }

            for range in expression.windows(3) {
                let previous_is_edge = matches!(range.get(0).map(|c| &c.props), Some(YumlProps::Edge(_)));
                let next_is_edge = matches!(range.get(2).map(|c| &c.props), Some(YumlProps::Edge(_)));

                if !previous_is_edge && !next_is_edge {
                    if let Some(YumlProps::Edge(props)) = range.get(1).map(|c| &c.props) {
                        let label = &range.get(1).unwrap().label;
                        let previous_is_note =
                            if let Some(YumlProps::NoteOrRecord(is_note, _, _)) = range.get(0).map(|c| &c.props) {
                                *is_note
                            } else {
                                false
                            };

                        let next_is_note =
                            if let Some(YumlProps::NoteOrRecord(is_note, _, _)) = range.get(2).map(|c| &c.props) {
                                *is_note
                            } else {
                                false
                            };

                        let style = if previous_is_note || next_is_note {
                            Style::Dashed
                        } else {
                            props.style.clone()
                        };

                        let mut edge = Dot {
                            shape: DotShape::Edge,
                            height: None,
                            width: None,
                            dir: Some("both".to_string()),
                            style: vec![style],
                            fillcolor: None,
                            fontcolor: None,
                            arrowtail: props.arrowtail.clone(),
                            arrowhead: props.arrowhead.clone(),
                            taillabel: None,
                            headlabel: None,
                            labeldistance: Some(1),
                            fontsize: Some(10),
                            label: None,
                            margin: None,
                            penwidth: None,
                        };

                        if !label.is_empty() {
                            edge.label = Some(label.to_string());
                        }

                        let uid1 = uids
                            .get(&range.get(0).map(|c| &c.label).unwrap_or(&EMPTY).to_string())
                            .unwrap_or(&EMPTY)
                            .to_string();

                        let mut uid2 = uids
                            .get(&range.get(2).map(|c| &c.label).unwrap_or(&EMPTY).to_string())
                            .unwrap_or(&EMPTY)
                            .to_string();

                        if range.get(2).map(|c| c.props == YumlProps::MRecord).unwrap_or(false) {
                            // note that the add_bar_facet call modifies elements!
                            if let Some(facet) = add_bar_facet(&mut elements, &uid2) {
                                uid2 = format!("{}:{}:{}", uid2, facet, options.dir.head_port());
                            }
                        }

                        elements.push(DotElement::new_edge(&uid1, &uid2, edge))
                    }
                }
            }
        }

        let mut dot = format!("    ranksep = {}\n", 0.5);
        dot.write_fmt(format_args!("    rankdir = {}\n", options.dir))?;
        dot.write_str(&serialize_dot_elements(elements)?)?;
        dot.write_str("}\n")?;

        Ok(dot)
    }

    fn parse_yuml_expr(&self, spec_line: &str) -> YumlResult<Vec<YumlExpression>> {
        let parts = split_yuml_expr(spec_line, "(<|", None)?;
        let expressions = parts.into_iter().filter_map(|part| {
            if part.is_empty() {
                return None;
            }

            if let Some(note) = extract_bg_from_regex(&part, &R_ACTIVITY) {
                return Some(Ok(note));
            }

            if let Some(decision) = R_DECISION.find(&part) {
                let a_str = decision.as_str();
                let part = &a_str[1..a_str.len() - 1];
                return Some(Ok(YumlExpression {
                    label: part.to_string(),
                    props: YumlProps::Diamond,
                }));
            }

            if let Some(bar) = R_BAR.find(&part) {
                let a_str = bar.as_str();
                let part = &a_str[1..a_str.len() - 1];
                return Some(Ok(YumlExpression {
                    label: part.to_string(),
                    props: YumlProps::MRecord,
                }));
            }

            if let Some(arrow) = R_ARROW.find(&part) {
                let a_str = arrow.as_str();
                let part = &a_str[..a_str.len() - 2].trim();
                return Some(Ok(YumlExpression {
                    label: part.to_string(),
                    props: YumlProps::Edge(EdgeProps {
                        arrowtail: None,
                        arrowhead: Some(Arrow::Vee),
                        taillabel: None,
                        headlabel: None,
                        style: Style::Solid,
                    }),
                }));
            }

            if part == "-" {
                return Some(Ok(YumlExpression {
                    label: String::new(),
                    props: YumlProps::Edge(EdgeProps {
                        arrowtail: None,
                        arrowhead: None,
                        taillabel: None,
                        headlabel: None,
                        style: Style::Solid,
                    }),
                }));
            }

            Some(Err(YumlError::Expression))
        });

        expressions.try_collect()
    }
}

#[test]
fn test_yuml_expression() {
    let activity = Activity {};
    let expression = activity
        .parse_yuml_expr("<a>[kettle empty]->(Fill Kettle)->|b|")
        .expect("can not parse");
    assert_eq!(expression.len(), 5);
    let str_ex = expression.iter().map(|expr| expr.to_string()).join(" | ");
    assert_eq!(
        str_ex,
        "a: diamond | [kettle empty]: edge | Fill Kettle: record | : edge | b: mrecord"
    );
}
