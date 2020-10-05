use crate::error::YumlResult;
use crate::model::{Options, YumlExpression};

pub trait Diagram {
    fn compose_dot_expr(&self, lines: &[&str], options: &Options) -> YumlResult<String>;
    fn parse_yuml_expr(&self, spec_line: &str) -> YumlResult<Vec<YumlExpression>>;
}
