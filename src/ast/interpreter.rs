use crate::ast::{
    Assign, Block, Call, Class, Function, Get, If, Logical, Return, Set, This, Var, Variable, While,
};
use crate::class::LoxClass;
use crate::environment::Environment;
use crate::function::LoxFunction;
use crate::function::native::ClockNativeFunction;
use crate::scanner::token::{Callable, LoxReturn, OptionLoxType};
use crate::{
    ast::{
        Binary, Expr, ExprVisitor, Expression, Grouping, Literal, Print, Stmt, StmtVisitor, Unary,
    },
    log_info,
    scanner::{LoxType, Token, TokenType},
};
use maplit::hashmap;
use std::collections::HashMap;
use std::mem;
use std::sync::{Arc, Mutex};
use unescape::unescape;

#[derive(Debug, Clone)]
struct LocalData {
    #[allow(unused)]
    expr: Box<dyn Expr>,
    depth: usize,
}

impl LocalData {
    pub fn new(expr: Box<dyn Expr>, depth: usize) -> Self {
        LocalData { expr, depth }
    }
}

#[derive(Debug, Clone)]
pub struct Interpreter {
    pub globals: Arc<Mutex<Environment>>,
    pub environment: Arc<Mutex<Environment>>,
    locals: HashMap<String, LocalData>,
}

impl Interpreter {
    pub fn new() -> Self {
        let globals = Arc::new(Mutex::new(Environment::new_with_values(hashmap! {
            "clock".to_string() => OptionLoxType::new(Some(LoxType::Function(Box::new(
                ClockNativeFunction::new()
            )))),
        })));
        Interpreter {
            globals: Arc::clone(&globals),
            environment: Arc::clone(&globals),
            locals: HashMap::new(),
        }
    }

    pub fn interpret(&mut self, statements: &Vec<Box<dyn Stmt>>) -> Result<(), LoxReturn> {
        for statement in statements {
            let _ = self.execute(statement.as_ref())?;
        }
        Ok(())
    }

    fn execute(&mut self, stmt: &dyn Stmt) -> Result<OptionLoxType, LoxReturn> {
        stmt.accept(self)
    }

    pub fn resolve<T: Expr>(&mut self, expr: &T, depth: usize) -> Result<(), LoxReturn> {
        log_info!("添加变量，深度 {}, Hash {}", depth, format!("{:?}", expr));
        self.locals.insert(
            format!("{:?}", expr),
            LocalData::new(expr.box_clone(), depth),
        );
        Ok(())
    }

    fn lookup_variable(&mut self, name: &Token, expr: &dyn Expr) -> OptionLoxType {
        log_info!("查询变量 {}, Hash {}", name.lexeme, format!("{:?}", expr));
        let distance = self.locals.get(&format!("{:?}", expr));
        log_info!("距离信息: {:?}", distance);
        if let Some(local_data) = distance {
            self.environment
                .lock()
                .unwrap()
                .get_at(local_data.depth, &name.lexeme)
        } else {
            self.globals.lock().unwrap().get(&name.lexeme)
        }
    }

    pub fn execute_block(
        &mut self,
        statements: &Vec<Box<dyn Stmt>>,
        environment: Environment,
    ) -> Result<(), LoxReturn> {
        let new_rc_environment = Arc::new(Mutex::new(environment));
        let original_env = mem::replace(&mut self.environment, new_rc_environment);
        // 不可以提前返回，需要恢复环境，所以使用 let 绑定 result
        let result = self.interpret(statements);
        self.environment = original_env;
        result
    }

    fn evaluate(&mut self, expr: &dyn Expr) -> Result<OptionLoxType, LoxReturn> {
        expr.accept(self)
    }

    fn is_truthy(&self, value: &OptionLoxType) -> bool {
        match value.get().as_ref() {
            None => true,
            Some(lox_type) => match lox_type {
                LoxType::Str(str) => !str.is_empty(),
                LoxType::Num(num) => **num != 0.0,
                LoxType::Bool(boolean) => *boolean.as_ref(),
                LoxType::Function(_function) => {
                    panic!("Cannot evaluate truthiness of function.");
                }
                LoxType::Class(_class) => {
                    panic!("Cannot evaluate truthiness of class.");
                }
                LoxType::Instance(_) => {
                    panic!("Cannot evaluate truthiness of instance.");
                }
            },
        }
    }

