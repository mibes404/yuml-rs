use crate::error::{OptionsError, YumlResult};
use crate::model::{BgAndNote, Dot, DotShape, Element, Options, YumlExpression};
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

pub fn extract_bg_from_regex(part: &str, re: &Regex) -> Option<YumlExpression> {
    if let Some(object) = re.find(part) {
        let a_str = object.as_str();
        let part = &a_str[1..a_str.len() - 1];
        let ret = extract_bg_and_note(part, true);
        Some(YumlExpression::from(ret))
    } else {
        None
    }
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

pub fn serialize_dot(mut dot: Dot) -> YumlResult<String> {
    let label = dot.label.clone().unwrap_or_default();
    if dot.shape == DotShape::Record && !R_LABEL.is_match(&label) {
        // Graphviz documentation says (https://www.graphviz.org/doc/info/shapes.html):
        // The record-based shape has largely been superseded and greatly generalized by HTML-like labels.
        // That is, instead of using shape=record, one might consider using shape=none, margin=0 and an HTML-like label. [...]
        // Also note that there are problems using non-trivial edges (edges with ports or labels) between adjacent nodes
        // on the same rank if one or both nodes has a record shape.

        if label.contains('|') {
            let mut result =
                r#"[fontsize=10,label=<<TABLE BORDER="0" CELLBORDER="1" CELLSPACING="0" CELLPADDING="9" "#.to_string();
            if let Some(fillcolor) = &dot.fillcolor {
                result.write_fmt(format_args!(r#"BGCOLOR="{}" "#, fillcolor))?;
            }
            if let Some(fontcolor) = &dot.fontcolor {
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
        dot.shape = DotShape::Rectangle
    }

    Ok(dot.to_string())
}

pub fn unescape_label(label: &str) -> String {
    label
        .replace(r"\\{", "{")
        .replace(r"\\}", "}")
        .replace(r"\\<", "<")
        .replace(r"\\>", ">")
}

pub fn format_label(label: &str, wrap: usize, allow_divisors: bool) -> String {
    let mut lines: Vec<&str> = vec![label];
    if allow_divisors && label.contains('|') {
        lines = label.split('|').collect();
    }

    let lines: Vec<String> = lines.iter().map(|line| word_wrap(line, wrap, '\n')).collect();
    escape_label(&lines.join("|"))
}

fn word_wrap(line: &str, width: usize, new_line: char) -> String {
    if line.len() < width {
        return line.to_string();
    }

    if let Some(p) = line.rfind(' ') {
        if p > 0 {
            let left = &line[0..p];
            let right = &line[p + 1..];
            return format!("{}{}{}", left, new_line, word_wrap(right, width, new_line));
        }
    }

    line.to_string()
}

pub fn serialize_dot_elements(mut elements: Vec<Element>) -> YumlResult<String> {
    let mut dot = String::new();
    while let Some(elem) = elements.pop() {
        if let Some(uid2) = elem.uid2 {
            dot.write_fmt(format_args!(
                "    {} -> {} {}\n",
                elem.uid.clone(),
                uid2.clone(),
                serialize_dot(elem.dot)?
            ))?;
        } else {
            dot.write_fmt(format_args!("    {} {}\n", elem.uid.clone(), serialize_dot(elem.dot)?))?;
        }
    }

    Ok(dot)
}

pub fn add_bar_facet(elements: &mut [Element], name: &str) -> Option<String> {
    for element in elements {
        if element.uid == name {
            let mut facet_num = 1;
            let node = &mut element.dot;
            if let Some(label) = &node.label {
                facet_num = label.split('|').count() + 1;
                node.label = Some(format!("{}|<f{}>", label, facet_num));
            } else {
                node.label = Some("<f1>".to_string());
            }

            return Some(format!("f{}", facet_num));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_yuml_expr() {
        let parts = split_yuml_expr("<a>[kettle empty]->(Fill Kettle)->|b|", "(<|", None).expect("can not parse");
        assert_eq!(parts.len(), 5);
        let part = parts.get(0).unwrap();
        assert_eq!(part, "<a>");
        let part = parts.get(1).unwrap();
        assert_eq!(part, "[kettle empty]->");
        let part = parts.get(2).unwrap();
        assert_eq!(part, "(Fill Kettle)");
    }

    #[test]
    fn test_luma() {
        let luma = get_luma("#102030");
        let expected = 0.2126 * (0x10 as f64) + 0.7152 * (0x20 as f64) + 0.0722 * (0x30 as f64);
        assert_eq!(luma, expected);

        let luma = get_luma("PaleVioletRed3");
        let expected = 0.2126 * 205.0 + 0.7152 * 104.0 + 0.0722 * 137.0;
        assert_eq!(luma, expected);
    }

    #[test]
    fn test_escape_label() {
        let escaped = escape_label("{hello}");
        assert_eq!(escaped, r"\\{hello\\}")
    }

    #[test]
    fn test_word_wrap() {
        let wrapped = word_wrap("Hello World!", 4, '\n');
        assert_eq!(wrapped, "Hello\nWorld!");
        let wrapped = word_wrap("Hello World!", 6, '\n');
        assert_eq!(wrapped, "Hello\nWorld!");
        let wrapped = word_wrap("Hello World!", 13, '\n');
        assert_eq!(wrapped, "Hello World!");
    }
}