use std::fs::{read, read_to_string};

use clap::{App, Arg};
use yuml_rs::{parse_file, process_yuml_document, render_svg_from_dot};

const SAMPLE_YUML: &str = r#"
// {type:activity}
// {generate:true}

(start)-><a>[kettle empty]->(Fill Kettle)->|b|
<a>[kettle full]->|b|->(Boil Kettle)->|c|
|b|->(Add Tea Bag)->(Add Milk)->|c|->(Pour Water)
(Pour Water)->(end)
"#;

fn main() {
    let matches = App::new("yUML diagram utility")
        .version("0.1")
        .author("Marcel Ibes <mibes@avaya.com>")
        .about("Renders SVG and PNG images based on yUML input")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("FILE")
                .help("Sets the input yUML file")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("FILE")
                .help("Sets the input SVG file")
                .required(true)
                .takes_value(true),
        )
        .get_matches();

    let input_file = matches.value_of("input").expect("an input file is mandatory");
    let output_file = matches.value_of("output").expect("an output file is mandatory");
    let text = read(input_file).expect("can not read input file");

    let dot = match parse_file(&text) {
        Ok(dot) => dot,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };

    render_svg_from_dot(&dot.to_string(), output_file).expect("can not write output file");
}
