use std::fmt::Debug;

use crate::scanner::token::Token;

macro_rules! expr_impl {
    (
        $(
            $struct_name:ident($visitor_fn:ident) {
                $($field_name:ident : $field_type:ty),* $(,)?
            }
        ),* $(,)?
    ) => {

        pub enum ExprType {
            $(
                $struct_name,
            )*
        }

        pub trait ExprVisitor {
            $(
                fn $visitor_fn(&mut self, expr: &$struct_name);
            )*
        }

        pub trait Expr:Debug {
            fn accept(&self, visitor: &mut dyn ExprVisitor);
            fn get_type(&self) -> ExprType;
        }

        $(

            #[derive(Debug)]
            pub struct $struct_name {
                $(pub $field_name: $field_type),*
            }

            impl Expr for $struct_name {

                fn accept(&self, visitor: &mut dyn ExprVisitor) {
                    visitor.$visitor_fn(self)
                }

                fn get_type(&self) -> ExprType {
                    ExprType::$struct_name
                }
            }

            impl $struct_name {
                pub fn new($($field_name : $field_type),*) -> Self {
                    $struct_name { $($field_name),* }
                }
            }
        )*
    };
}

expr_impl! {
    Binary(binary_visite) {
        left: Box<dyn Expr>,
        operator: Token,
        right: Box<dyn Expr>,
    },
    Grouping(grouping_visite) {
        expression: Box<dyn Expr>,
    },
    Literal(literal_visite) {
        value: Option<String>,
    },
    Unary(unary_visite) {
        operator: Token,
        right: Box<dyn Expr>,
    },
}

pub struct PrintExprVisitor;

impl ExprVisitor for PrintExprVisitor {
    fn binary_visite(&mut self, expr: &Binary) {
        print!("([binary] ");
        expr.left.accept(self);
        print!(" {} ", expr.operator.lexeme);
        expr.right.accept(self);
        print!(")");
    }

    fn grouping_visite(&mut self, expr: &Grouping) {
        print!("([group] ");
        expr.expression.accept(self);
        print!(")");
    }

    fn literal_visite(&mut self, expr: &Literal) {
        match &expr.value {
            Some(v) => print!("({:?})", v),
            None => print!("(nil)"),
        }
    }

    fn unary_visite(&mut self, expr: &Unary) {
        print!("([unary] {} ", expr.operator.lexeme);
        expr.right.accept(self);
        print!(")");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_expr() {
        let left = Box::new(Literal::new(Some("1".to_string())));
        let right = Box::new(Literal::new(Some("2".to_string())));
        let operator = Token::new(
            crate::scanner::token::TokenType::Plus,
            "+".to_string(),
            1,
            None,
        );
        let binary_expr = Binary::new(left, operator, right);
        println!("{:?}", binary_expr);

        let mut printer = PrintExprVisitor;
        binary_expr.accept(&mut printer);
        println!();

        assert_eq!(
            format!("{:?}", binary_expr.left),
            "Literal { value: Some(\"1\") }"
        );
        assert_eq!(
            format!("{:?}", binary_expr.right),
            "Literal { value: Some(\"2\") }"
        );
    }
}