    fn panic_none_or_nil(&self, lists: Vec<&OptionLoxType>) {
        for item in lists {
            if item.get().is_none() {
                panic!("Operand must not be nil.");
            }
        }
    }

    fn is_equal(&self, a: OptionLoxType, b: OptionLoxType) -> bool {
        match (a.get().as_ref(), b.get().as_ref()) {
            (None, None) => true,
            (Some(_), None) | (None, Some(_)) => false,
            (Some(val_a), Some(val_b)) => match (val_a, val_b) {
                (LoxType::Num(num_a), LoxType::Num(num_b)) => *num_a == *num_b,
                (LoxType::Str(str_a), LoxType::Str(str_b)) => *str_a == *str_b,
                (LoxType::Bool(bool_a), LoxType::Bool(bool_b)) => *bool_a == *bool_b,
                (LoxType::Function(func_a), LoxType::Function(func_b)) => {
                    func_a.eq_callable(func_b.as_ref())
                }
                (LoxType::Class(class_a), LoxType::Class(class_b)) => class_a.name == class_b.name,
                (LoxType::Instance(instance_a), LoxType::Instance(instance_b)) => {
                    instance_a.class.name == instance_b.class.name
                }
                _ => false,
            },
        }
    }

    #[allow(unused)]
    fn check_number_operand(&self, operator: &Token, operand: &Option<LoxType>) {
        if let Some(LoxType::Num(_)) = operand {
            return;
        }
        panic!("Operand must be a number for operator {:?}", operator);
    }

    fn compare_numbers<F>(
        &self,
        left: OptionLoxType,
        right: OptionLoxType,
        compare: F,
    ) -> Result<OptionLoxType, LoxReturn>
    where
        F: FnOnce(f64, f64) -> bool,
    {
        self.panic_none_or_nil(vec![&left, &right]);
        match (left.get().as_ref().unwrap(), right.get().as_ref().unwrap()) {
            (LoxType::Num(left), LoxType::Num(right)) => Ok(OptionLoxType::new(Some(
                LoxType::new_bool(compare(**left, **right)),
            ))),
            _ => panic!("Operand must be numbers"),
        }
    }

    fn calculate_number<F>(
        &self,
        left: OptionLoxType,
        right: OptionLoxType,
        calculate: F,
    ) -> Result<OptionLoxType, LoxReturn>
    where
        F: FnOnce(f64, f64) -> f64,
    {
        self.panic_none_or_nil(vec![&left, &right]);
        match (left.get().as_ref().unwrap(), right.get().as_ref().unwrap()) {
            (LoxType::Num(left), LoxType::Num(right)) => Ok(OptionLoxType::new(Some(
                LoxType::new_num(calculate(**left, **right)),
            ))),
            _ => panic!("Operand must be numbers"),
        }
    }

    fn check_call_arguments_size(&self, arg_size: usize, call_size: usize) {
        // 检查调用的参数数量是否匹配
        if arg_size != call_size {
            panic!("Expected {} arguments but got {}.", call_size, arg_size);
        }
    }
}

impl ExprVisitor for Interpreter {
    fn assign_visit(&mut self, expr: &Assign) -> Result<OptionLoxType, LoxReturn> {
        log_info!("解析赋值表达式: {:?}", expr);
        let value = self.evaluate(expr.value.as_ref())?;

        let distance = self.locals.get(&format!("{:?}", expr));
        if let Some(local_data) = distance {
            self.environment
                .lock()
                .unwrap()
                .assign_at(local_data.depth, &expr.name, value.clone())
                .unwrap_or_else(|err| {
                    panic!("{}", err);
                });
        } else {
            self.globals
                .lock()
                .unwrap()
                .assign(expr.name.lexeme.clone(), value.clone())
                .unwrap_or_else(|err| {
                    panic!("{}", err);
                });
        }
        Ok(value)
    }

