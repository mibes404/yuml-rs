use super::{
    dot::{Arrow, Directions, Dot, DotElement, DotShape, Style},
    shared::NoteProps,
};
use crate::parser::utils::as_str;
use itertools::Itertools;
use std::{borrow::Cow, cell::RefCell};

#[derive(Debug)]
pub enum Element<'a> {
    Note(NoteProps<'a>),
    Class(Cow<'a, str>),
    Connection(Connection<'a>),
}

#[derive(Debug, Default)]
pub struct Connection<'a> {
    pub left: Connector<'a>,
    pub right: Connector<'a>,
    pub dotted: bool,
}

#[derive(Debug)]
pub enum Connector<'a> {
    Directional(RelationProps<'a>),
    Aggregation(RelationProps<'a>),
    Composition(RelationProps<'a>),
    Inheritance(RelationProps<'a>),
    Dependencies(RelationProps<'a>),
    Cardinality(RelationProps<'a>),
}

impl<'a> Default for Connector<'a> {
    fn default() -> Self {
        Connector::Directional(RelationProps::default())
    }
}

#[derive(Debug, Default)]
pub struct RelationProps<'a> {
    pub label: Option<Cow<'a, str>>,
    pub target_connection_id: RefCell<u8>,
}

pub fn as_note<'a>(note: (&'a [u8], Option<&'a [u8]>)) -> Element {
    let label = as_str(note.0);
    let attributes = note.1.map(as_str);
    Element::Note(NoteProps { label, attributes })
}
