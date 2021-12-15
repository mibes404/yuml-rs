use crate::error::{OptionsError, YumlError};
use itertools::Itertools;
use std::convert::TryFrom;
use std::fmt::{Display, Formatter, Write};

#[derive(Debug, PartialEq)]
pub enum ChartType {
    Class,
    UseCase,
    Activity,
    State,
    Deployment,
    Package,
    Sequence,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Directions {
    LeftToRight,
    RightToLeft,
    TopDown,
}

impl Default for Directions {
    fn default() -> Self {
        Directions::TopDown
    }
}

impl Display for Directions {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Directions::LeftToRight => f.write_str("LR"),
            Directions::RightToLeft => f.write_str("RL"),
            Directions::TopDown => f.write_str("TB"),
        }
    }
}

impl Directions {
    pub fn head_port(&self) -> &str {
        match self {
            Directions::LeftToRight => "w",
            Directions::RightToLeft => "e",
            Directions::TopDown => "n",
        }
    }
}

impl Display for ChartType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ChartType::Class => f.write_str("class"),
            ChartType::UseCase => f.write_str("usecase"),
            ChartType::Activity => f.write_str("activity"),
            ChartType::State => f.write_str("state"),
            ChartType::Deployment => f.write_str("deployment"),
            ChartType::Package => f.write_str("package"),
            ChartType::Sequence => f.write_str("sequence"),
        }
    }
}

impl TryFrom<&str> for Directions {
    type Error = YumlError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "leftToRight" => Ok(Directions::LeftToRight),
            "rightToLeft" => Ok(Directions::RightToLeft),
            "topDown" => Ok(Directions::TopDown),
            _ => Err(OptionsError::new("invalid value for 'direction'. Allowed values are: leftToRight, rightToLeft, topDown <i>(default)</i>.").into())
        }
    }
}

impl TryFrom<&str> for ChartType {
    type Error = YumlError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "class" => Ok(ChartType::Class),
            "usecase" => Ok(ChartType::UseCase),
            "activity" => Ok(ChartType::Activity),
            "state" => Ok(ChartType::State),
            "deployment" => Ok(ChartType::Deployment),
            "package" => Ok(ChartType::Package),
            "sequence" => Ok(ChartType::Sequence),
            _ => Err(OptionsError::new(
                "invalid value for 'type'. Allowed values are: class, usecase, activity, state, deployment, package.",
            )
            .into()),
        }
    }
}

#[derive(Default)]
pub struct Options {
    pub dir: Directions,
    pub generate: bool,
    pub is_dark: bool,
    pub chart_type: Option<ChartType>,
}

#[derive(PartialEq)]
pub enum DotShape {
    Record,
    Circle,
    DoubleCircle,
    Diamond,
    Note,
    Edge,
    Point,
    Rectangle,
}

impl Default for DotShape {
    fn default() -> Self {
        DotShape::Circle
    }
}

impl Display for DotShape {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DotShape::Record => f.write_str("record"),
            DotShape::Circle => f.write_str("circle"),
            DotShape::DoubleCircle => f.write_str("doublecircle"),
            DotShape::Diamond => f.write_str("diamond"),
            DotShape::Note => f.write_str("note"),
            DotShape::Edge => f.write_str("edge"),
            DotShape::Point => f.write_str("point"),
            DotShape::Rectangle => f.write_str("rectangle"),
        }
    }
}

#[derive(Default)]
pub struct Dot {
    pub shape: DotShape,
    pub height: Option<f32>,
    pub width: Option<f32>,
    pub margin: Option<String>,
    pub label: Option<String>,
    pub fontsize: Option<i32>,
    pub style: Vec<Style>,
    pub fillcolor: Option<String>,
    pub fontcolor: Option<String>,
    pub penwidth: Option<i32>,
    pub dir: Option<String>,
    pub arrowtail: Option<Arrow>,
    pub arrowhead: Option<Arrow>,
    pub taillabel: Option<String>,
    pub headlabel: Option<String>,
    pub labeldistance: Option<u32>,
}

pub struct DotElement {
    pub uid: String,
    pub uid2: Option<String>,
    pub dot: Dot,
}

impl DotElement {
    pub fn new(uid: &str, dot: Dot) -> Self {
        DotElement {
            uid: uid.to_string(),
            uid2: None,
            dot,
        }
    }

    pub fn new_edge(uid: &str, uid2: &str, dot: Dot) -> Self {
        DotElement {
            uid: uid.to_string(),
            uid2: Some(uid2.to_string()),
            dot,
        }
    }
}

impl Display for DotElement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(uid2) = &self.uid2 {
            f.write_fmt(format_args!("    {} -> {} {}", self.uid, uid2, self.dot))
        } else {
            f.write_fmt(format_args!("    {} {}", self.uid, self.dot))
        }
    }
}

pub struct ActivityDotFile {
    dots: Vec<DotElement>,
    dir: Directions,
}

impl ActivityDotFile {
    pub fn new(dots: Vec<DotElement>, options: &Options) -> Self {
        ActivityDotFile { dots, dir: options.dir }
    }
}

