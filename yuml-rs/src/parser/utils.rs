use crate::model::shared::LabeledElement;

use super::*;

pub fn as_str(b: &[u8]) -> Cow<str> {
    String::from_utf8_lossy(b)
}

pub struct Uids<'a, T: LabeledElement> {
    uids: HashMap<Cow<'a, str>, (usize, &'a T)>,
    uid: usize,
}

impl<'a, T: LabeledElement> Default for Uids<'a, T> {
    fn default() -> Self {
        Self {
            uids: Default::default(),
            uid: Default::default(),
        }
    }
}

impl<'a, T: LabeledElement> Uids<'a, T> {
    pub fn insert_uid(&mut self, label: Cow<'a, str>, e: &'a T) -> usize {
        self.uid += 1;
        self.uids.insert(label, (self.uid, e));
        self.uid
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.uids.contains_key(key)
    }

    pub fn get(&'a self, key: &str) -> Option<&'a (usize, &'a T)> {
        self.uids.get(key)
    }
}
