use super::*;

pub fn as_str(b: &[u8]) -> Cow<str> {
    String::from_utf8_lossy(b)
}
