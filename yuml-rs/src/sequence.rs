use crate::diagram::Diagram;
use crate::error::{YumlError, YumlResult};
use crate::model::{Actor, Arrow, Options, Signal, SignalProps, SignalType, Style, YumlExpression, YumlProps};
use crate::utils::{extract_bg_from_regex, format_label, record_name, split_yuml_expr};
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;

pub struct Sequence {}

lazy_static! {
    static ref R_OBJECT: Regex = Regex::new(r"^\[.*]$").unwrap();
    static ref R_MESSAGE: Regex = Regex::new(r"[.|>]{0,1}>[(|)]{0,1}$").unwrap();
}

fn is_note(props: &YumlProps) -> bool {
    if let YumlProps::NoteOrRecord(is_note, _, _) = props {
        *is_note
    } else {
        false
    }
}

impl Diagram for Sequence {
    fn compose_dot_expr(&self, lines: &[&str], _options: &Options) -> YumlResult<String> {
        let mut uids: HashMap<String, Actor> = HashMap::new();
        let svg = String::new();
        let mut signals: Vec<Signal> = vec![];

        let expressions: Vec<Vec<YumlExpression>> =
            lines.iter().map(|line| self.parse_yuml_expr(line)).try_collect()?;

        for expression in expressions {
            for elem in &expression {
                if let YumlProps::NoteOrRecord(is_note, _fillcolor, _fontcolor) = &elem.props {
                    if !is_note {
                        // object
                        let label = &elem.label;
                        let uid_label = record_name(label).to_string();
                        if uids.contains_key(&uid_label) {
                            continue;
                        }

                        let label = format_label(label, 20, true);
                        let actor = Actor {
                            actor_type: "object".to_string(),
                            name: uid_label.clone(),
                            label,
                            index: uids.len(),
                        };

                        uids.insert(uid_label, actor);
                    }
                }
            }

            if expression.len() == 3 {
                let previous = &expression[0];
                let elem = &expression[1];
                let next = &expression[2];

                if let YumlProps::Signal(signal) = &elem.props {
                    if is_note(&previous.props) && is_note(&next.props) {
                        // todo:
                        let message = &signal.prefix;
                        let style = &signal.style;
                        let actor_a = uids.get(record_name(&previous.label)).map(|a| (*a).clone());
                        let actor_b = uids.get(record_name(&next.label)).map(|b| (*b).clone());
                        // let signal: Dot;

                        let signal = match style {
                            Style::Solid => Some(Signal {
                                signal_type: SignalType::Signal,
                                actor_a,
                                actor_b,
                                line_type: Some(Style::Dashed),
                                arrow_type: Some(Arrow::Filled),
                                message: message.clone(),
                            }),
                            Style::Dashed => Some(Signal {
                                signal_type: SignalType::Signal,
                                actor_a,
                                actor_b,
                                line_type: Some(Style::Solid),
                                arrow_type: Some(Arrow::Filled),
                                message: message.clone(),
                            }),
                            Style::Async => Some(Signal {
                                signal_type: SignalType::Signal,
                                actor_a,
                                actor_b,
                                line_type: Some(Style::Solid),
                                arrow_type: Some(Arrow::Open),
                                message: message.clone(),
                            }),
                            _ => None,
                        };

                        if let Some(signal) = signal {
                            signals.push(signal);
                        }
                    }
                }
            }
        }

        Ok(svg)
    }

    fn parse_yuml_expr(&self, spec_line: &str) -> YumlResult<Vec<YumlExpression>> {
        let parts = split_yuml_expr(spec_line, "[", None)?;
        let expressions = parts.into_iter().filter_map(|part| {
            if part.is_empty() {
                return None;
            }

            if let Some(note) = extract_bg_from_regex(&part, &R_OBJECT) {
                return Some(Ok(note));
            }

            // note connector
            if part == "-" {
                return Some(Ok(YumlExpression {
                    label: "".to_string(),
                    props: YumlProps::Signal(SignalProps {
                        prefix: None,
                        suffix: None,
                        style: Style::Dashed,
                    }),
                }));
            }

            // message
            if part.contains('>') {
                let mut part: &str = &part;
                let style = if part.contains(".>") {
                    Style::Dashed
                } else if part.contains(">>") {
                    Style::Async
                } else {
                    Style::Solid
                };

                let prefix = if part.starts_with('(') || part.starts_with(')') {
                    let prefix = &part[0..1];
                    part = &part[1..];
                    prefix
                } else {
                    ""
                };

                let message = if let Some(msg_match) = R_MESSAGE.find(part) {
                    let pos = msg_match.start();
                    let message = &part[0..pos];
                    part = &part[pos..];
                    message
                } else {
                    ""
                };

                let suffix = if part.ends_with('(') || part.ends_with(')') {
                    &part[part.len() - 1..]
                } else {
                    ""
                };

                return Some(Ok(YumlExpression {
                    label: message.to_string(),
                    props: YumlProps::Signal(SignalProps {
                        prefix: Some(prefix.to_string()),
                        suffix: Some(suffix.to_string()),
                        style,
                    }),
                }));
            }

            Some(Err(YumlError::Expression))
        });

        expressions.try_collect()
    }
}
