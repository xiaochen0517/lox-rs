mod error;
pub mod token;

use crate::prompt::Prompt;
use error::Error;

// 重导出
pub use token::{LoxType, Token, TokenType};

pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    column: usize,
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
            column: 0,
            error: None,
        }
    }

    pub fn get_line(&self) -> usize {
        self.line
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }
        self.tokens.push(Token::new(
            TokenType::Eof,
            "".to_string(),
            self.line,
            self.column,
            self.column + 1,
            None,
        ));
        self.tokens.clone()
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
                self.column = 0;
            }
            // String literals.
            '"' => {
                self.string();
            }
            // Unexpected character.
            _ => {
                if Scanner::is_digit(c) {
                    self.number();
                    return;
                }
                if Scanner::is_alpha(c) {
                    self.identifier();
                    return;
                }
                self.error = Some(Error {
                    line: self.line,
                    column: self.current - 1,
                    message: format!("Unexpected character: {}", c),
                });
                Prompt::error_by_line(
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
        self.column += 1;
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
        self.column += 1;
        true
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        self.source.chars().nth(self.current).unwrap()
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        }
        self.source.chars().nth(self.current + 1).unwrap()
    }

    fn add_token(&mut self, token_type: TokenType) {
        let text = &self.source[self.start..self.current];
        self.tokens.push(Token::new(
            token_type,
            text.to_string(),
            self.line,
            self.column,
            self.column + text.len(),
            None,
        ));
    }

    fn add_token_with_literal(&mut self, token_type: TokenType, literal: Option<LoxType>) {
        let text = &self.source[self.start..self.current];
        self.tokens.push(Token::new(
            token_type,
            text.to_string(),
            self.line,
            self.column,
            self.column + text.len(),
            literal,
        ));
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
                self.column = 0;
            }
            self.advance();
        }
        if self.is_at_end() {
            self.error = Some(Error {
                line: self.line,
                column: self.current,
                message: "Unterminated string.".to_string(),
            });
            Prompt::error_by_line(
                self.line,
                &self.source,
                self.current,
                "Unterminated string.",
            );
            return;
        }
        self.advance();

        let value = self.source[self.start + 1..self.current - 1].to_string();
        self.add_token_with_literal(TokenType::String, Some(LoxType::new_str(value.as_str())));
    }

    fn is_digit(c: char) -> bool {
        c.is_digit(10)
    }

    fn number(&mut self) {
        while Scanner::is_digit(self.peek()) {
            self.advance();
        }

        if self.peek() == '.' && Scanner::is_digit(self.peek_next()) {
            self.advance();

            while Scanner::is_digit(self.peek()) {
                self.advance();
            }
        }

        let value = self.source[self.start..self.current].to_string();
        let float_value: f64 = value.parse().unwrap();
        self.add_token_with_literal(TokenType::Number, Some(LoxType::new_num(float_value)));
    }

    fn is_alpha(c: char) -> bool {
        c.is_alphabetic() || c == '_'
    }

    fn is_alpha_numeric(c: char) -> bool {
        Scanner::is_alpha(c) || Scanner::is_digit(c)
    }

    fn identifier(&mut self) {
        while Scanner::is_alpha_numeric(self.peek()) {
            self.advance();
        }

        let text = &self.source[self.start..self.current];
        let keywords_map = token::get_keywords_map();
        let token_type = match keywords_map.get(text) {
            Some(t) => t.clone(),
            None => TokenType::Identifier,
        };
        self.add_token(token_type);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scanner() {
        let source = String::from("var a = \"test\";\nvar b = 123.45;");
        let mut scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens();
        for token in tokens {
            println!("{:?}", token);
        }
    }
}
