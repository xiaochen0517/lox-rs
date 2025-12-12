use std::fmt::Debug;

use crate::scanner::token::Token;

pub trait Expr {
    fn accept<R, V: ExprVisitor<R>>(&self, visitor: &V) -> R;
}

macro_rules! expr_impl {
    (
        $(
            $struct_name:ident<$($generic_param:ident$(: $generic_bound:path)?),* $(,)?>($visitor_fn:ident) {
                $($field_name:ident : $field_type:ty),* $(,)?
            }
        ),* $(,)?
    ) => {

        pub trait ExprVisitor<ER> {
            $(
                fn $visitor_fn<$($generic_param$(: $generic_bound)?),*>(&self, expr: &$struct_name<$($generic_param),*>) -> ER;
            )*
        }

        $(

            #[derive(Debug)]
            pub struct $struct_name<$($generic_param$(: $generic_bound)?),*> {
                $(pub $field_name: $field_type),*
            }

            impl<$($generic_param$(: $generic_bound)?),*> Expr for $struct_name<$($generic_param),*> {
                fn accept<ER, V: ExprVisitor<ER>>(&self, visitor: &V) -> ER {
                    visitor.$visitor_fn(self)
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
    Binary<L: Expr, R: Expr>(binary_visite) {
        left: Box<L>,
        operator: Token,
        right: Box<R>,
    },
    Grouping<T: Expr>(grouping_visite) {
        expression: Box<T>,
    },
    Literal<T: Debug>(literal_visite) {
        value: Option<T>,
    },
    Unary<T: Expr>(unary_visite) {
        operator: Token,
        right: Box<T>,
    },
}

struct PrintExprVisitor;

impl ExprVisitor<String> for PrintExprVisitor {

    fn binary_visite<L: Expr, R: Expr>(&self, expr: &Binary<L, R>) -> String {
        format!(
            "({} {} {})",
            expr.operator.lexeme,
            expr.left.accept(self),
            expr.right.accept(self)
        )
    }

    fn grouping_visite<T:Expr>(&self,expr: &Grouping<T>) -> String {
        format!("(group {})", expr.expression.accept(self))
    }

    fn literal_visite<T:Debug>(&self,expr: &Literal<T>) -> String {
        match &expr.value {
            Some(v) => format!("{:?}", v),
            None => "nil".to_string(),
        }
    }

    fn unary_visite<T:Expr>(&self,expr: &Unary<T>) -> String {
        format!("({} {})", expr.operator.lexeme, expr.right.accept(self))
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

        let printer = PrintExprVisitor;
        let result = binary_expr.accept(&printer);
        println!("Result: {}", result);
        assert_eq!(result, "(+ \"1\" \"2\")");
    }
}
