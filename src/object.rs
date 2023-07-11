use std::fmt;

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
