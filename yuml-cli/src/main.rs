use yuml_rs::{process_yuml_document, render_svg_from_dot};

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

    render_svg_from_dot(&dot)
}
