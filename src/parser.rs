use std::string;
use std::{any::TypeId, char::ToUppercase, fmt::Debug};

use crate::expression::{Binary, Expr, Grouping, Literal, PrintExprVisitor, Unary};
use crate::prompt::Prompt;
use crate::scanner::{Token, TokenType};

pub struct ParseError;

impl Debug for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ParseError")
    }
}

impl ParseError {
    pub fn new() -> Self {
        return ParseError;
    }

    pub fn error(token: Token, message: &str) -> Self {
        Prompt::error(token, message);
        return ParseError::new();
    }
}

#[derive(Debug)]
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        return Parser { tokens, current: 0 };
    }

    pub fn parse(&mut self) -> Box<dyn Expr> {
        return self.expression();
    }

    fn expression(&mut self) -> Box<dyn Expr> {
        return self.equality();
    }

    fn equality(&mut self) -> Box<dyn Expr> {
        let mut expr = self.comparison();

        while self.match_types(vec![TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous();
            let right = self.comparison();
            expr = Box::new(Binary::new(expr, operator, right));
        }
        return expr;
    }

    fn comparison(&mut self) -> Box<dyn Expr> {
        let mut expr = self.term();

        while self.match_types(vec![
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous();
            let right = self.term();
            expr = Box::new(Binary::new(expr, operator, right));
        }

        return expr;
    }

    fn term(&mut self) -> Box<dyn Expr> {
        let mut expr = self.factor();

        while self.match_types(vec![TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous();
            let right = self.factor();
            expr = Box::new(Binary::new(expr, operator, right));
        }

        return expr;
    }

    fn factor(&mut self) -> Box<dyn Expr> {
        let mut expr = self.unary();

        while self.match_types(vec![TokenType::Slash, TokenType::Star]) {
            let operator = self.previous();
            let right = self.unary();
            expr = Box::new(Binary::new(expr, operator, right));
        }

        return expr;
    }

    fn unary(&mut self) -> Box<dyn Expr> {
        if self.match_types(vec![TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            let right = self.unary();
            return Box::new(Unary::new(operator, right));
        }
        return self.primary();
    }

    fn primary(&mut self) -> Box<dyn Expr> {
        if self.match_types(vec![TokenType::False]) {
            return Box::new(Literal::new(Some("false".to_string())));
        } else if self.match_types(vec![TokenType::True]) {
            return Box::new(Literal::new(Some("true".to_string())));
        } else if self.match_types(vec![TokenType::Nil]) {
            return Box::new(Literal::new(None));
        } else if self.match_types(vec![TokenType::Number, TokenType::String]) {
            return Box::new(Literal::new(Some(self.previous().literal.clone().unwrap())));
        } else if self.match_types(vec![TokenType::LeftParen]) {
            let expr = self.expression();
            self.consume(TokenType::RightParen, "Expect ')' after expression.");
            return Box::new(Grouping::new(expr));
        }
        panic!("无法解析 primary 表达式")
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().token_type == TokenType::Semicolon {
                return;
            }

            match self.peek().token_type {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => {
                    return;
                }
                _ => {}
            }
            self.advance();
        }
    }

    fn match_types(&mut self, types: Vec<TokenType>) -> bool {
        for token_type in types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        return false;
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> Token {
        if self.check(token_type) {
            return self.advance();
        }
        panic!("Parser consume error: {}", message);
    }

    fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek().token_type == token_type
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    fn peek(&self) -> &Token {
        self.tokens
            .get(self.current)
            .expect("peek 没有获取到 token")
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        return self.previous();
    }

    fn previous(&self) -> Token {
        self.tokens
            .get(self.current - 1)
            .expect("previous 没有获取到 token")
            .clone()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parser() {
        let tokens = vec![
            Token::new(TokenType::Number, "1".to_string(), 1, Some("1".to_string())),
            Token::new(TokenType::Plus, "+".to_string(), 1, None),
            Token::new(TokenType::Number, "2".to_string(), 1, Some("2".to_string())),
            Token::new(TokenType::Star, "*".to_string(), 1, None),
            Token::new(TokenType::LeftParen, "(".to_string(), 1, None),
            Token::new(TokenType::Number, "3".to_string(), 1, Some("3".to_string())),
            Token::new(TokenType::Minus, "-".to_string(), 1, None),
            Token::new(TokenType::Number, "4".to_string(), 1, Some("4".to_string())),
            Token::new(TokenType::RightParen, ")".to_string(), 1, None),
            Token::new(TokenType::Eof, "".to_string(), 1, None),
        ];
        let mut parser = Parser::new(tokens);
        let expr = parser.expression();
        println!("{:?}", expr);

        let mut printer = PrintExprVisitor;
        expr.accept(&mut printer);
        println!();
    }
}
