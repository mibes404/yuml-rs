use std::{borrow::Cow, cell::RefCell};

use super::dot::Directions;

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
pub struct NoteProps<'a> {
    pub label: Cow<'a, str>,
    pub attributes: Option<Cow<'a, str>>,
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
