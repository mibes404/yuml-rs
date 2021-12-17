use super::{
    dot::{Arrow, Directions, Dot, DotElement, DotShape, Style},
    shared::{ElementDetails, LabeledElement, NoteProps},
};
use crate::parser::utils::as_str;
use itertools::Itertools;
use std::{borrow::Cow, cell::RefCell};

#[derive(Debug)]
pub enum Element<'a> {
    Note(NoteProps<'a>),
    Class(Cow<'a, str>),
    Connection(Connection<'a>),
    Inheritance,
}

impl<'a> LabeledElement for Element<'a> {
    fn label(&self) -> Cow<'a, str> {
        match self {
            Element::Note(props) => props.label.clone(),
            Element::Class(label) => label.clone(),
            Element::Connection(_details) => Cow::default(),
            Element::Inheritance => Cow::default(),
        }
    }

    fn is_connection(&self) -> bool {
        matches!(self, Element::Connection(_))
    }
}

#[derive(Debug, Default)]
pub struct Connection<'a> {
    pub left: Connector<'a>,
    pub right: Connector<'a>,
    pub dotted: bool,
}

#[derive(Debug)]
pub enum Connector<'a> {
    None(RelationProps<'a>),
    Directional(RelationProps<'a>),
    Aggregation(RelationProps<'a>),
    Composition(RelationProps<'a>),
    Dependencies(RelationProps<'a>),
    Cardinality(RelationProps<'a>),
}

impl<'a> Default for Connector<'a> {
    fn default() -> Self {
        Connector::None(RelationProps::default())
    }
}

#[derive(Debug, Default)]
pub struct RelationProps<'a> {
    pub label: Option<Cow<'a, str>>,
}

pub fn as_note<'a>(note: (&'a [u8], Option<&'a [u8]>)) -> Element {
    let label = as_str(note.0);
    let attributes = note.1.map(as_str);
    Element::Note(NoteProps { label, attributes })
}

impl<'a> From<&ElementDetails<'a, Element<'a>>> for DotElement {
    fn from(e: &ElementDetails<'a, Element<'a>>) -> Self {
        match e.element {
            Element::Note(_) | Element::Class(_) => DotElement {
                dot: Dot::from(e.element),
                uid: format!("A{}", e.id.unwrap_or_default()),
                uid2: None,
            },
            Element::Connection(_con) => {
                let (uid1, uid2) = if let Some(relation) = &e.relation {
                    let uid1 = format!("A{}", relation.previous_id);
                    let uid2 = format!("A{}", relation.next_id);
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
            Element::Inheritance => DotElement {
                dot: Dot::from(e.element),
                uid: format!("A{}", e.id.unwrap_or_default()),
                uid2: None,
            },
        }
    }
}

impl<'a> From<&Element<'a>> for Dot {
    fn from(e: &Element<'a>) -> Self {
        match e {
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
            Element::Class(label) => Dot {
                shape: DotShape::Rectangle,
                height: Some(0.5),
                margin: Some("0.20,0.05".to_string()),
                label: Some(label.clone().into_owned()),
                style: vec![Style::Rounded],
                fontsize: Some(10),
                ..Dot::default()
            },
            Element::Connection(connection) => {
                let (left_arrow_style, left_props) = extract_props(&connection.left);
                let (right_arrow_style, right_props) = extract_props(&connection.right);

                Dot {
                    shape: DotShape::Edge,
                    style: vec![Style::Solid],
                    dir: Some("both".to_string()),
                    arrowhead: left_arrow_style,
                    arrowtail: right_arrow_style,
                    fontsize: Some(10),
                    labeldistance: Some(1),
                    headlabel: left_props.label.as_ref().map(|s| s.clone().into_owned()),
                    taillabel: right_props.label.as_ref().map(|s| s.clone().into_owned()),
                    ..Dot::default()
                }
            }
            Element::Inheritance => Dot {
                shape: DotShape::Edge,
                style: vec![Style::Dashed],
                dir: Some("both".to_string()),
                arrowhead: Some(Arrow::Vee),
                fontsize: Some(10),
                ..Dot::default()
            },
        }
    }
}

fn extract_props<'a>(props: &'a Connector<'a>) -> (Option<Arrow>, &'a RelationProps<'a>) {
    match &props {
        Connector::Directional(props) => (Some(Arrow::Vee), props),
        Connector::Aggregation(props) | Connector::Cardinality(props) => (Some(Arrow::ODiamond), props),
        Connector::Composition(props) => (Some(Arrow::Diamond), props),
        Connector::Dependencies(props) => (Some(Arrow::Empty), props),
        Connector::None(props) => (None, props),
    }
}
