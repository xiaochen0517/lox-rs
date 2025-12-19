use crate::ast::{Assign, Block, Var, Variable};
use crate::environment::Environment;
use crate::{
    ast::{
        Binary, Expr, ExprVisitor, Expression, Grouping, Literal, Print, Stmt, StmtVisitor, Unary,
    },
    scanner::{LoxType, Token, TokenType},
};
use std::any::Any;
use std::mem;
use unescape::unescape;

#[derive(Debug)]
pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            environment: Environment::new(),
        }
    }

    pub fn interpret(&mut self, statements: &Vec<Box<dyn Stmt>>) {
        for statement in statements {
            self.execute(statement);
        }
    }

    fn execute(&mut self, stmt: &Box<dyn Stmt>) {
        stmt.accept(self);
    }

    fn execute_block(&mut self, statements: &Vec<Box<dyn Stmt>>, environment: Environment) {
        let original_env = mem::replace(&mut self.environment, environment);
        for statement in statements {
            self.execute(statement);
        }
        self.environment = original_env;
    }

    fn evaluate(&mut self, expr: &dyn Expr) -> Option<LoxType> {
        expr.accept(self)
    }

    fn is_truthy(&self, value: &dyn Any) -> bool {
        if let Some(bool_value) = value.downcast_ref::<bool>() {
            return *bool_value;
        }
        if value.is::<()>() {
            return false;
        }
        true
    }

    fn panic_none_or_nil(&self, lists: Vec<&Option<LoxType>>) {
        for item in lists {
            if item.is_none() {
                panic!("Operand must not be nil.");
            }
        }
    }

    fn is_equal(&self, a: Option<LoxType>, b: Option<LoxType>) -> bool {
        match (a, b) {
            (None, None) => true,
            (Some(_), None) | (None, Some(_)) => false,
            (Some(val_a), Some(val_b)) => val_a == val_b,
        }
    }

    fn check_number_operand(&self, operator: &Token, operand: &Option<LoxType>) {
        if let Some(LoxType::Num(_)) = operand {
            return;
        }
        panic!("Operand must be a number for operator {:?}", operator);
    }
}

impl ExprVisitor for Interpreter {
    fn assign_visit(&mut self, expr: &Assign) -> Option<LoxType> {
        let value = self.evaluate(expr.value.as_ref());

        self.environment
            .assign(expr.name.lexeme.clone(), value.clone())
            .unwrap_or_else(|err| {
                panic!("{}", err);
            });
        value
    }

    fn binary_visit(&mut self, expr: &Binary) -> Option<LoxType> {
        println!("Visiting Binary Expression: {:?}", expr);
        let left = self.evaluate(expr.left.as_ref());
        let right = self.evaluate(expr.right.as_ref());
        // if left.is_none() || right.is_none() {
        //     panic!("Operands must not be nil.");
        // }
        // let left = left.unwrap();
        // let right = right.unwrap();

        match expr.operator.token_type {
            TokenType::Plus => {
                self.panic_none_or_nil(vec![&left, &right]);
                match (left.unwrap(), right.unwrap()) {
                    (LoxType::Str(left_str), LoxType::Str(right_str)) => {
                        return Some(LoxType::Str(Box::new(format!(
                            "{}{}",
                            *left_str, *right_str
                        ))));
                    }
                    (LoxType::Num(left_num), LoxType::Num(right_str)) => {
                        return Some(LoxType::Num(Box::new(*left_num + *right_str)));
                    }
                    // 一侧为字符串，另一侧为数字时，进行字符串拼接
                    (LoxType::Str(left_str), LoxType::Num(right_num)) => {
                        return Some(LoxType::Str(Box::new(format!(
                            "{}{}",
                            *left_str, *right_num
                        ))));
                    }
                    (LoxType::Num(left_num), LoxType::Str(right_str)) => {
                        return Some(LoxType::Str(Box::new(format!(
                            "{}{}",
                            *left_num, *right_str
                        ))));
                    }
                    _ => {
                        panic!("Operands must be numbers or strings.");
                    }
                }
            }
            TokenType::Minus => {
                self.panic_none_or_nil(vec![&left, &right]);
                match (left.unwrap(), right.unwrap()) {
                    (LoxType::Num(left), LoxType::Num(right)) => {
                        return Some(LoxType::new_num(*left - *right));
                    }
                    _ => {
                        panic!("Operands must be numbers.");
                    }
                }
            }
            TokenType::Star => {
                self.panic_none_or_nil(vec![&left, &right]);
                match (left.unwrap(), right.unwrap()) {
                    (LoxType::Num(left), LoxType::Num(right)) => {
                        return Some(LoxType::new_num(*left * *right));
                    }
                    _ => {
                        panic!("Operands must be numbers.");
                    }
                }
            }
            TokenType::Slash => {
                self.panic_none_or_nil(vec![&left, &right]);
                match (left.unwrap(), right.unwrap()) {
                    (LoxType::Num(left_num), LoxType::Num(right_num)) => {
                        if *right_num == 0.0 {
                            panic!("Division by zero.");
                        }
                        return Some(LoxType::new_num(*left_num / *right_num));
                    }
                    _ => {
                        panic!("Operands must be numbers.");
                    }
                }
            }
            // Comparison operators
            TokenType::Greater => {
                self.panic_none_or_nil(vec![&left, &right]);
                match (left.unwrap(), right.unwrap()) {
                    (LoxType::Num(left), LoxType::Num(right)) => {
                        return Some(LoxType::new_bool(*left > *right));
                    }
                    _ => {
                        panic!("Operands must be numbers.");
                    }
                }
            }
            TokenType::GreaterEqual => {
                self.panic_none_or_nil(vec![&left, &right]);
                match (left.unwrap(), right.unwrap()) {
                    (LoxType::Num(left), LoxType::Num(right)) => {
                        return Some(LoxType::new_bool(*left >= *right));
                    }
                    _ => {
                        panic!("Operands must be numbers.");
                    }
                }
            }
            TokenType::Less => {
                self.panic_none_or_nil(vec![&left, &right]);
                match (left.unwrap(), right.unwrap()) {
                    (LoxType::Num(left), LoxType::Num(right)) => {
                        return Some(LoxType::new_bool(*left < *right));
                    }
                    _ => {
                        panic!("Operands must be numbers.");
                    }
                }
            }
            TokenType::LessEqual => {
                self.panic_none_or_nil(vec![&left, &right]);
                match (left.unwrap(), right.unwrap()) {
                    (LoxType::Num(left), LoxType::Num(right)) => {
                        return Some(LoxType::new_bool(*left <= *right));
                    }
                    _ => {
                        panic!("Operands must be numbers.");
                    }
                }
            }
            TokenType::BangEqual => {
                return Some(LoxType::new_bool(!self.is_equal(left, right)));
            }
            TokenType::EqualEqual => {
                return Some(LoxType::new_bool(self.is_equal(left, right)));
            }
            _ => {
                return None;
            }
        }
    }

