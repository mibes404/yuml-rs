use crate::error::{OptionsError, YumlResult};
use crate::model::{BgAndNote, NodeOrEdge, Options};
use crate::rgb::COLOR_TABLE;
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::convert::TryInto;
use std::fmt::Write;
use std::str::FromStr;

lazy_static! {
    static ref R_KEY_VALUE: Regex =
        Regex::new(r"(?m)^//\s+\{\s*([\w]+)\s*:\s*([\w]+)\s*}$").unwrap(); // extracts directives as:  // {key:value}
    static ref R_BG_PARTS: Regex = Regex::new(r"(?m)^(.*)\{ *bg *: *([a-zA-Z]+\d*|#[0-9a-fA-F]{6}) *}$").unwrap();
    static ref R_LABEL: Regex = Regex::new(r"(?m)^<.+>(|<.+>)*$").unwrap();
    static ref ESCAPED_CHARS: HashMap<char, String> = build_escaped_chars();
    pub static ref EMPTY: String = String::new();
}

fn build_escaped_chars() -> HashMap<char, String> {
    let mut escaped_chars = HashMap::new();
    escaped_chars.insert('\n', "<BR/>".to_string());
    escaped_chars.insert('&', "&amp;".to_string());
    escaped_chars.insert('<', "&lt;".to_string());
    escaped_chars.insert('>', "&gt;".to_string());
    escaped_chars
}

pub fn process_directives(line: &str, options: &mut Options) -> YumlResult<()> {
    let mut matches = R_KEY_VALUE.captures_iter(line);
    if let Some(caps) = matches.next() {
        if caps.len() == 3 {
            let key = caps.get(1).unwrap().as_str();
            let value = caps.get(2).unwrap().as_str();
            match key {
                "type" => {
                    let chart_type = value.try_into()?;
                    options.chart_type = Some(chart_type);
                }
                "direction" => {
                    let direction = value.try_into()?;
                    options.dir = direction
                }
                "generate" => {
                    if let Ok(generate) = bool::from_str(value) {
                        options.generate = generate;
                    } else {
                        return Err(OptionsError::new(
                            "invalid value for 'generate'. Allowed values are: true, false <i>(default)</i>.",
                        )
                        .into());
                    }
                }
                _ => {
                    // unsupported
                }
            }
        }
    }

    Ok(())
}

pub fn build_dot_header(is_dark: bool) -> String {
    let colors = if is_dark {
        "color=white, fontcolor=white"
    } else {
        "color=black, fontcolor=black"
    };

    format!(
        r#"digraph G {{
  graph [ bgcolor=transparent, fontname=Helvetica ]
  node [ shape=none, margin=0, {colors}, fontname=Helvetica ]
  edge [ {colors}, fontname=Helvetica ]
"#,
        colors = colors
    )
}

pub fn record_name(label: &str) -> &str {
    label.split('|').next().map(|l| l.trim()).unwrap_or_default()
}

