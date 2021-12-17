use std::borrow::Cow;

#[derive(Debug)]
pub struct NoteProps<'a> {
    pub label: Cow<'a, str>,
    pub attributes: Option<Cow<'a, str>>,
}

pub trait LabeledElement {
    fn label(&self) -> Cow<'_, str>;
}
