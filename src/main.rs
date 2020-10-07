mod activity;
mod classdiagram;
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
use std::io::{Read, Write};
use std::process::{Command, Stdio};

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
            ChartType::Class => class_diagram(&new_lines, &options)?,
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

fn class_diagram(lines: &[&str], options: &Options) -> YumlResult<String> {
    let class_diagram = classdiagram::ClassDiagram {};
    class_diagram.compose_dot_expr(lines, options)
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

    let dot = match process_yuml_document(text, false) {
        Ok(dot) => dot,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };

    render_svg(dot)
}

fn render_svg(dot: String) {
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

    println!("{}", output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_activity() {
        let text = r#"
    // {type:activity}
    // {generate:true}
        
    (start)-><a>[kettle empty]->(Fill Kettle)->|b|
    <a>[kettle full]->|b|->(Boil Kettle)->|c|
    |b|->(Add Tea Bag)->(Add Milk)->|c|->(Pour Water)
    (Pour Water)->(end)
"#;

        let expected = r#"digraph G {
  graph [ bgcolor=transparent, fontname=Helvetica ]
  node [ shape=none, margin=0, color=black, fontcolor=black, fontname=Helvetica ]
  edge [ color=black, fontcolor=black, fontname=Helvetica ]
    ranksep = 0.5
    rankdir = TB
    A9 -> A10 [shape="edge" , dir="both" , style="solid" , arrowtail="none" , arrowhead="vee" , label="" , labeldistance=1 , fontsize=10 , ]
    A10 [shape="doublecircle" , margin="0,0" , label="" , style="" , height=0.3 , width=0.3 , ]
    A6 -> A9 [shape="edge" , dir="both" , style="solid" , arrowtail="none" , arrowhead="vee" , label="" , labeldistance=1 , fontsize=10 , ]
    A8 -> A6:f2:n [shape="edge" , dir="both" , style="solid" , arrowtail="none" , arrowhead="vee" , label="" , labeldistance=1 , fontsize=10 , ]
    A7 -> A8 [shape="edge" , dir="both" , style="solid" , arrowtail="none" , arrowhead="vee" , label="" , labeldistance=1 , fontsize=10 , ]
    A4 -> A7 [shape="edge" , dir="both" , style="solid" , arrowtail="none" , arrowhead="vee" , label="" , labeldistance=1 , fontsize=10 , ]
    A9 [shape="rectangle" , margin="0.20,0.05" , label="Pour Water" , style="rounded" , height=0.5 , fontsize=10 , ]
    A8 [shape="rectangle" , margin="0.20,0.05" , label="Add Milk" , style="rounded" , height=0.5 , fontsize=10 , ]
    A7 [shape="rectangle" , margin="0.20,0.05" , label="Add Tea Bag" , style="rounded" , height=0.5 , fontsize=10 , ]
    A5 -> A6:f1:n [shape="edge" , dir="both" , style="solid" , arrowtail="none" , arrowhead="vee" , label="" , labeldistance=1 , fontsize=10 , ]
    A4 -> A5 [shape="edge" , dir="both" , style="solid" , arrowtail="none" , arrowhead="vee" , label="" , labeldistance=1 , fontsize=10 , ]
    A2 -> A4:f2:n [shape="edge" , dir="both" , style="solid" , arrowtail="none" , arrowhead="vee" , label="[kettle full]" , labeldistance=1 , fontsize=10 , ]
    A6 [shape="record" , margin="0,0" , label="<f1>|<f2>" , style="filled" , height=0.05 , width=0.5 , fontsize=1 , penwidth=4 , ]
    A5 [shape="rectangle" , margin="0.20,0.05" , label="Boil Kettle" , style="rounded" , height=0.5 , fontsize=10 , ]
    A3 -> A4:f1:n [shape="edge" , dir="both" , style="solid" , arrowtail="none" , arrowhead="vee" , label="" , labeldistance=1 , fontsize=10 , ]
    A2 -> A3 [shape="edge" , dir="both" , style="solid" , arrowtail="none" , arrowhead="vee" , label="[kettle empty]" , labeldistance=1 , fontsize=10 , ]
    A1 -> A2 [shape="edge" , dir="both" , style="solid" , arrowtail="none" , arrowhead="vee" , label="" , labeldistance=1 , fontsize=10 , ]
    A4 [shape="record" , margin="0,0" , label="<f1>|<f2>" , style="filled" , height=0.05 , width=0.5 , fontsize=1 , penwidth=4 , ]
    A3 [shape="rectangle" , margin="0.20,0.05" , label="Fill Kettle" , style="rounded" , height=0.5 , fontsize=10 , ]
    A2 [shape="diamond" , margin="0,0" , label="" , style="" , height=0.5 , width=0.5 , fontsize=0 , ]
    A1 [shape="circle" , margin="0,0" , label="" , style="" , height=0.3 , width=0.3 , ]
}"#;

        let dot = process_yuml_document(text, false).expect("can not generate activity dot");
        assert_eq!(dot.trim(), expected.trim());
    }

    #[test]
    fn test_class() {
        let text = r#"
// {type:class}
// {direction:topDown}
// {generate:true}

[note: You can stick notes on diagrams too!{bg:cornsilk}]
[Customer]<>1-orders 0..*>[Order]
[Order]++*-*>[LineItem]
[Order]-1>[DeliveryMethod]
[Order]*-*>[Product|EAN_Code|promo_price()]
[Category]<->[Product]
[DeliveryMethod]^[National]
[DeliveryMethod]^[International]"#;
        let expected = "";
        let dot = process_yuml_document(text, false).expect("can not generate activity dot");
        assert_eq!(dot.trim(), expected.trim());
    }
}
