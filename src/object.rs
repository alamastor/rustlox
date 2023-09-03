use std::{collections::HashSet, fmt, rc::Rc};

use rstest::rstest;

pub struct Objects {
    objects: Vec<Object>,
    strings: HashSet<Rc<String>>,
}

impl Objects {
    pub fn new() -> Self {
        Objects {
            objects: vec![],
            strings: HashSet::new(),
        }
    }

    pub fn new_string(&mut self, string_data: String) -> Object {
        let string_data = self.strings.get_or_insert(Rc::new(string_data));
        self.objects.push(Object::String {
            chars: string_data.clone(),
        });
        self.objects.last().unwrap().clone()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Object {
    String { chars: Rc<String> },
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Object::String { chars } => write!(f, "Object<String<\"{chars}\">>"),
        }
    }
}

#[rstest]
fn string_interning_works() {
    let mut objects = Objects::new();
    assert_eq!(objects.strings.len(), 0);

    objects.new_string("asdf".to_string());
    assert_eq!(objects.strings.len(), 1);

    objects.new_string("zxcv".to_string());
    assert_eq!(objects.strings.len(), 2);

    objects.new_string("asdf".to_string());
    assert_eq!(objects.strings.len(), 2); // New string should not be added
}
