use std::borrow::Cow;

#[derive(Debug)]
pub struct NoteProps<'a> {
    pub label: Cow<'a, str>,
    pub attributes: Option<Cow<'a, str>>,
}

pub trait LabeledElement {
    fn label(&self) -> Cow<'_, str>;
    fn is_connection(&self) -> bool;
}

#[derive(Debug)]
pub struct ElementDetails<'a, T: LabeledElement> {
    pub id: Option<usize>,
    pub element: &'a T,
    pub relation: Option<Relation>,
}

#[derive(Debug)]
pub struct Relation {
    pub previous_id: usize,
    pub next_id: usize,
}
