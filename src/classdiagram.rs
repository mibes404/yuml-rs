use crate::diagram::Diagram;
use crate::error::{YumlError, YumlResult};
use crate::model::{Arrow, EdgeProps, Options, Style, YumlExpression, YumlProps};
use crate::utils::{extract_bg_and_note, split_yuml_expr};
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref R_CLASS_BOX: Regex = Regex::new(r"^\[.*]$").unwrap();
}

pub struct ClassDiagram {}

impl Diagram for ClassDiagram {
    fn compose_dot_expr(&self, lines: &[&str], options: &Options) -> YumlResult<String> {
        unimplemented!()
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

        let parts = split_yuml_expr(spec_line, "(|", None)?;
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
                    id: "".to_string(),
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
                    id: "".to_string(),
                    props: YumlProps::Edge(EdgeProps {
                        arrowtail: l_arrow,
                        arrowhead: r_arrow,
                        taillabel: Some(l_text),
                        headlabel: Some(r_text),
                        style,
                    }),
                }));
            }

            Some(Err(YumlError::Expression))
        });

        expressions.try_collect()
    }
}
