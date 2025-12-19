use crate::ast::{Assign, Block, If, Logical, Var, Variable, While};
use crate::environment::Environment;
use crate::{
    ast::{
        Binary, Expr, ExprVisitor, Expression, Grouping, Literal, Print, Stmt, StmtVisitor, Unary,
    },
    scanner::{LoxType, Token, TokenType},
};
use std::any::Any;
use std::cell::RefCell;
use std::mem;
use std::rc::Rc;
use unescape::unescape;

#[derive(Debug)]
pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            environment: Rc::new(RefCell::new(Environment::new())),
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
        let new_rc_environment = Rc::new(RefCell::new(environment));
        let original_env = mem::replace(&mut self.environment, new_rc_environment);
        for statement in statements {
            self.execute(statement);
        }
        self.environment = original_env;
    }

    fn evaluate(&mut self, expr: &dyn Expr) -> Option<LoxType> {
        expr.accept(self)
    }

    fn is_truthy(&self, value: &Option<LoxType>) -> bool {
        match value {
            None => true,
            Some(lox_type) => match lox_type {
                LoxType::Str(str) => str.len() > 0,
                LoxType::Num(num) => **num != 0.0,
                LoxType::Bool(boolean) => boolean.as_ref().clone(),
            },
        }
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

    fn compare_numbers<F>(
        &self,
        left: Option<LoxType>,
        right: Option<LoxType>,
        compare: F,
    ) -> Option<LoxType>
    where
        F: FnOnce(f64, f64) -> bool,
    {
        self.panic_none_or_nil(vec![&left, &right]);
        match (left.unwrap(), right.unwrap()) {
            (LoxType::Num(left), LoxType::Num(right)) => {
                Some(LoxType::new_bool(compare(*left, *right)))
            }
            _ => panic!("Operand must be numbers"),
        }
    }

    fn calculate_number<F>(
        &self,
        left: Option<LoxType>,
        right: Option<LoxType>,
        calculate: F,
    ) -> Option<LoxType>
    where
        F: FnOnce(f64, f64) -> f64,
    {
        self.panic_none_or_nil(vec![&left, &right]);
        match (left.unwrap(), right.unwrap()) {
            (LoxType::Num(left), LoxType::Num(right)) => {
                Some(LoxType::new_num(calculate(*left, *right)))
            }
            _ => panic!("Operand must be numbers"),
        }
    }
}

impl ExprVisitor for Interpreter {
    fn assign_visit(&mut self, expr: &Assign) -> Option<LoxType> {
        let value = self.evaluate(expr.value.as_ref());

        self.environment
            .borrow_mut()
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
            TokenType::Minus => self.calculate_number(left, right, |left, right| left - right),
            TokenType::Star => self.calculate_number(left, right, |left, right| left * right),
            TokenType::Slash => self.calculate_number(left, right, |left, right| {
                if right == 0.0 {
                    panic!("Division by zero.");
                }
                left / right
            }),
            // Comparison operators
            TokenType::Greater => self.compare_numbers(left, right, |left, right| left > right),
            TokenType::GreaterEqual => {
                self.compare_numbers(left, right, |left, right| left >= right)
            }
            TokenType::Less => self.compare_numbers(left, right, |left, right| left < right),
            TokenType::LessEqual => self.compare_numbers(left, right, |left, right| left <= right),
            TokenType::BangEqual => Some(LoxType::new_bool(!self.is_equal(left, right))),
            TokenType::EqualEqual => Some(LoxType::new_bool(self.is_equal(left, right))),
            _ => None,
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

    fn logical_visit(&mut self, expr: &Logical) -> Option<LoxType> {
        let left = self.evaluate(expr.left.as_ref());

        if expr.operator.token_type == TokenType::Or {
            if self.is_truthy(&left) {
                return left;
            }
        } else {
            if !self.is_truthy(&left) {
                return left;
            }
        }

        self.evaluate(expr.right.as_ref())
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
            _ => None,
        }
    }

    fn variable_visit(&mut self, expr: &Variable) -> Option<LoxType> {
        self.environment
            .borrow()
            .get(expr.name.lexeme.as_str())
            .clone()
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

    fn if_visit(&mut self, stmt: &If) -> Option<LoxType> {
        let condition_result = self.evaluate(stmt.condition.as_ref());
        if self.is_truthy(&condition_result) {
            self.execute(&stmt.then_branch);
            return None;
        }
        if let Some(else_branch) = stmt.else_branch.as_ref() {
            self.execute(else_branch);
        }
        None
    }

    fn block_visit(&mut self, stmt: &Block) -> Option<LoxType> {
        self.execute_block(
            &stmt.statements,
            Environment::new_with_enclosing(Rc::clone(&self.environment)),
        );
        None
    }

    fn expression_visit(&mut self, stmt: &Expression) -> Option<LoxType> {
        self.evaluate(stmt.expression.as_ref());
        None
    }

    fn var_visit(&mut self, stmt: &Var) -> Option<LoxType> {
        let value = self.evaluate(stmt.initializer.as_ref());
        self.environment
            .borrow_mut()
            .define(stmt.name.lexeme.clone(), value);
        None
    }

    fn while_visit(&mut self, stmt: &While) -> Option<LoxType> {
        let mut condition_result = self.evaluate(stmt.condition.as_ref());
        while self.is_truthy(&condition_result) {
            self.execute(&stmt.body);
            condition_result = self.evaluate(stmt.condition.as_ref());
        }
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
