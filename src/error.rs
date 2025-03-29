use std::fmt;

#[derive(Debug, Clone)]
pub struct WsonParseError {
    pub message: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
}

impl fmt::Display for WsonParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let (Some(line), Some(col)) = (self.line, self.column) {
            write!(f, "{} (line {}, column {})", self.message, line, col)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

impl std::error::Error for WsonParseError {}

impl WsonParseError {
    pub fn new<S: Into<String>>(msg: S, line: Option<usize>, col: Option<usize>) -> Self {
        Self {
            message: msg.into(),
            line,
            column: col,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WsonSerializeError {
    pub message: String,
}

impl fmt::Display for WsonSerializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for WsonSerializeError {}

impl WsonSerializeError {
    pub fn new<S: Into<String>>(msg: S) -> Self {
        Self {
            message: msg.into(),
        }
    }
}
