use crate::ast::interpreter::Interpreter;
use crate::ast::{
    Assign, Binary, Block, Call, Class, Expr, ExprVisitor, Expression, Function, Get, Grouping, If,
    Literal, Logical, Print, Return, Set, Stmt, StmtVisitor, This, Unary, Var, Variable, While,
};
use crate::log_info;
use crate::prompt::Prompt;
use crate::scanner::Token;
use crate::scanner::token::{LoxReturn, OptionLoxType};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
enum FunctionType {
    None,
    Function,
    Method,
    Initializer,
}

#[derive(Debug, Clone, PartialEq)]
enum ClassType {
    None,
    Class,
}

pub struct Resolver {
    interpreter: Interpreter,
    scopes: Vec<HashMap<String, bool>>,
    current_function: FunctionType,
    current_class: ClassType,
}

impl Resolver {
    pub fn new(interpreter: Interpreter) -> Self {
        Resolver {
            interpreter,
            scopes: Vec::new(),
            current_function: FunctionType::None,
            current_class: ClassType::None,
        }
    }
    pub fn get_interpreter(&self) -> Interpreter {
        self.interpreter.clone()
    }

    pub fn resolve_stmts(&mut self, statements: &[Box<dyn Stmt>]) -> Result<(), LoxReturn> {
        for statement in statements {
            self.resolve_stmt(statement.as_ref())?;
        }
        Ok(())
    }

    fn resolve_stmt(&mut self, stmt: &dyn Stmt) -> Result<(), LoxReturn> {
        stmt.accept(self)?;
        Ok(())
    }

    fn resolve_expr(&mut self, expr: &dyn Expr) -> Result<(), LoxReturn> {
        expr.accept(self)?;
        Ok(())
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }
        let scope = self.scopes.last_mut().unwrap();
        if scope.contains_key(&name.lexeme) {
            Prompt::error(
                name,
                "A variable with this name already declared in this scope.",
            );
        }
        scope.insert(name.lexeme.clone(), false);
    }

    fn define(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }
        let scope = self.scopes.last_mut().unwrap();
        scope.insert(name.lexeme.clone(), true);
    }

    fn resolve_local<T: Expr>(&mut self, expr: &T, name: &Token) {
        log_info!("变量解析，数量: {}", self.scopes.len());
        for (index, scope) in self.scopes.iter().rev().enumerate() {
            log_info!("检查作用域 {}: {:?}", index, scope);
            if scope.contains_key(&name.lexeme) {
                let _ = self.interpreter.resolve(expr, index);
                return;
            }
        }
    }

    fn resolve_function(
        &mut self,
        function: &Function,
        func_type: FunctionType,
    ) -> Result<(), LoxReturn> {
        let enclosing_function = self.current_function.clone();
        self.current_function = func_type;

        self.begin_scope();
        for param in &function.params {
            self.declare(param);
            self.define(param);
        }
        self.resolve_stmts(&function.body)?;
        self.end_scope();
        self.current_function = enclosing_function;
        Ok(())
    }
}

impl ExprVisitor for Resolver {
    fn assign_visit(&mut self, expr: &Assign) -> Result<OptionLoxType, LoxReturn> {
        self.resolve_expr(expr.value.as_ref())?;
        self.resolve_local(expr, &expr.name);
        Ok(OptionLoxType::none())
    }

    fn binary_visit(&mut self, expr: &Binary) -> Result<OptionLoxType, LoxReturn> {
        self.resolve_expr(expr.left.as_ref())?;
        self.resolve_expr(expr.right.as_ref())?;
        Ok(OptionLoxType::none())
    }

    fn grouping_visit(&mut self, expr: &Grouping) -> Result<OptionLoxType, LoxReturn> {
        self.resolve_expr(expr.expression.as_ref())?;
        Ok(OptionLoxType::none())
    }

    fn literal_visit(&mut self, _expr: &Literal) -> Result<OptionLoxType, LoxReturn> {
        Ok(OptionLoxType::none())
    }

    fn logical_visit(&mut self, expr: &Logical) -> Result<OptionLoxType, LoxReturn> {
        self.resolve_expr(expr.left.as_ref())?;
        self.resolve_expr(expr.right.as_ref())?;
        Ok(OptionLoxType::none())
    }

    fn unary_visit(&mut self, expr: &Unary) -> Result<OptionLoxType, LoxReturn> {
        self.resolve_expr(expr.right.as_ref())?;
        Ok(OptionLoxType::none())
    }

    fn variable_visit(&mut self, expr: &Variable) -> Result<OptionLoxType, LoxReturn> {
        if !self.scopes.is_empty()
            && self.scopes.last().unwrap().get(expr.name.lexeme.as_str()) == Some(&false)
        {
            Prompt::error(
                &expr.name,
                "Cannot read local variable in its own initializer.",
            );
        }

        self.resolve_local(expr, &expr.name);
        Ok(OptionLoxType::none())
    }