    fn grouping_visit(&mut self, expr: &Grouping) -> Option<LoxType> {
        println!("Visiting Grouping Expression: {:?}", expr);
        expr.expression.accept(self)
    }

    fn literal_visit(&mut self, expr: &Literal) -> Option<LoxType> {
        println!("Visiting Literal Expression: {:?}", expr);
        expr.value.clone()
    }

    fn unary_visit(&mut self, expr: &Unary) -> Option<LoxType> {
        println!("Visiting Unary Expression: {:?}", expr);
        let right = self.evaluate(expr.right.as_ref());

        match expr.operator.token_type {
            TokenType::Minus => {
                if let Some(LoxType::Num(num)) = right {
                    Some(LoxType::new_num(-*num))
                } else {
                    panic!("Operand must be a number.");
                }
            }
            _ => {
                None
            }
        }
    }

    fn variable_visit(&mut self, expr: &Variable) -> Option<LoxType> {
        self.environment.get(expr.name.lexeme.as_str()).clone()
    }
}

impl StmtVisitor for Interpreter {
    fn print_visit(&mut self, stmt: &Print) -> Option<LoxType> {
        let value = self.evaluate(stmt.expression.as_ref());
        match value {
            Some(v) => match v {
                LoxType::Str(s) => match unescape(&*s.as_str()) {
                    Some(unescaped_str) => print!("{}", unescaped_str),
                    None => print!("{}", *s),
                },
                LoxType::Num(n) => {
                    print!("{}", *n);
                }
                LoxType::Bool(b) => {
                    print!("{}", *b);
                }
            },
            None => {
                print!("<nil>");
            }
        }
        None
    }

    fn block_visit(&mut self, stmt: &Block) -> Option<LoxType> {
        self.execute_block(&stmt.statements, Environment::new());
        None
    }

    fn expression_visit(&mut self, stmt: &Expression) -> Option<LoxType> {
        self.evaluate(stmt.expression.as_ref());
        None
    }

    fn var_visit(&mut self, stmt: &Var) -> Option<LoxType> {
        let value = self.evaluate(stmt.initializer.as_ref());
        self.environment.define(stmt.name.lexeme.clone(), value);
        None
    }
}

#[cfg(test)]
mod test {
    use std::sync::OnceLock;

    use super::*;

    fn get_number_one() -> Box<Literal> {
        Box::new(Literal::new(Some(LoxType::new_num(1.0))))
    }

    fn get_number_two() -> Box<Literal> {
        Box::new(Literal::new(Some(LoxType::new_num(2.0))))
    }

    #[test]
    fn test_interpreter_plus() {
        // let left = get_number_one();
        // let right = get_number_two();
        // let plus_operator = Token::new(TokenType::Plus, "+".to_string(), 1, 2, 2, None);
        // let binary_expr = Binary::new(left, plus_operator, right);

        // let mut interpreter = Interpreter::new();
        // interpreter.interpret(&binary_expr);
    }
}
