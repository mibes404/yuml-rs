mod activity;
mod diagram;
mod error;
mod model;
mod rgb;
mod utils;

use crate::diagram::Diagram;
use crate::error::{OptionsError, YumlResult};
use crate::model::{ChartType, Directions, Options};
use crate::utils::{build_dot_header, process_directives};
use itertools::Itertools;

fn process_yuml_document(text: &str, is_dark: bool) -> YumlResult<String> {
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
            ChartType::Class => String::new(),
            ChartType::UseCase => String::new(),
            ChartType::Activity => activity_diagram(&new_lines, &options)?,
            ChartType::State => String::new(),
            ChartType::Deployment => String::new(),
            ChartType::Package => String::new(),
            ChartType::Sequence => String::new(),
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

fn main() {
    let text = r#"
    // {type:activity}
    // {generate:true}
        
    (start)-><a>[kettle empty]->(Fill Kettle)->|b|
    <a>[kettle full]->|b|->(Boil Kettle)->|c|
    |b|->(Add Tea Bag)->(Add Milk)->|c|->(Pour Water)
    (Pour Water)->(end)
"#;

    match process_yuml_document(text, false) {
        Ok(dot) => println!("{}", dot),
        Err(err) => println!("{}", err),
    }

    // dot -Tsvg sample_dot.txt
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
    fn test_yuml_expression() {
        let expression = parse_yuml_expr("<a>[kettle empty]->(Fill Kettle)->|b|").expect("can not parse");
        assert_eq!(expression.len(), 5);
        let str_ex = expression.iter().map(|expr| expr.to_string()).join(" | ");
        assert_eq!(
            str_ex,
            "a: diamond | [kettle empty]: edge | Fill Kettle: record | : edge | b: mrecord"
        );
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
}
