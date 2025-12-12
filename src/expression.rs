use std::fmt::{self, Debug};

use crate::scanner::token::Token;

pub trait Expr: fmt::Debug + fmt::Debug {
    fn accept<T: ExprVisitor>(&self, visitor: &T);
}

pub trait ExprVisitor: fmt::Debug + fmt::Debug {
    fn visit<T: Expr>(&self, expr: &T);
}

macro_rules! expr_impl {
    (
        $(
            $struct_name:ident<$($generic_param:ident$(: $generic_bound:path)?),* $(,)?> {
                $($field_name:ident : $field_type:ty),* $(,)?
            }
        ),* $(,)?
    ) => {
        $(

            #[derive(Debug)]
            pub struct $struct_name<$($generic_param$(: $generic_bound)?),*> {
                $(pub $field_name: $field_type),*
            }

            impl<$($generic_param$(: $generic_bound)?),*> Expr for $struct_name<$($generic_param),*> {
                fn accept<EV: ExprVisitor>(&self, visitor: &EV) {
                    visitor.visit(self);
                }
            }

            impl<$($generic_param$(: $generic_bound)?),*> $struct_name<$($generic_param),*> {
                pub fn new($($field_name : $field_type),*) -> Self {
                    $struct_name { $($field_name),* }
                }
            }

        )*
    };
}

expr_impl! {
    Binary<L: Expr, R: Expr> {
        left: Box<L>,
        operator: Token,
        right: Box<R>,
    },
    Grouping<T: Expr> {
        expression: Box<T>,
    },
    Literal<T: Debug> {
        value: Option<T>,
    },
    Unary<T: Expr> {
        operator: Token,
        right: Box<T>,
    },
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
    }
}