impl Display for ActivityDotFile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("digraph G {\n")?;
        f.write_str("  graph [ bgcolor=transparent, fontname=Helvetica ]\n")?;
        f.write_str("  node [ shape=none, margin=0, color=black, fontcolor=black, fontname=Helvetica ]\n")?;
        f.write_str("  edge [ color=black, fontcolor=black, fontname=Helvetica ]\n")?;
        f.write_str("    ranksep = 0.5\n")?;
        f.write_fmt(format_args!("    rankdir = {}\n", self.dir))?;
        for dot in &self.dots {
            f.write_str(&dot.to_string())?;
            f.write_char('\n')?;
        }
        f.write_char('}')
    }
}

#[derive(PartialEq)]
pub struct EdgeProps {
    pub arrowtail: Option<Arrow>,
    pub arrowhead: Option<Arrow>,
    pub taillabel: Option<String>,
    pub headlabel: Option<String>,
    pub style: Style,
}

#[derive(PartialEq)]
pub struct SignalProps {
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub style: Style,
}

#[derive(PartialEq, Clone)]
pub enum Arrow {
    Vee,
    ODiamond,
    Diamond,
    Empty,
    Filled,
    Open,
}

#[derive(PartialEq, Clone)]
pub enum Style {
    Solid,
    Dashed,
    Filled,
    Rounded,
    Invis,
    Async,
}

impl Display for Arrow {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Arrow::Vee => f.write_str("vee"),
            Arrow::Empty => f.write_str("empty"),
            Arrow::ODiamond => f.write_str("odiamond"),
            Arrow::Diamond => f.write_str("diamond"),
            Arrow::Filled => f.write_str("arrow-filled"),
            Arrow::Open => f.write_str("arrow-open"),
        }
    }
}

impl Display for Style {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Style::Solid => f.write_str("solid"),
            Style::Dashed => f.write_str("dashed"),
            Style::Filled => f.write_str("filled"),
            Style::Rounded => f.write_str("rounded"),
            Style::Invis => f.write_str("invis"),
            Style::Async => f.write_str("async"),
        }
    }
}

impl Display for Dot {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("[")?;

        // strings
        f.write_fmt(format_args!(r#"shape="{}" , "#, self.shape))?;
        if let Some(margin) = &self.margin {
            f.write_fmt(format_args!(r#"margin="{}" , "#, margin))?;
        }

        f.write_fmt(format_args!(
            r#"label="{}" , "#,
            self.label.as_deref().unwrap_or_default()
        ))?;

        f.write_fmt(format_args!(
            r#"style="{}" , "#,
            self.style.iter().map(Style::to_string).join(",")
        ))?;

        if let Some(fillcolor) = &self.fillcolor {
            f.write_fmt(format_args!(r#"fillcolor="{}" , "#, fillcolor))?;
        }
        if let Some(fontcolor) = &self.fontcolor {
            f.write_fmt(format_args!(r#"fontcolor="{}" , "#, fontcolor))?;
        }

        if let Some(dir) = &self.dir {
            f.write_fmt(format_args!(r#"dir="{}" , "#, dir))?;
        }

        if let Some(arrowtail) = &self.arrowtail {
            f.write_fmt(format_args!(r#"arrowtail="{}" , "#, arrowtail))?;
        } else {
            f.write_fmt(format_args!(r#"arrowtail="none" , "#))?;
        }

        if let Some(arrowhead) = &self.arrowhead {
            f.write_fmt(format_args!(r#"arrowhead="{}" , "#, arrowhead))?;
        } else {
            f.write_fmt(format_args!(r#"arrowhead="none" , "#))?;
        }

        if let Some(taillabel) = &self.taillabel {
            f.write_fmt(format_args!(r#"taillabel="{}" , "#, taillabel))?;
        }
        if let Some(headlabel) = &self.headlabel {
            f.write_fmt(format_args!(r#"headlabel="{}" , "#, headlabel))?;
        }

        // non-strings
        if let Some(labeldistance) = &self.labeldistance {
            f.write_fmt(format_args!("labeldistance={} , ", labeldistance))?;
        }

        if let Some(height) = &self.height {
            f.write_fmt(format_args!("height={} , ", height))?;
        }

        if let Some(width) = &self.width {
            f.write_fmt(format_args!("width={} , ", width))?;
        }
        if let Some(fontsize) = &self.fontsize {
            f.write_fmt(format_args!("fontsize={} , ", fontsize))?;
        }
        if let Some(penwidth) = &self.penwidth {
            f.write_fmt(format_args!("penwidth={} , ", penwidth))?;
        }

        f.write_str("]")
    }
}

#[derive(Clone)]
pub struct Actor {
    pub actor_type: String,
    pub name: String,
    pub label: String,
    pub index: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_node() {
        let node = Dot {
            shape: DotShape::Note,
            height: Some(1.0),
            width: Some(2.0),
            margin: Some("m".to_string()),
            label: Some("l".to_string()),
            fontsize: Some(3),
            style: vec![Style::Solid],
            fillcolor: None,
            fontcolor: Some("fc".to_string()),
            penwidth: None,
            dir: None,
            arrowtail: None,
            arrowhead: None,
            taillabel: None,
            headlabel: None,
            labeldistance: None,
        }
        .to_string();

        assert_eq!(
            node,
            r#"[shape="note" , margin="m" , label="l" , style="solid" , fontcolor="fc" , arrowtail="none" , arrowhead="none" , height=1 , width=2 , fontsize=3 , ]"#
        );
    }
}
