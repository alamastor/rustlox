use crate::object::Object;
use std::{fmt, rc::Rc};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Bool(bool),
    Nil,
    Number(f64),
    Obj(Rc<Object>),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Value::Bool(x) => format!("{x}"),
                Value::Nil => "nil".to_string(),
                Value::Number(x) => format!("{x}"),
                Value::Obj(object) => (**object).to_string(),
            }
        )
    }
}
