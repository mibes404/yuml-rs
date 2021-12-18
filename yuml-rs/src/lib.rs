//! Parse yUML as SVG using the "dot" binary from the ["graphviz"](https://graphviz.org/download/) toolset.
//!
//! Based on the Javascript version from Jaime Olivares: [yuml-diagram](https://github.com/jaime-olivares/yuml-diagram).
//! At the moment only Activity diagrams are supported, with no guarantees that the other variations will be added in the future.

mod error;
mod model;
mod parser;

use crate::error::YumlResult;
use error::YumlError;
use parser::ParsedYuml;
use std::{
    fs::File,
    io::Write,
    process::{Command, Stdio},
};

/// Generate the interediate `DotFile` from the yUML input.
/// Usage:
/// ```rust,no_run
/// use std::fs::read_to_string;
/// use yuml_rs::parse_yuml;
///
/// let yuml = read_to_string("activity.yaml").expect("can not read input file");
/// let dot = parse_yuml(&yuml).expect("invalid yUML");
/// ```
pub fn parse_yuml(yuml: &str) -> YumlResult<ParsedYuml> {
    let (_, df) = parser::parse_yuml(yuml).map_err(|e| YumlError::InvalidFile(e.to_string()))?;
    Ok(df)
}

/// Render SVG using the "dot" binary, taking a valid dot-description as input.
/// Usage:
/// ```rust,no_run
/// use std::fs::read_to_string;
/// use yuml_rs::{parse_yuml, render_svg_from_dot};
///
/// let yuml = read_to_string("activity.yaml").expect("can not read input file");
/// let dot = parse_yuml(&yuml).expect("invalid yUML");
/// render_svg_from_dot(&dot.to_string()).expect("can not generate SVG");
/// ```
/// # Panics
/// Panics when the "dot" binary is not installed, or when the dot input is invalid.
pub fn render_svg_from_dot(dot: &str) -> YumlResult<impl std::io::Read> {
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

    let data_out = dot_process.stdout.unwrap();
    Ok(data_out)
}

/// Similar to `render_svg_from_dot` but writes the output directly to a file
pub fn write_svg_from_dot(dot: &str, target_file: &str) -> YumlResult<()> {
    let mut data_out = render_svg_from_dot(dot)?;
    let mut output_file = File::create(target_file)?;
    std::io::copy(&mut data_out, &mut output_file)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_activity() {
        let text = include_str!("../test/activity.yuml");
        let expected = include_str!("../test/activity.dot");
        let dot = parse_yuml(text).expect("can not generate activity dot");
        assert_eq!(dot.to_string(), expected);
    }
}
