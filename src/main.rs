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
