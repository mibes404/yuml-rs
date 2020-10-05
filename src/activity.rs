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
use crate::model::{Directions, Edge, Node, NodeOrEdge, Options, YumlExpression, YumlProps};
use crate::EMPTY;
use crate::{add_bar_facet, escape_label, extract_bg_and_note, record_name, serialize_dot_elements, split_yuml_expr};
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
        let mut elements: Vec<NodeOrEdge> = vec![];

        let expressions: Vec<Vec<YumlExpression>> =
            lines.iter().map(|line| self.parse_yuml_expr(line)).try_collect()?;

        for expression in expressions {
            for elem in &expression {
                let label = &elem.id;
                match &elem.props {
                    YumlProps::Diamond => {
                        if uids.contains_key(label) {
                            continue;
                        }

                        len += 1;
                        let uid = format!("A{}", len);
                        uids.insert(record_name(label).to_string(), uid.clone());

                        let node = Node {
                            shape: "diamond".to_string(),
                            height: 0.5,
                            width: 0.5,
                            margin: "0,0".to_string(),
                            label: None,
                            fontsize: Some(0),
                            style: "".to_string(),
                            fillcolor: None,
                            fontcolor: None,
                            penwidth: None,
                        };

                        elements.push(NodeOrEdge::Node(uid, node));
                    }
                    YumlProps::MRecord => {
                        if uids.contains_key(label) {
                            continue;
                        }

                        len += 1;
                        let uid = format!("A{}", len);
                        uids.insert(record_name(&label).to_string(), uid.clone());

                        let node = Node {
                            shape: "record".to_string(),
                            height: if options.dir == Directions::TopDown { 0.05 } else { 0.5 },
                            width: if options.dir == Directions::TopDown { 0.5 } else { 0.05 },
                            margin: "0,0".to_string(),
                            style: "filled".to_string(),
                            fillcolor: None,
                            label: None,
                            fontsize: Some(1),
                            penwidth: Some(4),
                            fontcolor: None,
                        };

                        elements.push(NodeOrEdge::Node(uid, node));
                    }
                    YumlProps::Edge(_, _, _) => {
                        // ignore for now
                    }
                    YumlProps::NoteOrRecord(is_note, fillcolor, fontcolor) => {
                        if uids.contains_key(label) {
                            continue;
                        }

                        len += 1;
                        let uid = format!("A{}", len);
                        uids.insert(record_name(label).to_string(), uid.clone());

                        let node = if !is_note && (label == "start" || label == "end") {
                            Node {
                                shape: if label == "start" {
                                    "circle".to_string()
                                } else {
                                    "doublecircle".to_string()
                                },
                                height: 0.3,
                                width: 0.3,
                                margin: "0,0".to_string(),
                                label: None,
                                fontsize: None,
                                style: "".to_string(),
                                fillcolor: None,
                                fontcolor: None,
                                penwidth: None,
                            }
                        } else {
                            let mut node = Node {
                                shape: if *is_note {
                                    "note".to_string()
                                } else {
                                    "record".to_string()
                                },
                                height: 0.5,
                                fontsize: Some(10),
                                margin: "0.20,0.05".to_string(),
                                label: Some(escape_label(&label)),
                                style: "rounded".to_string(),
                                fillcolor: None,
                                width: 0.0,
                                penwidth: None,
                                fontcolor: None,
                            };

                            if !fillcolor.is_empty() {
                                node.style += ",filled";
                                node.fillcolor = Some(fillcolor.clone());
                            }

                            if !fontcolor.is_empty() {
                                node.fontcolor = Some(fontcolor.clone());
                            }

                            node
                        };

                        elements.push(NodeOrEdge::Node(uid, node));
                    }
                }
            }

            for range in expression.windows(3) {
                let previous_is_edge = if let Some(YumlProps::Edge(_, _, _)) = range.get(0).map(|c| &c.props) {
                    true
                } else {
                    false
                };

                let next_is_edge = if let Some(YumlProps::Edge(_, _, _)) = range.get(2).map(|c| &c.props) {
                    true
                } else {
                    false
                };

                if !previous_is_edge && !next_is_edge {
                    if let Some(YumlProps::Edge(arrowtail, arrowhead, style)) = range.get(1).map(|c| &c.props) {
                        let label = &range.get(1).unwrap().id;
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
                            "dashed"
                        } else {
                            &style
                        };

                        let mut edge = Edge {
                            shape: "edge".to_string(),
                            dir: "both".to_string(),
                            style: style.to_string(),
                            arrowtail: arrowtail.to_string(),
                            arrowhead: arrowhead.to_string(),
                            labeldistance: 1,
                            fontsize: 10,
                            label: None,
                        };

                        if !label.is_empty() {
                            edge.label = Some(label.to_string());
                        }

                        let uid1 = uids
                            .get(&range.get(0).map(|c| &c.id).unwrap_or(&EMPTY).to_string())
                            .unwrap_or(&EMPTY)
                            .to_string();

                        let mut uid2 = uids
                            .get(&range.get(2).map(|c| &c.id).unwrap_or(&EMPTY).to_string())
                            .unwrap_or(&EMPTY)
                            .to_string();

                        if range.get(2).map(|c| c.props == YumlProps::MRecord).unwrap_or(false) {
                            // note that the add_bar_facet call modifies elements!
                            if let Some(facet) = add_bar_facet(&mut elements, &uid2) {
                                uid2 = format!("{}:{}:{}", uid2, facet, options.dir.head_port());
                            }
                        }

                        elements.push(NodeOrEdge::Edge(uid1, uid2, edge))
                    }
                }
            }
        }

        let mut dot = format!("    ranksep = {}\r\n", 0.5);
        dot.write_fmt(format_args!("    rankdir = {}\r\n", options.dir))?;
        dot.write_str(&serialize_dot_elements(elements)?)?;
        dot.write_str("}\r\n")?;

        Ok(dot)
    }

    fn parse_yuml_expr(&self, spec_line: &str) -> YumlResult<Vec<YumlExpression>> {
        let parts = split_yuml_expr(spec_line, "(<|", None)?;
        let expressions = parts.into_iter().filter_map(|part| {
            if part.is_empty() {
                return None;
            }

            if let Some(actvity) = R_ACTIVITY.find(&part) {
                let a_str = actvity.as_str();
                let part = &a_str[1..a_str.len() - 1];
                let ret = extract_bg_and_note(part, true);
                return Some(Ok(YumlExpression {
                    id: ret.part,
                    props: YumlProps::NoteOrRecord(
                        ret.is_note,
                        ret.bg.unwrap_or_default(),
                        ret.font_color.unwrap_or_default(),
                    ),
                }));
            }

            if let Some(decision) = R_DECISION.find(&part) {
                let a_str = decision.as_str();
                let part = &a_str[1..a_str.len() - 1];
                return Some(Ok(YumlExpression {
                    id: part.to_string(),
                    props: YumlProps::Diamond,
                }));
            }

            if let Some(bar) = R_BAR.find(&part) {
                let a_str = bar.as_str();
                let part = &a_str[1..a_str.len() - 1];
                return Some(Ok(YumlExpression {
                    id: part.to_string(),
                    props: YumlProps::MRecord,
                }));
            }

            if let Some(arrow) = R_ARROW.find(&part) {
                let a_str = arrow.as_str();
                let part = &a_str[..a_str.len() - 2].trim();
                return Some(Ok(YumlExpression {
                    id: part.to_string(),
                    props: YumlProps::Edge("none".to_string(), "vee".to_string(), "solid".to_string()),
                }));
            }

            if part == "-" {
                return Some(Ok(YumlExpression {
                    id: String::new(),
                    props: YumlProps::Edge("none".to_string(), "none".to_string(), "solid".to_string()),
                }));
            }

            Some(Err(YumlError::Expression))
        });

        expressions.try_collect()
    }
}
