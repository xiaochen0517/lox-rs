use std::string;
use std::{any::TypeId, char::ToUppercase, fmt::Debug};

use crate::ast::{
    Binary, Expr, Expression, Grouping, Literal, Print, PrintExprVisitor, Stmt, Unary,
};
use crate::prompt::Prompt;
use crate::scanner::{LoxType, Token, TokenType};

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

    pub fn parse(&mut self) -> Vec<Box<dyn Stmt>> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(self.statement());
        }
        return statements;
    }

    fn expression(&mut self) -> Box<dyn Expr> {
        return self.equality();
    }

    fn statement(&mut self) -> Box<dyn Stmt> {
        if self.match_types(vec![TokenType::Print]) {
            return self.print_statement();
        }
        return self.expression_statement();
    }

    fn print_statement(&mut self) -> Box<dyn Stmt> {
        let value = self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after value.");
        return Box::new(Print::new(value));
    }

    fn expression_statement(&mut self) -> Box<dyn Stmt> {
        let expr = self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after expression.");
        return Box::new(Expression::new(expr));
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
            return Box::new(Literal::new(Some(LoxType::new_bool(false))));
        } else if self.match_types(vec![TokenType::True]) {
            return Box::new(Literal::new(Some(LoxType::new_bool(true))));
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
    use crate::scanner::LoxType;

    use super::*;

    #[test]
    fn test_parser() {
        let tokens = vec![
            Token::new(
                TokenType::Number,
                "1".to_string(),
                1,
                1,
                1,
                Some(LoxType::new_str("1")),
            ),
            Token::new(TokenType::Plus, "+".to_string(), 1, 2, 2, None),
            Token::new(
                TokenType::Number,
                "2".to_string(),
                1,
                3,
                3,
                Some(LoxType::new_str("2")),
            ),
            Token::new(TokenType::Star, "*".to_string(), 1, 4, 4, None),
            Token::new(TokenType::LeftParen, "(".to_string(), 1, 5, 5, None),
            Token::new(
                TokenType::Number,
                "3".to_string(),
                1,
                6,
                6,
                Some(LoxType::new_str("3")),
            ),
            Token::new(TokenType::Minus, "-".to_string(), 1, 7, 7, None),
            Token::new(
                TokenType::Number,
                "4".to_string(),
                1,
                8,
                8,
                Some(LoxType::new_str("4")),
            ),
            Token::new(TokenType::RightParen, ")".to_string(), 1, 9, 9, None),
            Token::new(TokenType::Eof, "".to_string(), 1, 10, 10, None),
        ];
        let mut parser = Parser::new(tokens);
        let expr = parser.expression();
        println!("{:?}", expr);

        let mut printer = PrintExprVisitor;
        expr.accept(&mut printer);
        println!();
    }
}
