mod activity;
mod classdiagram;
mod diagram;
mod error;
mod model;
mod parser;
mod rgb;
mod sequence;
mod utils;

use crate::diagram::Diagram;
use crate::error::{OptionsError, YumlResult};
use crate::model::{ChartType, Directions, Options};
use crate::utils::{build_dot_header, process_directives};
use error::YumlError;
use itertools::Itertools;
use parser::DotFile;
use std::io::{Read, Write};
use std::process::{Command, Stdio};

pub fn parse_file(yuml: &[u8]) -> YumlResult<DotFile> {
    let (_, df) = parser::parse_file(yuml).map_err(|e| YumlError::InvalidFile(e.to_string()))?;
    Ok(df)
}

pub fn process_yuml_document(text: &str, is_dark: bool) -> YumlResult<String> {
    let mut options = Options {
        dir: Directions::TopDown,
        generate: false,
        is_dark,
        chart_type: None,
    };

    let new_lines: YumlResult<Vec<&str>> = text
        .lines()
        .map(|line| line.trim())
        .filter_map(|line| {
            if line.starts_with("//") {
                if let Err(err) = process_directives(line, &mut options) {
                    Some(Err(err))
                } else {
                    None
                }
            } else {
                Some(Ok(line))
            }
        })
        .try_collect();

    // return in case of an error
    let new_lines = new_lines?;

    if new_lines.is_empty() {
        return Ok(String::new());
    }

    let dot = if let Some(chart_type) = &options.chart_type {
        match chart_type {
            ChartType::Class => class_diagram(&new_lines, &options)?,
            ChartType::UseCase => String::new(),
            ChartType::Activity => activity_diagram(&new_lines, &options)?,
            ChartType::State => String::new(),
            ChartType::Deployment => String::new(),
            ChartType::Package => String::new(),
            ChartType::Sequence => sequence_diagram(&new_lines, &options)?,
        }
    } else {
        return Err(OptionsError::new("Missing mandatory 'type' directive").into());
    };

    Ok(format!("{}{}", build_dot_header(is_dark), dot))
}

fn activity_diagram(lines: &[&str], options: &Options) -> YumlResult<String> {
    let activity = activity::Activity {};
    activity.compose_dot_expr(lines, options)
}

fn class_diagram(lines: &[&str], options: &Options) -> YumlResult<String> {
    let class_diagram = classdiagram::ClassDiagram {};
    class_diagram.compose_dot_expr(lines, options)
}

fn sequence_diagram(lines: &[&str], options: &Options) -> YumlResult<String> {
    let sequence_diagram = sequence::Sequence {};
    sequence_diagram.compose_dot_expr(lines, options)
}

/// Render SVG using the "dot" binary.
/// # Panics
/// Panics when the "dot" binary is not installed, or when the dot input is invalid.
pub fn render_svg_from_dot(dot: &str, target_file: &str) -> YumlResult<()> {
    // dot -Tsvg sample_dot.txt
    let dot_process = Command::new("dot")
        .arg("-Tsvg")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to execute process");

    dot_process
        .stdin
        .unwrap()
        .write_all(dot.as_bytes())
        .expect("can not stream to dot process");

    let mut output = String::new();
    dot_process
        .stdout
        .unwrap()
        .read_to_string(&mut output)
        .expect("can not read from dot process");

    std::fs::write(target_file, output)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_activity() {
        let text = include_bytes!("../test/activity.yuml");
        let expected = include_str!("../test/activity.dot");
        let dot = parse_file(text).expect("can not generate activity dot");
        assert_eq!(dot.to_string(), expected);
    }

    #[test]
    fn test_class() {
        let text = include_str!("../test/class.yuml");
        let expected = include_str!("../test/class.dot");
        let dot = process_yuml_document(text, false).expect("can not generate class dot");
        assert_eq!(dot.trim(), expected.trim());
    }

    #[test]
    fn test_sequence() {
        let text = include_str!("../test/sequence.yuml");
        let expected = include_str!("../test/sequence.dot");
        let dot = process_yuml_document(text, false).expect("can not generate sequence dot");
        // std::fs::write("../test/sequence.dot", &dot).expect("can not write output");
        assert_eq!(dot.trim(), expected.trim());
    }
}
