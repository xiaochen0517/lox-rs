pub mod interpreter;
mod macros;

use paste::paste;
use std::fmt::Debug;

use crate::generate_ast;
use crate::scanner::LoxType;
use crate::scanner::token::Token;

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
        Unary(unary_visit) {
            operator: Token,
            right: Box<dyn Expr>,
        },
        Variable(variable_visit) {
            name: Token,
        }
    },
    Stmt {
        Print(print_visit) {
            expression: Box<dyn Expr>,
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
        }
    },
}

pub struct PrintExprVisitor;

impl ExprVisitor for PrintExprVisitor {
    fn assign_visit(&mut self, expr: &Assign) -> Option<LoxType> {
        todo!()
    }

    fn binary_visit(&mut self, expr: &Binary) -> Option<LoxType> {
        print!("([binary] ");
        expr.left.accept(self);
        print!(" {} ", expr.operator.lexeme);
        expr.right.accept(self);
        print!(")");
        return None;
    }

    fn grouping_visit(&mut self, expr: &Grouping) -> Option<LoxType> {
        print!("([group] ");
        expr.expression.accept(self);
        print!(")");
        return None;
    }

    fn literal_visit(&mut self, expr: &Literal) -> Option<LoxType> {
        return None;
    }

    fn unary_visit(&mut self, expr: &Unary) -> Option<LoxType> {
        print!("([unary] {} ", expr.operator.lexeme);
        expr.right.accept(self);
        print!(")");
        return None;
    }

    fn variable_visit(&mut self, expr: &Variable) -> Option<LoxType> {
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
