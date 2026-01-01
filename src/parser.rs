use std::fmt::Debug;
mod error;

use crate::ast::{
    Assign, Binary, Block, Call, Expr, Expression, Grouping, If, Literal, Logical, Print,
    PrintExprVisitor, Return, Stmt, Unary, Var, Variable, While,
};
use crate::parser::error::{ParseError, create_parse_error};
use crate::scanner::TokenType::Or;
use crate::scanner::{LoxType, Token, TokenType};

#[derive(Debug)]
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Vec<Box<dyn Stmt>> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(self.declaration());
        }
        statements
    }

    fn declaration(&mut self) -> Box<dyn Stmt> {
        let result = if self.match_types(vec![TokenType::Fun]) {
            self.function("function")
        } else if self.match_types(vec![TokenType::Var]) {
            self.var_declaration()
        } else {
            self.statement()
        };
        result.unwrap_or_else(|err| {
            self.synchronize();
            Box::new(Expression::new(Box::new(Literal::new(None))))
        })
    }

    fn function(&mut self, kind: &str) -> Result<Box<dyn Stmt>, ParseError> {
        let name = self.consume(
            TokenType::Identifier,
            format!("Expect '{}' name.", kind).as_str(),
        )?;
        // 解析括号部分
        self.consume(
            TokenType::LeftParen,
            format!("Expect '(' after {} name.", kind).as_str(),
        )?;
        let mut parameters = Vec::new();
        if !self.check(TokenType::RightParen) {
            loop {
                if parameters.len() >= 255 {
                    return Err(ParseError::new("Can't have more than 255 parameters."));
                }
                parameters.push(self.consume(TokenType::Identifier, "Expect parameter name")?);
                if !self.match_types(vec![TokenType::Comma]) {
                    break;
                }
            }
        }
        self.consume(
            TokenType::RightParen,
            format!("Expect ')' after parameters of {}.", kind).as_str(),
        )?;
        // 解析函数体部分
        self.consume(
            TokenType::LeftBrace,
            format!("Expect '{{' before {} body.", kind).as_str(),
        )?;
        let body = self.block()?;
        // 拼接函数节点并返回
        Ok(Box::new(crate::ast::Function::new(name, parameters, body)))
    }

    fn var_declaration(&mut self) -> Result<Box<dyn Stmt>, ParseError> {
        let name = self.consume(TokenType::Identifier, "Expect variable name")?;

        let mut initializer: Box<dyn Expr> = Box::new(Literal::new(None));
        if self.match_types(vec![TokenType::Equal]) {
            initializer = self.expression()?;
        }
        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration",
        )?;
        Ok(Box::new(Var::new(name, initializer)))
    }

    fn expression(&mut self) -> Result<Box<dyn Expr>, ParseError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Box<dyn Expr>, ParseError> {
        let expr = self.or()?;

        if self.match_types(vec![TokenType::Equal]) {
            let equals = self.previous();
            let value = self.assignment()?;

            if let Some(var_expr) = expr.as_any().downcast_ref::<Variable>() {
                let name = var_expr.name.clone();
                return Ok(Box::new(Assign::new(name, value)));
            }

            let err_message = "Invalid assignment target.";
            return Err(create_parse_error(&equals, err_message));
        }

        Ok(expr)
    }

    fn or(&mut self) -> Result<Box<dyn Expr>, ParseError> {
        let mut expr = self.and()?;

        while self.match_types(vec![TokenType::Or]) {
            let operator = self.previous();
            let right = self.and()?;
            expr = Box::new(Logical::new(expr, operator, right));
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Box<dyn Expr>, ParseError> {
        let mut expr = self.equality()?;

        while self.match_types(vec![TokenType::And]) {
            let operator = self.previous();
            let right = self.equality()?;
            expr = Box::new(Logical::new(expr, operator, right));
        }

        Ok(expr)
    }

    fn statement(&mut self) -> Result<Box<dyn Stmt>, ParseError> {
        if self.match_types(vec![TokenType::Print]) {
            return self.print_statement();
        }
        if self.match_types(vec![TokenType::Return]) {
            return self.return_statement();
        }
        if self.match_types(vec![TokenType::LeftBrace]) {
            return Ok(Box::new(Block::new(self.block()?)));
        }
        if self.match_types(vec![TokenType::If]) {
            return self.if_statement();
        }
        if self.match_types(vec![TokenType::While]) {
            return self.while_statement();
        }
        if self.match_types(vec![TokenType::For]) {
            return self.for_statement();
        }
        self.expression_statement()
    }

    fn return_statement(&mut self) -> Result<Box<dyn Stmt>, ParseError> {
        let keyword = self.previous();
        let mut value: Option<Box<dyn Expr>> = None;
        if !self.check(TokenType::Semicolon) {
            value = Some(self.expression()?);
        }
        self.consume(TokenType::Semicolon, "Expect ';' after return value.")?;
        Ok(Box::new(Return::new(keyword, value)))
    }

    fn for_statement(&mut self) -> Result<Box<dyn Stmt>, ParseError> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'for'.")?;
        // 解析初始化部分
        let mut initializer: Option<Box<dyn Stmt>> = None;
        if (self.match_types(vec![TokenType::Semicolon])) {
            // 省略初始化部分
            initializer = None;
        } else if self.match_types(vec![TokenType::Var]) {
            // 初始化部分为变量定义
            initializer = Some(self.var_declaration()?);
        } else {
            // 初始化部分为一个表达式
            initializer = Some(self.expression_statement()?);
        }
        // 解析条件表达式
        let mut condition: Option<Box<dyn Expr>> = None;
        if !self.check(TokenType::Semicolon) {
            // 条件表达式存在
            condition = Some(self.expression()?);
        }
        self.consume(TokenType::Semicolon, "Expect ';' after loop condition.")?;
        // 解析后处理部分
        let mut increment: Option<Box<dyn Expr>> = None;
        if !self.check(TokenType::RightParen) {
            // 后处理部分存在
            increment = Some(self.expression()?);
        }
        self.consume(TokenType::RightParen, "Expect ')' after for clauses.")?;
        // 获取到主要执行部分
        let mut body = self.statement()?;
        // 脱糖流程，将for转换为while格式
        // 将自增后处理部分合并到body中
        if increment.is_some() {
            let increment_expression = Box::new(Expression::new(increment.unwrap()));
            body = Box::new(Block::new(vec![body, increment_expression]));
        }
        // 将条件部分与body合并
        if condition.is_none() {
            let true_expr = Literal::new(Some(LoxType::new_bool(true)));
            condition = Some(Box::new(true_expr));
        }
        body = Box::new(While::new(condition.unwrap(), body));
        // 将初始化部分与body合并
        if initializer.is_some() {
            body = Box::new(Block::new(vec![initializer.unwrap(), body]));
        }
        Ok(body)
    }

    fn print_statement(&mut self) -> Result<Box<dyn Stmt>, ParseError> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        Ok(Box::new(Print::new(value)))
    }

    fn expression_statement(&mut self) -> Result<Box<dyn Stmt>, ParseError> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after expression.")?;
        Ok(Box::new(Expression::new(expr)))
    }

    fn if_statement(&mut self) -> Result<Box<dyn Stmt>, ParseError> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after condition.")?;

        let then_branch = self.statement()?;
        let mut else_branch = None;
        if self.match_types(vec![TokenType::Else]) {
            else_branch = Some(self.statement()?);
        }
        Ok(Box::new(If::new(condition, then_branch, else_branch)))
    }

    fn while_statement(&mut self) -> Result<Box<dyn Stmt>, ParseError> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after condition.")?;
        let body = self.statement()?;

        Ok(Box::new(While::new(condition, body)))
    }

    fn block(&mut self) -> Result<Vec<Box<dyn Stmt>>, ParseError> {
        let mut statements = Vec::new();

        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration());
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.")?;
        Ok(statements)
    }

    fn equality(&mut self) -> Result<Box<dyn Expr>, ParseError> {
        let mut expr = self.comparison()?;

        while self.match_types(vec![TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous();
            let right = self.comparison()?;
            expr = Box::new(Binary::new(expr, operator, right));
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Box<dyn Expr>, ParseError> {
        let mut expr = self.term()?;

        while self.match_types(vec![
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous();
            let right = self.term()?;
            expr = Box::new(Binary::new(expr, operator, right));
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Box<dyn Expr>, ParseError> {
        let mut expr = self.factor()?;

        while self.match_types(vec![TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous();
            let right = self.factor()?;
            expr = Box::new(Binary::new(expr, operator, right));
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Box<dyn Expr>, ParseError> {
        let mut expr = self.unary()?;

        while self.match_types(vec![TokenType::Slash, TokenType::Star]) {
            let operator = self.previous();
            let right = self.unary()?;
            expr = Box::new(Binary::new(expr, operator, right));
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Box<dyn Expr>, ParseError> {
        if self.match_types(vec![TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            let right = self.unary()?;
            return Ok(Box::new(Unary::new(operator, right)));
        }
        self.call()
    }

    fn call(&mut self) -> Result<Box<dyn Expr>, ParseError> {
        let mut expr = self.primary()?;
        loop {
            if self.match_types(vec![TokenType::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn finish_call(&mut self, callee: Box<dyn Expr>) -> Result<Box<dyn Expr>, ParseError> {
        let mut arguments = Vec::new();
        if !self.check(TokenType::RightParen) {
            loop {
                if arguments.len() >= 255 {
                    return Err(ParseError::new("Can't have more than 255 arguments."));
                }
                arguments.push(self.expression()?);
                if !self.match_types(vec![TokenType::Comma]) {
                    break;
                }
            }
        }
        let paren = self.consume(TokenType::RightParen, "Expect ')' after arguments.")?;
        Ok(Box::new(Call::new(callee, paren, arguments)))
    }

    fn primary(&mut self) -> Result<Box<dyn Expr>, ParseError> {
        if self.match_types(vec![TokenType::False]) {
            return Ok(Box::new(Literal::new(Some(LoxType::new_bool(false)))));
        } else if self.match_types(vec![TokenType::True]) {
            return Ok(Box::new(Literal::new(Some(LoxType::new_bool(true)))));
        } else if self.match_types(vec![TokenType::Nil]) {
            return Ok(Box::new(Literal::new(None)));
        } else if self.match_types(vec![TokenType::Number, TokenType::String]) {
            return Ok(Box::new(Literal::new(Some(
                self.previous().literal.clone().unwrap(),
            ))));
        } else if self.match_types(vec![TokenType::Identifier]) {
            return Ok(Box::new(Variable::new(self.previous())));
        } else if self.match_types(vec![TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expect ')' after expression.")?;
            return Ok(Box::new(Grouping::new(expr)));
        }
        Err(create_parse_error(self.peek(), "Expect expression."))
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

    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<Token, ParseError> {
        if self.check(token_type) {
            return Ok(self.advance());
        }
        let err_message = format!("Parser consume error: {}", message);
        return Err(create_parse_error(self.peek(), err_message.as_str()));
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
        let mut expr = parser.expression().unwrap();
        println!("{:?}", expr);

        let mut printer = PrintExprVisitor;
        expr.accept(&mut printer);
        println!();
    }
}
