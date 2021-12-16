use super::{
    dot::{Arrow, Directions, Dot, DotElement, DotShape, Style},
    shared::NoteProps,
};
use crate::parser::utils::as_str;
use itertools::Itertools;
use std::{borrow::Cow, cell::RefCell};

#[derive(Debug)]
pub enum Element<'a> {
    StartTag,
    EndTag,
    Activity(ElementProps<'a>),
    Parallel(ElementProps<'a>),
    Decision(ElementProps<'a>),
    Arrow(ArrowProps<'a>),
    Note(NoteProps<'a>),
}

pub fn as_note<'a>(note: (&'a [u8], Option<&'a [u8]>)) -> Element {
    let label = as_str(note.0);
    let attributes = note.1.map(as_str);
    Element::Note(NoteProps { label, attributes })
}

impl<'a> Element<'a> {
    pub fn label(&self) -> Cow<'a, str> {
        match self {
            Element::StartTag => Cow::from("start"),
            Element::EndTag => Cow::from("end"),
            Element::Activity(props) | Element::Parallel(props) | Element::Decision(props) => props.label.clone(),
            Element::Arrow(details) => details.label.clone().unwrap_or_default(),
            Element::Note(props) => props.label.clone(),
        }
    }

    pub fn is_arrow(&self) -> bool {
        matches!(self, Element::Arrow(_))
    }

    pub fn is_note(&self) -> bool {
        matches!(self, Element::Note(_))
    }
}

#[derive(Debug)]
pub struct ElementProps<'a> {
    pub label: Cow<'a, str>,
    pub incoming_connections: RefCell<u8>,
}

#[derive(Debug)]
pub struct ArrowProps<'a> {
    pub label: Option<Cow<'a, str>>,
    pub target_connection_id: RefCell<u8>,
    pub dashed: RefCell<bool>,
    pub chart_direction: Directions,
}

impl<'a> ElementProps<'a> {
    pub fn new(label: Cow<'a, str>) -> Self {
        Self {
            label,
            incoming_connections: RefCell::new(0),
        }
    }
}

impl<'a> ArrowProps<'a> {
    pub fn new(label: Option<Cow<'a, str>>, chart_direction: &Directions) -> Self {
        Self {
            label,
            target_connection_id: RefCell::new(0),
            dashed: RefCell::new(false),
            chart_direction: *chart_direction,
        }
    }
}

#[derive(Debug)]
pub struct ElementDetails<'a> {
    pub id: Option<usize>,
    pub element: &'a Element<'a>,
    pub relation: Option<Relation>,
}

#[derive(Debug)]
pub struct Relation {
    pub previous_id: usize,
    pub next_id: usize,
}

impl<'a> From<&ElementDetails<'a>> for DotElement {
    fn from(e: &ElementDetails<'a>) -> Self {
        match e.element {
            Element::StartTag | Element::EndTag => DotElement {
                dot: Dot::from(e.element),
                uid: format!("A{}", e.id.unwrap_or_default()),
                uid2: None,
            },
            Element::Activity(_) | Element::Parallel(_) | Element::Decision(_) | Element::Note(_) => DotElement {
                dot: Dot::from(e.element),
                uid: format!("A{}", e.id.unwrap_or_default()),
                uid2: None,
            },
            Element::Arrow(props) => {
                let target_connection_id = *(props.target_connection_id.borrow());
                let (uid1, uid2) = if let Some(relation) = &e.relation {
                    let uid1 = format!("A{}", relation.previous_id);
                    let uid2 = if target_connection_id > 0 {
                        format!(
                            "A{}:f{}:{}",
                            relation.next_id,
                            target_connection_id,
                            props.chart_direction.head_port()
                        )
                    } else {
                        format!("A{}", relation.next_id)
                    };
                    (uid1, uid2)
                } else {
                    ("A0".to_string(), "A0".to_string())
                };

                DotElement {
                    dot: Dot::from(e.element),
                    uid: uid1,
                    uid2: Some(uid2),
                }
            }
        }
    }
}

impl<'a> From<&Element<'a>> for Dot {
    fn from(e: &Element<'a>) -> Self {
        match e {
            Element::StartTag => Dot {
                shape: DotShape::Circle,
                height: Some(0.3),
                width: Some(0.3),
                ..Dot::default()
            },
            Element::EndTag => Dot {
                shape: DotShape::DoubleCircle,
                height: Some(0.3),
                width: Some(0.3),
                ..Dot::default()
            },
            Element::Activity(props) => Dot {
                shape: DotShape::Rectangle,
                height: Some(0.5),
                margin: Some("0.20,0.05".to_string()),
                label: Some(props.label.clone().into_owned()),
                style: vec![Style::Rounded],
                fontsize: Some(10),
                ..Dot::default()
            },
            Element::Parallel(props) => {
                let incoming_connections = *props.incoming_connections.borrow();
                let label = (1..=incoming_connections).map(|i| format!("<f{}>", i)).join("|");

                Dot {
                    shape: DotShape::Record,
                    height: Some(0.05),
                    width: Some(0.5),
                    penwidth: Some(4),
                    label: Some(label),
                    style: vec![Style::Filled],
                    fontsize: Some(1),
                    ..Dot::default()
                }
            }
            Element::Decision(props) => Dot {
                shape: DotShape::Diamond,
                height: Some(0.5),
                width: Some(0.5),
                label: Some(props.label.clone().into_owned()),
                fontsize: Some(0),
                ..Dot::default()
            },
            Element::Arrow(props) => Dot {
                shape: DotShape::Edge,
                style: vec![Style::Solid],
                dir: Some("both".to_string()),
                arrowhead: Some(Arrow::Vee),
                fontsize: Some(10),
                labeldistance: Some(1),
                label: props.label.as_ref().map(|s| s.clone().into_owned()),
                ..Dot::default()
            },
            // A1 [shape="note" , margin="0.20,0.05" , label="You can stick notes on diagrams too!\\{bg:cornsilk\\}" , style="filled" , fillcolor="cornsilk" , fontcolor="black" , arrowtail="none" , arrowhead="none" , height=0.5 , fontsize=10 , ]
            Element::Note(props) => {
                let (fillcolor, style) = if let Some(attr) = &props.attributes {
                    if attr.starts_with("bg:") {
                        (Some(attr.trim_start_matches("bg:").to_string()), vec![Style::Filled])
                    } else {
                        (None, vec![])
                    }
                } else {
                    (None, vec![])
                };

                Dot {
                    shape: DotShape::Note,
                    height: Some(0.5),
                    margin: Some("0.20,0.05".to_string()),
                    label: Some(props.label.clone().into_owned()),
                    fontsize: Some(10),
                    fillcolor,
                    style,
                    ..Dot::default()
                }
            }
        }
    }
}
