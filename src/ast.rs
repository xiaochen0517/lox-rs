use std::hash::Hasher;
pub mod interpreter;
mod macros;
pub mod resolver;

use paste::paste;
use std::fmt::Debug;
use std::hash::DefaultHasher;
use std::hash::Hash;

use crate::generate_ast;
use crate::scanner::LoxType;
use crate::scanner::token::{LoxReturn, Token};

/*pub trait SimpleHash {
    fn get_hash(&self) -> String;
}

impl SimpleHash for String {
    fn get_hash(&self) -> String {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        format!("{}", hasher.finish())
    }
}

impl<T: SimpleHash> SimpleHash for Box<T> {
    fn get_hash(&self) -> String {
        (**self).get_hash()
    }
}

impl<T: SimpleHash> SimpleHash for Option<T> {
    fn get_hash(&self) -> String {
        match self {
            Some(value) => value.get_hash(),
            None => "None".to_string(),
        }
    }
}

impl<T: SimpleHash> SimpleHash for Option<Box<T>> {
    fn get_hash(&self) -> String {
        match self {
            Some(value) => value.get_hash(),
            None => "None".to_string(),
        }
    }
}

impl<T: SimpleHash> SimpleHash for Vec<T> {
    fn get_hash(&self) -> String {
        let mut hash_str = String::new();
        for item in self {
            hash_str += &item.get_hash();
        }
        hash_str
    }
}*/

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
        Class(class_visit) {
            name: Token,
            methods: Vec<Box<dyn Stmt>>,
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

#[allow(unused)]
pub struct PrintExprVisitor;

impl ExprVisitor for PrintExprVisitor {
    fn assign_visit(&mut self, _expr: &Assign) -> Result<Option<LoxType>, LoxReturn> {
        todo!()
    }

    fn binary_visit(&mut self, expr: &Binary) -> Result<Option<LoxType>, LoxReturn> {
        print!("([binary] ");
        let _ = expr.left.accept(self);
        print!(" {} ", expr.operator.lexeme);
        let _ = expr.right.accept(self);
        print!(")");
        Ok(None)
    }

    fn grouping_visit(&mut self, expr: &Grouping) -> Result<Option<LoxType>, LoxReturn> {
        print!("([group] ");
        let _ = expr.expression.accept(self);
        print!(")");
        Ok(None)
    }

    fn literal_visit(&mut self, _expr: &Literal) -> Result<Option<LoxType>, LoxReturn> {
        Ok(None)
    }

    fn logical_visit(&mut self, _expr: &Logical) -> Result<Option<LoxType>, LoxReturn> {
        todo!()
    }

    fn unary_visit(&mut self, expr: &Unary) -> Result<Option<LoxType>, LoxReturn> {
        print!("([unary] {} ", expr.operator.lexeme);
        let _ = expr.right.accept(self);
        print!(")");
        Ok(None)
    }

    fn variable_visit(&mut self, _expr: &Variable) -> Result<Option<LoxType>, LoxReturn> {
        todo!()
    }

    fn call_visit(&mut self, _expr: &Call) -> Result<Option<LoxType>, LoxReturn> {
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
