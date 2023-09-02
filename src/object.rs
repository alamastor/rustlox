use std::{fmt, rc::Rc};

pub struct Objects {
    objects: Vec<Rc<Object>>,
}

impl Objects {
    pub fn new() -> Self {
        Objects { objects: vec![] }
    }

    pub fn new_string(&mut self, chars: String) -> Rc<Object> {
        self.objects.push(Rc::new(Object::String { chars }));
        self.objects.last().unwrap().clone()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Object {
    String { chars: String },
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Object::String { chars } => write!(f, "Object<String<\"{chars}\">>"),
        }
    }
}