    fn binary_visit(&mut self, expr: &Binary) -> Result<OptionLoxType, LoxReturn> {
        log_info!("Visiting Binary Expression: {:?}", expr);
        let left = self.evaluate(expr.left.as_ref())?;
        let right = self.evaluate(expr.right.as_ref())?;

        match expr.operator.token_type {
            TokenType::Plus => {
                self.panic_none_or_nil(vec![&left, &right]);
                match (left.get().as_ref().unwrap(), right.get().as_ref().unwrap()) {
                    (LoxType::Str(left_str), LoxType::Str(right_str)) => {
                        Ok(OptionLoxType::new(Some(LoxType::new_str(
                            format!("{}{}", *left_str, *right_str).as_str(),
                        ))))
                    }
                    (LoxType::Num(left_num), LoxType::Num(right_str)) => Ok(OptionLoxType::new(
                        Some(LoxType::new_num(**left_num + **right_str)),
                    )),
                    // 一侧为字符串，另一侧为数字时，进行字符串拼接
                    (LoxType::Str(left_str), LoxType::Num(right_num)) => {
                        Ok(OptionLoxType::new(Some(LoxType::new_str(
                            format!("{}{}", *left_str, *right_num).as_str(),
                        ))))
                    }
                    (LoxType::Num(left_num), LoxType::Str(right_str)) => {
                        Ok(OptionLoxType::new(Some(LoxType::new_str(
                            format!("{}{}", *left_num, *right_str).as_str(),
                        ))))
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
            TokenType::BangEqual => Ok(OptionLoxType::new(Some(LoxType::new_bool(
                !self.is_equal(left, right),
            )))),
            TokenType::EqualEqual => Ok(OptionLoxType::new(Some(LoxType::new_bool(
                self.is_equal(left, right),
            )))),
            _ => Ok(OptionLoxType::none()),
        }
    }

    fn grouping_visit(&mut self, expr: &Grouping) -> Result<OptionLoxType, LoxReturn> {
        log_info!("Visiting Grouping Expression: {:?}", expr);
        expr.expression.accept(self)
    }

    fn literal_visit(&mut self, expr: &Literal) -> Result<OptionLoxType, LoxReturn> {
        log_info!("Visiting Literal Expression: {:?}", expr);
        Ok(OptionLoxType::new(expr.value.clone()))
    }

    fn logical_visit(&mut self, expr: &Logical) -> Result<OptionLoxType, LoxReturn> {
        let left = self.evaluate(expr.left.as_ref())?;

        if expr.operator.token_type == TokenType::Or {
            if self.is_truthy(&left) {
                return Ok(left);
            }
        } else if !self.is_truthy(&left) {
            return Ok(left);
        }

        self.evaluate(expr.right.as_ref())
    }

    fn unary_visit(&mut self, expr: &Unary) -> Result<OptionLoxType, LoxReturn> {
        log_info!("Visiting Unary Expression: {:?}", expr);
        let right = self.evaluate(expr.right.as_ref())?;

        match expr.operator.token_type {
            TokenType::Minus => {
                if let Some(LoxType::Num(num)) = right.get().as_mut() {
                    Ok(OptionLoxType::new(Some(LoxType::new_num(-**num))))
                } else {
                    panic!("Operand must be a number.");
                }
            }
            _ => Ok(OptionLoxType::none()),
        }
    }

    fn variable_visit(&mut self, expr: &Variable) -> Result<OptionLoxType, LoxReturn> {
        Ok(self.lookup_variable(&expr.name, expr))
    }

    fn call_visit(&mut self, expr: &Call) -> Result<OptionLoxType, LoxReturn> {
        let callee = self.evaluate(expr.callee.as_ref())?;
        let mut arguments = Vec::new();
        for argument in &expr.arguments {
            arguments.push(self.evaluate(argument.as_ref())?);
        }
        // 需要确保 callee 是一个函数
        if let Some(LoxType::Function(function)) = callee.get().as_mut() {
            self.check_call_arguments_size(arguments.len(), function.arity());
            Ok(function.call(self, &arguments))
        } else if let Some(LoxType::Class(class)) = callee.get().as_mut() {
            self.check_call_arguments_size(arguments.len(), class.arity());
            Ok(class.call(self, &arguments))
        } else {
            panic!("Can only call functions.");
        }
    }

    fn get_visit(&mut self, expr: &Get) -> Result<OptionLoxType, LoxReturn> {
        let object = self.evaluate(expr.object.as_ref())?;
        if let Some(LoxType::Instance(instance)) = object.get().as_mut() {
            return Ok(instance.get(&expr.name));
        }
        panic!(
            "Can only get instances properties, prop name: {}",
            expr.name.lexeme
        );
    }

    fn set_visit(&mut self, expr: &Set) -> Result<OptionLoxType, LoxReturn> {
        let object = self.evaluate(expr.object.as_ref())?;
        if let Some(LoxType::Instance(instance)) = object.get().as_mut() {
            let value = self.evaluate(expr.value.as_ref())?;
            instance.set(&expr.name, &value);
            return Ok(value);
        }
        panic!(
            "Can only set instances properties, prop name: {}",
            expr.name.lexeme
        );
    }

    fn this_visit(&mut self, expr: &This) -> Result<OptionLoxType, LoxReturn> {
        Ok(self.lookup_variable(&expr.keyword, expr))
    }
}

impl StmtVisitor for Interpreter {
    fn print_visit(&mut self, stmt: &Print) -> Result<OptionLoxType, LoxReturn> {
        let value = self.evaluate(stmt.expression.as_ref())?;
        match value.get().as_ref() {
            Some(v) => match v {
                LoxType::Str(s) => match unescape(s.as_str()) {
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
                LoxType::Class(class) => {
                    print!("<class {}>", class.name);
                }
                LoxType::Instance(instance) => {
                    print!("<instance {}>", instance.class.name);
                }
            },
            None => {
                print!("<nil>");
            }
        }
        Ok(OptionLoxType::none())
    }

    fn if_visit(&mut self, stmt: &If) -> Result<OptionLoxType, LoxReturn> {
        let condition_result = self.evaluate(stmt.condition.as_ref())?;
        if self.is_truthy(&condition_result) {
            let _ = self.execute(stmt.then_branch.as_ref());
            return Ok(OptionLoxType::none());
        }
        if let Some(else_branch) = stmt.else_branch.as_ref() {
            let _ = self.execute(else_branch.as_ref());
        }
        Ok(OptionLoxType::none())
    }

    fn block_visit(&mut self, stmt: &Block) -> Result<OptionLoxType, LoxReturn> {
        let _ = self.execute_block(
            &stmt.statements,
            Environment::new_with_enclosing(Arc::clone(&self.environment)),
        );
        Ok(OptionLoxType::none())
    }

    fn class_visit(&mut self, stmt: &Class) -> Result<OptionLoxType, LoxReturn> {
        self.environment
            .lock()
            .unwrap()
            .define(stmt.name.lexeme.clone(), OptionLoxType::none());
        let mut methods = HashMap::new();
        for method in &stmt.methods {
            if let Some(func) = method.as_any().downcast_ref::<Function>() {
                let function = LoxFunction::new(
                    func.clone(),
                    Some(Arc::clone(&self.environment)),
                    func.name.lexeme.eq("init"),
                );
                methods.insert(func.name.lexeme.clone(), Box::new(function));
            } else {
                panic!("Class method is not a function");
            };
        }
        let class = LoxClass::new(stmt.name.lexeme.as_str(), methods);
        let _ = self.environment.lock().unwrap().assign(
            stmt.name.lexeme.clone(),
            OptionLoxType::new(Some(LoxType::new_class(Box::new(class)))),
        );
        Ok(OptionLoxType::none())
    }

    fn expression_visit(&mut self, stmt: &Expression) -> Result<OptionLoxType, LoxReturn> {
        let _ = self.evaluate(stmt.expression.as_ref());
        Ok(OptionLoxType::none())
    }

    fn var_visit(&mut self, stmt: &Var) -> Result<OptionLoxType, LoxReturn> {
        let value = self.evaluate(stmt.initializer.as_ref())?;
        self.environment
            .lock()
            .unwrap()
            .define(stmt.name.lexeme.clone(), value);
        Ok(OptionLoxType::none())
    }

    fn while_visit(&mut self, stmt: &While) -> Result<OptionLoxType, LoxReturn> {
        let mut condition_result = self.evaluate(stmt.condition.as_ref())?;
        while self.is_truthy(&condition_result) {
            let _ = self.execute(stmt.body.as_ref());
            condition_result = self.evaluate(stmt.condition.as_ref())?;
        }
        Ok(OptionLoxType::none())
    }

    fn function_visit(&mut self, stmt: &Function) -> Result<OptionLoxType, LoxReturn> {
        let function = LoxFunction::new(stmt.clone(), Some(Arc::clone(&self.environment)), false);
        self.environment.lock().unwrap().define(
            stmt.name.lexeme.clone(),
            OptionLoxType::new(Some(LoxType::new_function(Box::new(function)))),
        );
        Ok(OptionLoxType::none())
    }

    fn return_visit(&mut self, stmt: &Return) -> Result<OptionLoxType, LoxReturn> {
        let mut value = OptionLoxType::none();
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
