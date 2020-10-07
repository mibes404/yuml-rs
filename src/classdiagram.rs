use crate::diagram::Diagram;
use crate::error::{YumlError, YumlResult};
use crate::model::{Arrow, Dot, DotShape, EdgeProps, Options, Style, YumlExpression, YumlProps};
use crate::utils::EMPTY;
use crate::utils::{extract_bg_and_note, format_label, record_name, serialize_dot, split_yuml_expr};
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::fmt::Write;

lazy_static! {
    static ref R_CLASS_BOX: Regex = Regex::new(r"^\[.*]$").unwrap();
}

pub struct ClassDiagram {}

impl Diagram for ClassDiagram {
    fn compose_dot_expr(&self, lines: &[&str], options: &Options) -> YumlResult<String> {
        let mut uids: HashMap<String, String> = HashMap::new();
        let mut len = 0;

        let mut dot = format!("    ranksep = {}\n", 0.7);
        dot.write_fmt(format_args!("    rankdir = {}\n", options.dir))?;

        let expressions: Vec<Vec<YumlExpression>> =
            lines.iter().map(|line| self.parse_yuml_expr(line)).try_collect()?;

        for expression in expressions {
            for elem in &expression {
                if let YumlProps::NoteOrRecord(is_note, fillcolor, fontcolor) = &elem.props {
                    let shape = YumlProps::note_or_record_shape(*is_note);
                    let label = &elem.label;

                    if uids.contains_key(label) {
                        continue;
                    }

                    len += 1;
                    let uid = format!("A{}", len);
                    uids.insert(record_name(label).to_string(), uid.clone());

                    let label = format_label(label, 20, true);
                    let mut node = Dot {
                        shape,
                        height: Some(0.5),
                        width: None,
                        margin: Some("0.20,0.05".to_string()),
                        label: Some(label),
                        fontsize: Some(10),
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

                    if !fillcolor.is_empty() {
                        node.style = vec![Style::Filled];
                        node.fillcolor = Some(fillcolor.clone());
                    }

                    if !fontcolor.is_empty() {
                        node.fontcolor = Some(fontcolor.clone());
                    }

                    dot.write_fmt(format_args!("    {} {}\n", uid, serialize_dot(node)?))?
                }
            }

            if expression.len() == 3 {
                let elem = expression.get(1).unwrap();
                if let YumlProps::Edge(props) = &elem.props {
                    let previous = expression.get(0).unwrap();
                    let next = expression.get(2).unwrap();

                    let has_note = if let YumlProps::NoteOrRecord(is_note, _, _) = previous.props {
                        is_note
                    } else {
                        false
                    } || if let YumlProps::NoteOrRecord(is_note, _, _) = next.props {
                        is_note
                    } else {
                        false
                    };

                    let style = if has_note { Style::Dashed } else { props.style.clone() };

                    let edge = Dot {
                        shape: DotShape::Edge,
                        height: None,
                        width: None,
                        dir: Some("both".to_string()),
                        style: vec![style],
                        fillcolor: None,
                        fontcolor: None,
                        arrowtail: props.arrowtail.clone(),
                        arrowhead: props.arrowhead.clone(),
                        taillabel: props.taillabel.clone(),
                        headlabel: props.headlabel.clone(),
                        labeldistance: Some(2),
                        fontsize: Some(10),
                        label: None,
                        margin: None,
                        penwidth: None,
                    };

                    let uid_previous = uids.get(record_name(&previous.label)).unwrap_or(&EMPTY).to_string();
                    let uid_next = uids.get(record_name(&next.label)).unwrap_or(&EMPTY).to_string();

                    if has_note {
                        dot.write_fmt(format_args!(
                            "    {{ rank=same; {} -> {} {};}}\n",
                            uid_previous,
                            uid_next,
                            serialize_dot(edge)?
                        ))?;
                    } else {
                        dot.write_fmt(format_args!(
                            "    {} -> {} {}\n",
                            uid_previous,
                            uid_next,
                            serialize_dot(edge)?
                        ))?;
                    }
                }
            }

            if expression.len() == 4 {
                let previous = expression.get(0).unwrap();
                let current = expression.get(1).unwrap();
                let next = expression.get(2).unwrap();
                let last = expression.get(3).unwrap();

                let pattern = format!(
                    "{},{},{},{}",
                    previous.to_string(),
                    current.to_string(),
                    next.to_string(),
                    expression.get(3).unwrap().to_string()
                );

                if pattern == "record,edge,record,record" {
                    println!("pattern not implemented: {}", pattern);

                    let junction = Dot {
                        shape: DotShape::Point,
                        style: vec![Style::Invis],
                        fillcolor: None,
                        fontcolor: None,
                        penwidth: None,
                        dir: None,
                        arrowtail: None,
                        arrowhead: None,
                        taillabel: None,
                        headlabel: None,
                        label: Some("".to_string()),
                        height: Some(0.01),
                        width: Some(0.01),
                        margin: None,
                        fontsize: None,
                        labeldistance: None,
                    };

                    let uid_previous = uids.get(record_name(&previous.label)).unwrap_or(&EMPTY).to_string();
                    let uid_next = uids.get(record_name(&next.label)).unwrap_or(&EMPTY).to_string();
                    let uid_last = uids.get(record_name(&last.label)).unwrap_or(&EMPTY).to_string();

                    let uid = format!("{}J{}", uid_previous, uid_next);
                    dot.write_fmt(format_args!("    {} {}\n", uid, serialize_dot(junction)?))?;

                    if let YumlProps::Edge(props) = &current.props {
                        let edge1 = Dot {
                            shape: DotShape::Edge,
                            height: None,
                            width: None,
                            margin: None,
                            dir: Some("both".to_string()),
                            style: vec![props.style.clone()],
                            fillcolor: None,
                            fontcolor: None,
                            arrowtail: props.arrowtail.clone(),
                            taillabel: props.taillabel.clone(),
                            arrowhead: None,
                            labeldistance: Some(2),
                            fontsize: Some(10),
                            label: None,
                            penwidth: None,
                            headlabel: None,
                        };

                        let edge2 = Dot {
                            shape: DotShape::Edge,
                            height: None,
                            width: None,
                            margin: None,
                            dir: Some("both".to_string()),
                            style: vec![props.style.clone()],
                            fillcolor: None,
                            fontcolor: None,
                            arrowtail: None,
                            arrowhead: props.arrowhead.clone(),
                            taillabel: None,
                            headlabel: props.headlabel.clone(),
                            labeldistance: Some(2),
                            fontsize: Some(10),
                            label: None,
                            penwidth: None,
                        };

                        let edge3 = Dot {
                            shape: DotShape::Edge,
                            height: None,
                            width: None,
                            margin: None,
                            label: None,
                            dir: Some("both".to_string()),
                            style: vec![Style::Dashed],
                            fillcolor: None,
                            fontcolor: None,
                            arrowtail: None,
                            arrowhead: Some(Arrow::Vee),
                            taillabel: None,
                            headlabel: None,
                            labeldistance: Some(2),
                            fontsize: None,
                            penwidth: None,
                        };

                        dot.write_fmt(format_args!(
                            "    {} -> {} {}\n",
                            uid_previous,
                            uid,
                            serialize_dot(edge1)?
                        ))?;
                        dot.write_fmt(format_args!("    {} -> {} {}\n", uid, uid_next, serialize_dot(edge2)?))?;
                        dot.write_fmt(format_args!(
                            "    {{ rank=same; {} -> {} {}; }}\n",
                            uid_last,
                            uid,
                            serialize_dot(edge3)?
                        ))?;
                    }

                    // edge1
                }
            }
        }

        dot.write_str("}\n")?;
        Ok(dot)
    }

    fn parse_yuml_expr(&self, spec_line: &str) -> YumlResult<Vec<YumlExpression>> {
        fn process_left(left: &str) -> (Option<Arrow>, String) {
            if left.starts_with("<>") {
                (Some(Arrow::ODiamond), left[2..].to_string())
            } else if left.starts_with("++") {
                (Some(Arrow::Diamond), left[2..].to_string())
            } else if left.starts_with('+') {
                (Some(Arrow::Diamond), left[1..].to_string())
            } else if left.starts_with('<') || left.starts_with('>') {
                (Some(Arrow::Vee), left[1..].to_string())
            } else if left.starts_with('^') {
                (Some(Arrow::Empty), left[1..].to_string())
            } else {
                (None, left.to_string())
            }
        }

        fn process_right(right: &str) -> (Option<Arrow>, String) {
            let len = right.len();
            if right.ends_with("<>") {
                (Some(Arrow::ODiamond), right[0..len - 1].to_string())
            } else if right.ends_with("++") {
                (Some(Arrow::Diamond), right[2..len].to_string())
            } else if right.ends_with('+') {
                (Some(Arrow::Diamond), right[1..len].to_string())
            } else if right.ends_with('>') {
                (Some(Arrow::Vee), right[1..len].to_string())
            } else if right.ends_with('^') {
                (Some(Arrow::Empty), right[1..len].to_string())
            } else {
                process_left(right)
            }
        }

        let parts = split_yuml_expr(spec_line, "[", None)?;
        let expressions = parts.into_iter().filter_map(|part| {
            if part.is_empty() {
                return None;
            }

            if let Some(class_box) = R_CLASS_BOX.find(&part) {
                let a_str = class_box.as_str();
                let part = &a_str[1..a_str.len() - 1];
                let ret = extract_bg_and_note(part, true);
                return Some(Ok(YumlExpression::from(ret)));
            }

            // inheritance
            if part == "^" {
                return Some(Ok(YumlExpression {
                    label: "empty".to_string(),
                    props: YumlProps::Edge(EdgeProps {
                        arrowtail: Some(Arrow::Empty),
                        arrowhead: None,
                        taillabel: None,
                        headlabel: None,
                        style: Style::Solid,
                    }),
                }));
            }

            // association
            if part.contains('-') {
                let style: Style;
                let tokens: Vec<&str>;
                if part.contains("-.-") {
                    style = Style::Dashed;
                    tokens = part.split("-.-").collect();
                } else {
                    style = Style::Solid;
                    tokens = part.split('-').collect();
                }

                if tokens.len() != 2 {
                    return Some(Err(YumlError::Expression));
                }

                let left = tokens.get(0).unwrap();
                let right = tokens.get(1).unwrap();
                let (l_arrow, l_text) = process_left(left);
                let (r_arrow, r_text) = process_right(right);

                return Some(Ok(YumlExpression {
                    label: "".to_string(),
                    props: YumlProps::Edge(EdgeProps {
                        arrowtail: l_arrow,
                        arrowhead: r_arrow,
                        taillabel: Some(l_text),
                        headlabel: Some(r_text),
                        style,
                    }),
                }));
            }

            println!("no_match: {}", part);
            Some(Err(YumlError::Expression))
        });

        expressions.try_collect()
    }
}