pub fn serialize_dot(node_or_edge: NodeOrEdge) -> YumlResult<String> {
    match node_or_edge {
        NodeOrEdge::Node(_, mut node) => {
            let label = node.label.clone().unwrap_or_default();
            if node.shape == "record" && !R_LABEL.is_match(&label) {
                // Graphviz documentation says (https://www.graphviz.org/doc/info/shapes.html):
                // The record-based shape has largely been superseded and greatly generalized by HTML-like labels.
                // That is, instead of using shape=record, one might consider using shape=none, margin=0 and an HTML-like label. [...]
                // Also note that there are problems using non-trivial edges (edges with ports or labels) between adjacent nodes
                // on the same rank if one or both nodes has a record shape.

                if label.contains('|') {
                    let mut result =
                        r#"[fontsize=10,label=<<TABLE BORDER="0" CELLBORDER="1" CELLSPACING="0" CELLPADDING="9" "#
                            .to_string();
                    if let Some(fillcolor) = &node.fillcolor {
                        result.write_fmt(format_args!(r#"BGCOLOR="{}" "#, fillcolor))?;
                    }
                    if let Some(fontcolor) = &node.fontcolor {
                        result.write_fmt(format_args!(r#"COLOR="{}" "#, fontcolor))?;
                    }

                    result.write_str(">")?;
                    result.write_str(
                        &label
                            .split('|')
                            .map(|t| {
                                let text = unescape_label(t);
                                let html_text: String = text
                                    .chars()
                                    .map(|c| ESCAPED_CHARS.get(&c).unwrap_or(&c.to_string()).to_string())
                                    .join("");
                                format!("<TR><TD>{}</TD></TR>", html_text)
                            })
                            .join(""),
                    )?;

                    result.write_str("</TABLE>>]")?;
                    return Ok(result);
                }

                // To avoid this issue, we can use a "rectangle" shape
                node.shape = "rectangle".to_string();
            }

            Ok(node.to_string())
        }

        NodeOrEdge::Edge(_, _, edge) => Ok(edge.to_string()),
    }
}

pub fn unescape_label(label: &str) -> String {
    label
        .replace(r"\\{", "{")
        .replace(r"\\}", "}")
        .replace(r"\\<", "<")
        .replace(r"\\>", ">")
}

pub fn serialize_dot_elements(mut elements: Vec<NodeOrEdge>) -> YumlResult<String> {
    let mut dot = String::new();
    while let Some(elem) = elements.pop() {
        match elem {
            NodeOrEdge::Node(uid, node) => {
                let _ = dot.write_fmt(format_args!(
                    "    {} {}\r\n",
                    uid.clone(),
                    serialize_dot(NodeOrEdge::Node(uid, node))?
                ))?;
            }
            NodeOrEdge::Edge(uid1, uid2, edge) => {
                let _ = dot.write_fmt(format_args!(
                    "    {} -> {} {}\r\n",
                    uid1.clone(),
                    uid2.clone(),
                    serialize_dot(NodeOrEdge::Edge(uid1, uid2, edge))?
                ))?;
            }
        }
    }

    Ok(dot)
}

pub fn add_bar_facet(elements: &mut [NodeOrEdge], name: &str) -> Option<String> {
    for element in elements {
        if let NodeOrEdge::Node(uid, node) = element {
            if uid == name {
                let mut facet_num = 1;

                if let Some(label) = &node.label {
                    facet_num = label.split('|').count() + 1;
                    node.label = Some(format!("{}|<f{}>", label, facet_num));
                } else {
                    node.label = Some("<f1>".to_string());
                }

                return Some(format!("f{}", facet_num));
            }
        }
    }

    None
}

pub fn escape_label(label: &str) -> String {
    label
        .replace('{', r"\\{")
        .replace('}', r"\\}")
        .replace(';', "\n")
        .replace('<', r"\\<")
        .replace('>', r"\\>")
}

pub fn extract_bg_and_note(part: &str, allow_note: bool) -> BgAndNote {
    let mut ret = BgAndNote {
        bg: None,
        is_note: false,
        luma: 128,
        font_color: None,
        part: "".to_string(),
    };

    let mut bg_parts = R_BG_PARTS.captures_iter(part);
    if let Some(caps) = bg_parts.next() {
        if caps.len() == 3 {
            ret.part = caps.get(1).unwrap().as_str().trim().to_string();
            let bg = caps.get(2).unwrap().as_str().trim().to_lowercase();
            let luma = get_luma(&bg);

            if luma < 64.0 {
                ret.font_color = Some("white".to_string());
            } else if luma > 192.0 {
                ret.font_color = Some("black".to_string());
            }

            ret.bg = Some(bg);
        } else {
            ret.part = part.trim().to_string();
        }
    } else {
        ret.part = part.trim().to_string();
    }

    if allow_note && part.starts_with("note:") {
        ret.part = part[5..].trim().to_string();
        ret.is_note = true;
    }

    ret
}

pub fn get_luma(color: &str) -> f64 {
    let mut luma = 128.0;

    if color.starts_with('#') {
        let red = match i64::from_str_radix(&color[1..3], 16) {
            Ok(r) => r as f64,
            Err(_) => return luma,
        };
        let green = match i64::from_str_radix(&color[3..5], 16) {
            Ok(r) => r as f64,
            Err(_) => return luma,
        };
        let blue = match i64::from_str_radix(&color[5..7], 16) {
            Ok(r) => r as f64,
            Err(_) => return luma,
        };

        luma = 0.2126 * red + 0.7152 * green + 0.0722 * blue;
    } else if let Some(l) = COLOR_TABLE.get(color) {
        luma = *l;
    }

    luma
}

pub fn split_yuml_expr(line: &str, separators: &str, escape: Option<char>) -> YumlResult<Vec<String>> {
    let mut word = String::new();
    let mut parts: Vec<String> = vec![];

    let escape = escape.unwrap_or('\\');
    let mut last_char: Option<char> = None;

    let line_length = line.len();
    let mut chars = line.chars().enumerate();

    while let Some((i, c)) = chars.next() {
        if c == escape && i + 1 < line_length {
            word.write_char(c)?;
            if let Some((_, next_c)) = chars.next() {
                word.write_char(next_c)?;
            }
        } else if separators.contains(c) && last_char.is_none() {
            if !word.is_empty() {
                parts.push(word.trim().to_string());
            }

            match c {
                '[' => last_char = Some(']'),
                '(' => last_char = Some(')'),
                '<' => last_char = Some('>'),
                '|' => last_char = Some('|'),
                _ => last_char = None,
            }

            word = c.to_string();
        } else if last_char.map(|lc| lc == c).unwrap_or(false) {
            last_char = None;
            word = word.trim().to_string();
            word.write_char(c)?;
            parts.push(word);
            word = String::new()
        } else {
            word.write_char(c)?;
        }
    }

    if !word.is_empty() {
        parts.push(word.trim().to_string());
    }

    Ok(parts)
}
