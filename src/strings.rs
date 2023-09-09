use std::{collections::HashSet, rc::Rc};

use rstest::rstest;

pub struct Strings(HashSet<Rc<String>>);

impl Strings {
    pub fn new() -> Self {
        Strings (HashSet::new(),)
    }

    pub fn new_string(&mut self, string_data: String) -> Rc<String> {
        self.0.get_or_insert(Rc::new(string_data)).clone()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

#[rstest]
fn string_interning_works() {
    let mut strings = Strings::new();
    assert_eq!(strings.len(), 0);

    strings.new_string("asdf".to_string());
    assert_eq!(strings.len(), 1);

    strings.new_string("zxcv".to_string());
    assert_eq!(strings.len(), 2);

    strings.new_string("asdf".to_string());
    assert_eq!(strings.len(), 2); // New string should not be added
}
