use std::str;
use std::fmt;

#[derive(Clone)]
pub struct CreateParserError {
    pub message: String
}
impl fmt::Display for CreateParserError {
    fn fmt(&self, f: &mut fmt::Formatter ) -> fmt::Result {
        write!(f, "Error")
    }
}
impl fmt::Debug for CreateParserError {
    fn fmt(&self, f: &mut fmt::Formatter ) -> fmt::Result {
        write!(f, "Error")
    }
}
impl From<&str> for CreateParserError {
    fn from(inp: &str) -> CreateParserError {
        CreateParserError {
            message: inp.to_owned()
        }
    }
}
impl std::error::Error for CreateParserError {
    fn description(&self) -> &str {
        "Parse error"
    }
}
