use crate::scanner::Token;
use std::error::Error;
use std::fmt;
use std::fmt::Formatter;
use crate::prompt::Prompt;

#[derive(Debug)]
pub struct ParseError {
    message: String,
}

impl ParseError {
    pub fn new(message: &str) -> Self {
        ParseError {
            message: message.to_string(),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for ParseError {}

pub fn create_parse_error(token: &Token, message: &str) -> ParseError {
    Prompt::error(token, message);
    ParseError::new(message)
}
