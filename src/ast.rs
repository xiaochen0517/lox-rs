pub mod interpreter;
mod macros;

use paste::paste;
use std::fmt::Debug;

use crate::generate_ast;
use crate::scanner::LoxType;
use crate::scanner::token::{LoxReturn, Token};

generate_ast! {
    Expr {
        Assign(assign_visit) {
            name: Token,
            value: Box<dyn Expr>,
        },
        Binary(binary_visit) {
            left: Box<dyn Expr>,
            operator: Token,
            right: Box<dyn Expr>,
        },
        Grouping(grouping_visit) {
            expression: Box<dyn Expr>,
        },
        Literal(literal_visit) {
            value: Option<LoxType>,
        },
        Logical(logical_visit) {
            left: Box<dyn Expr>,
            operator: Token,
            right: Box<dyn Expr>,
        },
        Unary(unary_visit) {
            operator: Token,
            right: Box<dyn Expr>,
        },
        Variable(variable_visit) {
            name: Token,
        },
        Call(call_visit) {
            callee: Box<dyn Expr>,
            paren: Token,
            arguments: Vec<Box<dyn Expr>>,
        }
    },
    Stmt {
        Print(print_visit) {
            expression: Box<dyn Expr>,
        },
        If(if_visit) {
            condition: Box<dyn Expr>,
            then_branch: Box<dyn Stmt>,
            else_branch: Option<Box<dyn Stmt>>,
        },
        Block(block_visit) {
            statements: Vec<Box<dyn Stmt>>,
        },
        Expression(expression_visit) {
            expression: Box<dyn Expr>,
        },
        Var(var_visit) {
            name: Token,
            initializer: Box<dyn Expr>
        },
        While(while_visit) {
            condition: Box<dyn Expr>,
            body: Box<dyn Stmt>
        },
        Function(function_visit) {
            name: Token,
            params: Vec<Token>,
            body: Vec<Box<dyn Stmt>>,
        },
        Return(return_visit) {
            keyword: Token,
            value: Option<Box<dyn Expr>>,
        }
    },
}

pub struct PrintExprVisitor;

impl ExprVisitor for PrintExprVisitor {
    fn assign_visit(&mut self, expr: &Assign) -> Result<Option<LoxType>, LoxReturn> {
        todo!()
    }

    fn binary_visit(&mut self, expr: &Binary) -> Result<Option<LoxType>, LoxReturn> {
        print!("([binary] ");
        expr.left.accept(self);
        print!(" {} ", expr.operator.lexeme);
        expr.right.accept(self);
        print!(")");
        Ok(None)
    }

    fn grouping_visit(&mut self, expr: &Grouping) -> Result<Option<LoxType>, LoxReturn> {
        print!("([group] ");
        expr.expression.accept(self);
        print!(")");
        Ok(None)
    }

    fn literal_visit(&mut self, expr: &Literal) -> Result<Option<LoxType>, LoxReturn> {
        Ok(None)
    }

    fn logical_visit(&mut self, expr: &Logical) -> Result<Option<LoxType>, LoxReturn> {
        todo!()
    }

    fn unary_visit(&mut self, expr: &Unary) -> Result<Option<LoxType>, LoxReturn> {
        print!("([unary] {} ", expr.operator.lexeme);
        expr.right.accept(self);
        print!(")");
        Ok(None)
    }

    fn variable_visit(&mut self, expr: &Variable) -> Result<Option<LoxType>, LoxReturn> {
        todo!()
    }

    fn call_visit(&mut self, expr: &Call) -> Result<Option<LoxType>, LoxReturn> {
        todo!()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn test_expr() {
        let left = Box::new(Literal::new(Some(LoxType::new_str("1"))));
        let right = Box::new(Literal::new(Some(LoxType::new_str("2"))));
        let operator = Token::new(
            crate::scanner::token::TokenType::Plus,
            "+".to_string(),
            1,
            2,
            2,
            None,
        );
        let binary_expr = Binary::new(left, operator, right);
        println!("{:?}", binary_expr);

        let mut printer = PrintExprVisitor;
        binary_expr.accept(&mut printer);
        println!();

        assert_eq!(
            format!("{:?}", binary_expr.left),
            "Literal { value: Some(Str(\"1\")) }"
        );
        assert_eq!(
            format!("{:?}", binary_expr.right),
            "Literal { value: Some(Str(\"2\")) }"
        );
    }
}
