mod error;
pub mod token;

use crate::prompt::Prompt;
use error::Error;

// 重导出
pub use token::{Token, TokenType};

pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    error: Option<Error>,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Scanner {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            error: None,
        }
    }

    pub fn get_line(&self) -> usize {
        self.line
    }

    pub fn scan_tokens(&mut self) -> &Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }
        self.tokens
            .push(Token::new(TokenType::Eof, "".to_string(), self.line, None));
        &self.tokens
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            // Single-character tokens.
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            ';' => self.add_token(TokenType::Semicolon),
            '*' => self.add_token(TokenType::Star),
            // Double-character tokens.
            '!' => {
                let match_quote = self.match_char('=');
                self.add_token(if match_quote {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                })
            }
            '=' => {
                let match_quote = self.match_char('=');
                self.add_token(if match_quote {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                })
            }
            '<' => {
                let match_quote = self.match_char('=');
                self.add_token(if match_quote {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                })
            }
            '>' => {
                let match_quote = self.match_char('=');
                self.add_token(if match_quote {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                })
            }
            // Comments and whitespace.
            '/' => {
                if self.match_char('/') {
                    // A comment goes until the end of the line.
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Slash);
                }
            }
            // Whitespace and newlines.
            ' ' | '\r' | '\t' => { /* Ignore whitespace. */ }
            '\n' => {
                self.line += 1;
            }
            // Unexpected character.
            _ => {
                self.error = Some(Error {
                    line: self.line,
                    column: self.current - 1,
                    message: format!("Unexpected character: {}", c),
                });
                Prompt::error(
                    self.line,
                    &self.source,
                    self.current - 1,
                    &format!("Unexpected character: {}", c),
                );
            }
        }
    }

    fn advance(&mut self) -> char {
        let c = self.source.chars().nth(self.current).unwrap();
        self.current += 1;
        c
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source.chars().nth(self.current).unwrap() != expected {
            return false;
        }
        self.current += 1;
        true
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        self.source.chars().nth(self.current).unwrap()
    }

    fn add_token(&mut self, token_type: TokenType) {
        let text = &self.source[self.start..self.current];
        self.tokens
            .push(Token::new(token_type, text.to_string(), self.line, None));
    }

    fn add_token_with_literal(&mut self, token_type: TokenType, literal: Option<String>) {
        let text = &self.source[self.start..self.current];
        self.tokens
            .push(Token::new(token_type, text.to_string(), self.line, literal));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scanner() {
        let source = String::from("var a = 10;");
        let mut scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens();
        for token in tokens {
            println!("{:?}", token);
        }
    }
}
