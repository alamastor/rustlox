use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
    Bool(bool),
    Nil,
    Number(f64),
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
            }
        )
    }
}
