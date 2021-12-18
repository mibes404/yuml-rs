use crate::model::shared::{ElementDetails, LabeledElement};

use super::*;

pub struct Uids<'a, T: LabeledElement> {
    uids: HashMap<&'a str, (usize, &'a T)>,
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
    pub fn insert_uid(&mut self, label: &'a str, e: &'a T) -> usize {
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

pub fn populate_uids<T: LabeledElement>(elements: &[T]) -> (Uids<T>, Vec<ElementDetails<T>>) {
    let mut uids = Uids::default();

    // we must collect to borrow uids in subsequent iterator
    let element_details: Vec<ElementDetails<T>> = elements
        .iter()
        .filter_map(|e| {
            if e.is_connection() {
                // ignore arrows for now
                None
            } else {
                let lbl = e.label();
                if uids.contains_key(lbl) {
                    None
                } else {
                    let id = uids.insert_uid(lbl, e);
                    Some((id, e))
                }
            }
        })
        .map(|(id, element)| ElementDetails {
            id: Some(id),
            element,
            relation: None,
        })
        .collect();

    (uids, element_details)
}
