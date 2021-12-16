use std::borrow::Cow;

#[derive(Debug)]
pub struct NoteProps<'a> {
    pub label: Cow<'a, str>,
    pub attributes: Option<Cow<'a, str>>,
}
