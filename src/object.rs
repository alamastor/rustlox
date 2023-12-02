use std::{fmt, rc::Rc};

use crate::chunk::Chunk;

#[derive(Debug, Clone, PartialEq)]
pub enum Object {
    String { chars: Rc<String> },
    Function(Function),
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Object::String { chars } => write!(f, "\"{chars}\""),
            Object::Function(function) => function.fmt(f),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    arity: usize,
    pub chunk: Chunk,
    pub name: Option<String>,
}

impl Function {
    pub fn new(name: Option<String>) -> Self {
        Function {
            arity: 0,
            chunk: Chunk::new(),
            name,
        }
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.name {
            Some(name) => write!(f, "fn <{name}>"),
            None => write!(f, " <script>")
        }
    }
}
