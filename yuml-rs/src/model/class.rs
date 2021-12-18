use super::{
    dot::{Arrow, Dot, DotElement, DotShape, Style},
    shared::{ElementDetails, LabeledElement, NoteProps},
};
use itertools::Itertools;

#[derive(Debug)]
pub enum Element<'a> {
    Note(NoteProps<'a>),
    Class(&'a str),
    Connection(Connection<'a>),
    Inheritance,
}

impl<'a> LabeledElement for Element<'a> {
    fn label(&self) -> &'a str {
        match self {
            Element::Note(props) => props.label,
            Element::Class(label) => {
                if label.contains('|') {
                    label.split('|').next().unwrap()
                } else {
                    label
                }
            }
            Element::Connection(_details) => "",
            Element::Inheritance => "",
        }
    }

    fn is_connection(&self) -> bool {
        matches!(self, Element::Connection(_)) || matches!(self, Element::Inheritance)
    }
}

#[derive(Debug, Default)]
pub struct Connection<'a> {
    pub left: Connector<'a>,
    pub right: Connector<'a>,
    pub dashed: bool,
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
    pub label: Option<&'a str>,
}

pub fn as_note<'a>(note: (&'a str, Option<&'a str>)) -> Element {
    let label = note.0;
    let attributes = note.1;
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
            Element::Inheritance => {
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
                    label: Some(props.label.to_string()),
                    fontsize: Some(10),
                    fillcolor,
                    style,
                    ..Dot::default()
                }
            }
            Element::Class(label) => {
                let (label, margin) = if label.contains('|') {
                    let rows = label
                        .split('|')
                        .into_iter()
                        .map(|row| format!("<TR><TD>{}</TD></TR>", row))
                        .join("");

                    let table = format!(
                        "<<TABLE BORDER=\"0\" CELLBORDER=\"1\" CELLSPACING=\"0\" CELLPADDING=\"9\">{}</TABLE>>",
                        rows
                    );

                    (table, None)
                } else {
                    (label.to_string(), Some("0.20,0.05".to_string()))
                };

                Dot {
                    shape: DotShape::Rectangle,
                    height: Some(0.5),
                    margin,
                    label: Some(label),
                    fontsize: Some(10),
                    ..Dot::default()
                }
            }
            Element::Connection(connection) => {
                let (left_arrow_style, left_props) = extract_props(&connection.left);
                let (right_arrow_style, right_props) = extract_props(&connection.right);

                Dot {
                    shape: DotShape::Edge,
                    style: if connection.dashed {
                        vec![Style::Dashed]
                    } else {
                        vec![Style::Solid]
                    },
                    dir: Some("both".to_string()),
                    arrowtail: left_arrow_style,
                    arrowhead: right_arrow_style,
                    fontsize: Some(10),
                    labeldistance: Some(2),
                    taillabel: left_props.label.as_ref().map(|s| s.to_string()),
                    headlabel: right_props.label.as_ref().map(|s| s.to_string()),
                    ..Dot::default()
                }
            }
            Element::Inheritance => Dot {
                shape: DotShape::Edge,
                style: vec![Style::Solid],
                dir: Some("both".to_string()),
                arrowtail: Some(Arrow::Empty),
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
