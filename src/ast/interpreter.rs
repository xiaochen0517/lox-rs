use crate::ast::{Assign, Block, Call, Function, If, Logical, Return, Var, Variable, While};
use crate::environment::Environment;
use crate::function::LoxFunction;
use crate::function::native::ClockNativeFunction;
use crate::scanner::token::LoxReturn;
use crate::{
    ast::{
        Binary, Expr, ExprVisitor, Expression, Grouping, Literal, Print, Stmt, StmtVisitor, Unary,
    },
    log_info,
    scanner::{LoxType, Token, TokenType},
};
use maplit::hashmap;
use std::cell::RefCell;
use std::mem;
use std::rc::Rc;
use unescape::unescape;

#[derive(Debug)]
pub struct Interpreter {
    pub globals: Rc<RefCell<Environment>>,
    pub environment: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        let globals = Rc::new(RefCell::new(Environment::new_with_values(hashmap! {
            "clock".to_string() => Some(LoxType::Function(Box::new(
                ClockNativeFunction::new()
            ))),
        })));
        Interpreter {
            globals: Rc::clone(&globals),
            environment: Rc::new(RefCell::new(Environment::new())),
        }
    }

    pub fn interpret(&mut self, statements: &Vec<Box<dyn Stmt>>) {
        for statement in statements {
            self.execute(statement);
        }
    }

    fn execute(&mut self, stmt: &Box<dyn Stmt>) -> Result<Option<LoxType>, LoxReturn> {
        stmt.accept(self)
    }

    pub fn execute_block(
        &mut self,
        statements: &Vec<Box<dyn Stmt>>,
        environment: Environment,
    ) -> Result<(), LoxReturn> {
        let new_rc_environment = Rc::new(RefCell::new(environment));
        let original_env = mem::replace(&mut self.environment, new_rc_environment);
        for statement in statements {
            self.execute(statement)?;
        }
        self.environment = original_env;
        Ok(())
    }

    fn evaluate(&mut self, expr: &dyn Expr) -> Result<Option<LoxType>, LoxReturn> {
        expr.accept(self)
    }

    fn is_truthy(&self, value: &Option<LoxType>) -> bool {
        match value {
            None => true,
            Some(lox_type) => match lox_type {
                LoxType::Str(str) => str.len() > 0,
                LoxType::Num(num) => **num != 0.0,
                LoxType::Bool(boolean) => boolean.as_ref().clone(),
                LoxType::Function(function) => {
                    panic!("Cannot evaluate truthiness of function.");
                }
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
    ) -> Result<Option<LoxType>, LoxReturn>
    where
        F: FnOnce(f64, f64) -> bool,
    {
        self.panic_none_or_nil(vec![&left, &right]);
        match (left.unwrap(), right.unwrap()) {
            (LoxType::Num(left), LoxType::Num(right)) => {
                Ok(Some(LoxType::new_bool(compare(*left, *right))))
            }
            _ => panic!("Operand must be numbers"),
        }
    }

    fn calculate_number<F>(
        &self,
        left: Option<LoxType>,
        right: Option<LoxType>,
        calculate: F,
    ) -> Result<Option<LoxType>, LoxReturn>
    where
        F: FnOnce(f64, f64) -> f64,
    {
        self.panic_none_or_nil(vec![&left, &right]);
        match (left.unwrap(), right.unwrap()) {
            (LoxType::Num(left), LoxType::Num(right)) => {
                Ok(Some(LoxType::new_num(calculate(*left, *right))))
            }
            _ => panic!("Operand must be numbers"),
        }
    }
}

impl ExprVisitor for Interpreter {
    fn assign_visit(&mut self, expr: &Assign) -> Result<Option<LoxType>, LoxReturn> {
        let value = self.evaluate(expr.value.as_ref())?;

        self.environment
            .borrow_mut()
            .assign(expr.name.lexeme.clone(), value.clone())
            .unwrap_or_else(|err| {
                panic!("{}", err);
            });
        Ok(value)
    }

    fn binary_visit(&mut self, expr: &Binary) -> Result<Option<LoxType>, LoxReturn> {
        log_info!("Visiting Binary Expression: {:?}", expr);
        let left = self.evaluate(expr.left.as_ref())?;
        let right = self.evaluate(expr.right.as_ref())?;

        match expr.operator.token_type {
            TokenType::Plus => {
                self.panic_none_or_nil(vec![&left, &right]);
                match (left.unwrap(), right.unwrap()) {
                    (LoxType::Str(left_str), LoxType::Str(right_str)) => {
                        return Ok(Some(LoxType::Str(Box::new(format!(
                            "{}{}",
                            *left_str, *right_str
                        )))));
                    }
                    (LoxType::Num(left_num), LoxType::Num(right_str)) => {
                        return Ok(Some(LoxType::Num(Box::new(*left_num + *right_str))));
                    }
                    // 一侧为字符串，另一侧为数字时，进行字符串拼接
                    (LoxType::Str(left_str), LoxType::Num(right_num)) => {
                        return Ok(Some(LoxType::Str(Box::new(format!(
                            "{}{}",
                            *left_str, *right_num
                        )))));
                    }
                    (LoxType::Num(left_num), LoxType::Str(right_str)) => {
                        return Ok(Some(LoxType::Str(Box::new(format!(
                            "{}{}",
                            *left_num, *right_str
                        )))));
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
            TokenType::BangEqual => Ok(Some(LoxType::new_bool(!self.is_equal(left, right)))),
            TokenType::EqualEqual => Ok(Some(LoxType::new_bool(self.is_equal(left, right)))),
            _ => Ok(None),
        }
    }

    fn grouping_visit(&mut self, expr: &Grouping) -> Result<Option<LoxType>, LoxReturn> {
        log_info!("Visiting Grouping Expression: {:?}", expr);
        expr.expression.accept(self)
    }

    fn literal_visit(&mut self, expr: &Literal) -> Result<Option<LoxType>, LoxReturn> {
        log_info!("Visiting Literal Expression: {:?}", expr);
        Ok(expr.value.clone())
    }

    fn logical_visit(&mut self, expr: &Logical) -> Result<Option<LoxType>, LoxReturn> {
        let left = self.evaluate(expr.left.as_ref())?;

        if expr.operator.token_type == TokenType::Or {
            if self.is_truthy(&left) {
                return Ok(left);
            }
        } else {
            if !self.is_truthy(&left) {
                return Ok(left);
            }
        }

        self.evaluate(expr.right.as_ref())
    }

    fn unary_visit(&mut self, expr: &Unary) -> Result<Option<LoxType>, LoxReturn> {
        log_info!("Visiting Unary Expression: {:?}", expr);
        let right = self.evaluate(expr.right.as_ref())?;

        match expr.operator.token_type {
            TokenType::Minus => {
                if let Some(LoxType::Num(num)) = right {
                    Ok(Some(LoxType::new_num(-*num)))
                } else {
                    panic!("Operand must be a number.");
                }
            }
            _ => Ok(None),
        }
    }

    fn variable_visit(&mut self, expr: &Variable) -> Result<Option<LoxType>, LoxReturn> {
        Ok(self
            .environment
            .borrow()
            .get(expr.name.lexeme.as_str())
            .clone())
    }

    fn call_visit(&mut self, expr: &Call) -> Result<Option<LoxType>, LoxReturn> {
        let callee = self.evaluate(expr.callee.as_ref())?;
        let mut arguments = Vec::new();
        for argument in &expr.arguments {
            arguments.push(self.evaluate(argument.as_ref())?);
        }
        // 需要确保 callee 是一个函数
        if let Some(LoxType::Function(mut function)) = callee {
            // 检查调用的参数数量是否匹配
            if arguments.len() != function.arity() {
                panic!(
                    "Expected {} arguments but got {}.",
                    function.arity(),
                    arguments.len()
                );
            }
            Ok(function.call(self, &arguments))
        } else {
            panic!("Can only call functions.");
        }
    }
}

impl StmtVisitor for Interpreter {
    fn print_visit(&mut self, stmt: &Print) -> Result<Option<LoxType>, LoxReturn> {
        let value = self.evaluate(stmt.expression.as_ref())?;
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
                LoxType::Function(_) => {
                    print!("<function>");
                }
            },
            None => {
                print!("<nil>");
            }
        }
        Ok(None)
    }

    fn if_visit(&mut self, stmt: &If) -> Result<Option<LoxType>, LoxReturn> {
        let condition_result = self.evaluate(stmt.condition.as_ref())?;
        if self.is_truthy(&condition_result) {
            self.execute(&stmt.then_branch);
            return Ok(None);
        }
        if let Some(else_branch) = stmt.else_branch.as_ref() {
            self.execute(else_branch);
        }
        Ok(None)
    }

    fn block_visit(&mut self, stmt: &Block) -> Result<Option<LoxType>, LoxReturn> {
        self.execute_block(
            &stmt.statements,
            Environment::new_with_enclosing(self.environment.clone()),
        );
        Ok(None)
    }

    fn expression_visit(&mut self, stmt: &Expression) -> Result<Option<LoxType>, LoxReturn> {
        self.evaluate(stmt.expression.as_ref());
        Ok(None)
    }

    fn var_visit(&mut self, stmt: &Var) -> Result<Option<LoxType>, LoxReturn> {
        let value = self.evaluate(stmt.initializer.as_ref())?;
        self.environment
            .borrow_mut()
            .define(stmt.name.lexeme.clone(), value);
        Ok(None)
    }

    fn while_visit(&mut self, stmt: &While) -> Result<Option<LoxType>, LoxReturn> {
        let mut condition_result = self.evaluate(stmt.condition.as_ref())?;
        while self.is_truthy(&condition_result) {
            self.execute(&stmt.body);
            condition_result = self.evaluate(stmt.condition.as_ref())?;
        }
        Ok(None)
    }

    fn function_visit(&mut self, stmt: &Function) -> Result<Option<LoxType>, LoxReturn> {
        let function = LoxFunction::new(stmt.clone());
        self.environment.borrow_mut().define(
            stmt.name.lexeme.clone(),
            Some(LoxType::new_function(Box::new(function))),
        );
        Ok(None)
    }

    fn return_visit(&mut self, stmt: &Return) -> Result<Option<LoxType>, LoxReturn> {
        let mut value = None;
        if let Some(return_value) = stmt.value.as_ref() {
            value = self.evaluate(return_value.as_ref())?;
        }
        Err(LoxReturn::new(value))
    }
}

#[cfg(test)]
mod test {
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
