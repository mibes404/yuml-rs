use crate::error::{OptionsError, YumlError};
use std::convert::TryFrom;
use std::fmt::{Display, Formatter};

pub struct BgAndNote {
    pub bg: Option<String>,
    pub is_note: bool,
    pub luma: u8,
    pub font_color: Option<String>,
    pub part: String,
}

#[derive(PartialEq)]
pub enum ChartType {
    Class,
    UseCase,
    Activity,
    State,
    Deployment,
    Package,
    Sequence,
}

#[derive(PartialEq)]
pub enum Directions {
    LeftToRight,
    RightToLeft,
    TopDown,
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

pub struct Options {
    pub dir: Directions,
    pub generate: bool,
    pub is_dark: bool,
    pub chart_type: Option<ChartType>,
}

pub enum NodeOrEdge {
    Node(String, Node),
    Edge(String, String, Edge),
}

#[derive(PartialEq)]
pub enum YumlProps {
    NoteOrRecord(bool, String, String),
    Diamond,
    MRecord,
    Edge(EdgeProps),
}

#[derive(PartialEq)]
pub struct EdgeProps {
    pub arrowtail: Option<Arrow>,
    pub arrowhead: Option<Arrow>,
    pub taillabel: Option<String>,
    pub headlabel: Option<String>,
    pub style: Style,
}

#[derive(PartialEq, Clone)]
pub enum Arrow {
    Vee,
    ODiamond,
    Diamond,
    Empty,
}

#[derive(PartialEq, Clone)]
pub enum Style {
    Solid,
    Dashed,
}

impl EdgeProps {
    pub fn arrowtail_str(&self) -> String {
        self.arrowtail
            .as_ref()
            .map(|a| a.to_string())
            .unwrap_or_else(|| "none".to_string())
    }
    pub fn arrowhead_str(&self) -> String {
        self.arrowhead
            .as_ref()
            .map(|a| a.to_string())
            .unwrap_or_else(|| "none".to_string())
    }
}

impl Display for Arrow {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Arrow::Vee => f.write_str("vee"),
            Arrow::Empty => f.write_str("empty"),
            Arrow::ODiamond => f.write_str("odiamond"),
            Arrow::Diamond => f.write_str("diamond"),
        }
    }
}

impl Display for Style {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Style::Solid => f.write_str("solid"),
            Style::Dashed => f.write_str("dashed"),
        }
    }
}

pub struct YumlExpression {
    pub id: String,
    pub props: YumlProps,
}

impl Display for YumlExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.id)?;
        f.write_str(": ")?;
        match self.props {
            YumlProps::NoteOrRecord(is_note, _, _) => {
                if is_note {
                    f.write_str("note")
                } else {
                    f.write_str("record")
                }
            }
            YumlProps::Diamond => f.write_str("diamond"),
            YumlProps::MRecord => f.write_str("mrecord"),
            YumlProps::Edge(_) => f.write_str("edge"),
        }
    }
}

impl From<BgAndNote> for YumlExpression {
    fn from(ret: BgAndNote) -> Self {
        YumlExpression {
            id: ret.part,
            props: YumlProps::NoteOrRecord(
                ret.is_note,
                ret.bg.unwrap_or_default(),
                ret.font_color.unwrap_or_default(),
            ),
        }
    }
}

pub struct Node {
    pub shape: String,
    pub height: f32,
    pub width: f32,
    pub margin: String,
    pub label: Option<String>,
    pub fontsize: Option<i32>,
    pub style: String,
    pub fillcolor: Option<String>,
    pub fontcolor: Option<String>,
    pub penwidth: Option<i32>,
}

impl Display for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("[")?;

        // strings
        f.write_fmt(format_args!(r#"shape="{}" , "#, self.shape))?;
        f.write_fmt(format_args!(r#"margin="{}" , "#, self.margin))?;
        f.write_fmt(format_args!(
            r#"label="{}" , "#,
            self.label.as_deref().unwrap_or_default()
        ))?;
        f.write_fmt(format_args!(r#"style="{}" , "#, self.style))?;
        if let Some(fillcolor) = &self.fillcolor {
            f.write_fmt(format_args!(r#"fillcolor="{}" , "#, fillcolor))?;
        }
        if let Some(fontcolor) = &self.fontcolor {
            f.write_fmt(format_args!(r#"fontcolor="{}" , "#, fontcolor))?;
        }

        // non-strings
        f.write_fmt(format_args!("height={} , ", self.height))?;
        f.write_fmt(format_args!("width={} , ", self.width))?;
        if let Some(fontsize) = &self.fontsize {
            f.write_fmt(format_args!("fontsize={} , ", fontsize))?;
        }
        if let Some(penwidth) = &self.penwidth {
            f.write_fmt(format_args!("penwidth={} , ", penwidth))?;
        }

        f.write_str("]")
    }
}

pub struct Edge {
    pub shape: String,
    pub dir: String,
    pub style: String,
    pub arrowtail: String,
    pub arrowhead: String,
    pub labeldistance: u32,
    pub fontsize: u32,
    pub label: Option<String>,
}

impl Display for Edge {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("[")?;

        // strings
        f.write_fmt(format_args!(r#"shape="{}" , "#, self.shape))?;
        f.write_fmt(format_args!(r#"dir="{}" , "#, self.dir))?;
        f.write_fmt(format_args!(r#"style="{}" , "#, self.style))?;
        f.write_fmt(format_args!(r#"arrowtail="{}" , "#, self.arrowtail))?;
        f.write_fmt(format_args!(r#"arrowhead="{}" , "#, self.arrowhead))?;
        f.write_fmt(format_args!(
            r#"label="{}" , "#,
            self.label.as_deref().unwrap_or_default()
        ))?;

        // non-strings
        f.write_fmt(format_args!("labeldistance={} , ", self.labeldistance))?;
        f.write_fmt(format_args!("fontsize={} , ", self.fontsize))?;
        f.write_str("]")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_node() {
        let node = Node {
            shape: "s".to_string(),
            height: 1.0,
            width: 2.0,
            margin: "m".to_string(),
            label: Some("l".to_string()),
            fontsize: Some(3),
            style: "s".to_string(),
            fillcolor: None,
            fontcolor: Some("fc".to_string()),
            penwidth: None,
        }
        .to_string();

        assert_eq!(
            node,
            r#"[shape="s" , margin="m" , label="l" , style="s" , fontcolor="fc" , height=1 , width=2 , fontsize=3 , ]"#
        );
    }
}