    fn call_visit(&mut self, expr: &Call) -> Result<OptionLoxType, LoxReturn> {
        self.resolve_expr(expr.callee.as_ref())?;
        for argument in &expr.arguments {
            self.resolve_expr(argument.as_ref())?;
        }
        Ok(OptionLoxType::none())
    }

    fn get_visit(&mut self, expr: &Get) -> Result<OptionLoxType, LoxReturn> {
        self.resolve_expr(expr.object.as_ref())?;
        Ok(OptionLoxType::none())
    }

    fn set_visit(&mut self, expr: &Set) -> Result<OptionLoxType, LoxReturn> {
        self.resolve_expr(expr.value.as_ref())?;
        self.resolve_expr(expr.object.as_ref())?;
        Ok(OptionLoxType::none())
    }

    fn this_visit(&mut self, expr: &This) -> Result<OptionLoxType, LoxReturn> {
        if self.current_class == ClassType::None {
            Prompt::error(&expr.keyword, "Cannot use 'this' outside of a class.");
            return Ok(OptionLoxType::none());
        }
        self.resolve_local(expr, &expr.keyword);
        Ok(OptionLoxType::none())
    }
}

impl StmtVisitor for Resolver {
    fn print_visit(&mut self, stmt: &Print) -> Result<OptionLoxType, LoxReturn> {
        self.resolve_expr(stmt.expression.as_ref())?;
        Ok(OptionLoxType::none())
    }

    fn if_visit(&mut self, stmt: &If) -> Result<OptionLoxType, LoxReturn> {
        self.resolve_expr(stmt.condition.as_ref())?;
        self.resolve_stmt(stmt.then_branch.as_ref())?;
        if let Some(else_branch) = &stmt.else_branch {
            self.resolve_stmt(else_branch.as_ref())?;
        }
        Ok(OptionLoxType::none())
    }

    fn block_visit(&mut self, stmt: &Block) -> Result<OptionLoxType, LoxReturn> {
        self.begin_scope();
        self.resolve_stmts(&stmt.statements)?;
        self.end_scope();
        Ok(OptionLoxType::none())
    }

    fn class_visit(&mut self, stmt: &Class) -> Result<OptionLoxType, LoxReturn> {
        let enclosing_class = self.current_class.clone();
        self.current_class = ClassType::Class;
        self.declare(&stmt.name);
        self.define(&stmt.name);
        self.begin_scope();
        self.scopes
            .last_mut()
            .unwrap()
            .insert("this".to_string(), true);
        for method_stmt in &stmt.methods {
            if let Some(method) = method_stmt.as_any().downcast_ref::<Function>() {
                let mut declaration = FunctionType::Method;
                if method.name.lexeme == "init" {
                    declaration = FunctionType::Initializer;
                }
                self.resolve_function(method, declaration)?;
            } else {
                panic!("Class method is not a function");
            }
        }
        self.end_scope();
        self.current_class = enclosing_class;
        Ok(OptionLoxType::none())
    }

    fn expression_visit(&mut self, stmt: &Expression) -> Result<OptionLoxType, LoxReturn> {
        self.resolve_expr(stmt.expression.as_ref())?;
        Ok(OptionLoxType::none())
    }

    fn var_visit(&mut self, stmt: &Var) -> Result<OptionLoxType, LoxReturn> {
        self.declare(&stmt.name);
        self.resolve_expr(stmt.initializer.as_ref())?;
        self.define(&stmt.name);
        Ok(OptionLoxType::none())
    }

    fn while_visit(&mut self, stmt: &While) -> Result<OptionLoxType, LoxReturn> {
        self.resolve_expr(stmt.condition.as_ref())?;
        self.resolve_stmt(stmt.body.as_ref())?;
        Ok(OptionLoxType::none())
    }

    fn function_visit(&mut self, stmt: &Function) -> Result<OptionLoxType, LoxReturn> {
        self.declare(&stmt.name);
        self.define(&stmt.name);

        self.resolve_function(stmt, FunctionType::Function)?;
        Ok(OptionLoxType::none())
    }

    fn return_visit(&mut self, stmt: &Return) -> Result<OptionLoxType, LoxReturn> {
        if self.current_function == FunctionType::None {
            Prompt::error(&stmt.keyword, "Cannot return from top-level code.");
        }
        if self.current_function == FunctionType::Initializer {
            Prompt::error(&stmt.keyword, "Cannot return a value from an initializer.");
        }
        if let Some(value) = &stmt.value {
            log_info!("解析 return 语句的值: {:?}", value);
            self.resolve_expr(value.as_ref())?;
        }
        Ok(OptionLoxType::none())
    }
}
