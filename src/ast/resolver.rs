use crate::ast::interpreter::Interpreter;
use crate::ast::{
    Assign, Binary, Block, Call, Expr, ExprVisitor, Expression, Function, Grouping, If, Literal,
    Logical, Print, Return, Stmt, StmtVisitor, Unary, Var, Variable, While,
};
use crate::log_info;
use crate::prompt::Prompt;
use crate::scanner::token::LoxReturn;
use crate::scanner::{LoxType, Token};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
enum FunctionType {
    None,
    Function,
}

pub struct Resolver {
    interpreter: Interpreter,
    scopes: Vec<HashMap<String, bool>>,
    current_function: FunctionType,
}

impl Resolver {
    pub fn new(interpreter: Interpreter) -> Self {
        Resolver {
            interpreter,
            scopes: Vec::new(),
            current_function: FunctionType::None,
        }
    }
    pub fn get_interpreter(&self) -> Interpreter {
        self.interpreter.clone()
    }

    pub fn resolve_stmts(&mut self, statements: &[Box<dyn Stmt>]) -> Result<(), LoxReturn> {
        for statement in statements {
            self.resolve_stmt(statement)?;
        }
        Ok(())
    }

    fn resolve_stmt(&mut self, stmt: &Box<dyn Stmt>) -> Result<(), LoxReturn> {
        stmt.accept(self)?;
        Ok(())
    }

    fn resolve_expr(&mut self, expr: &Box<dyn Expr>) -> Result<(), LoxReturn> {
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
        scope.insert(name.lexeme.clone(), false);
    }

    fn define(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }
        let scope = self.scopes.last_mut().unwrap();
        scope.insert(name.lexeme.clone(), true);
    }

    fn resolve_local(&mut self, expr: &Box<dyn Expr>, name: &Token) {
        log_info!("Resolving local variable size: {}", self.scopes.len());
        for index in (0..self.scopes.len()).rev() {
            let scope = &self.scopes[index];
            log_info!("Checking scope {}: {:?}", index, scope);
            if scope.contains_key(&name.lexeme) {
                let _ = self
                    .interpreter
                    .resolve(expr.clone(), self.scopes.len() - 1 - index);
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
    fn assign_visit(&mut self, expr: &Assign) -> Result<Option<LoxType>, LoxReturn> {
        self.resolve_expr(&expr.value)?;
        self.resolve_local(&expr.box_clone(), &expr.name);
        Ok(None)
    }

    fn binary_visit(&mut self, expr: &Binary) -> Result<Option<LoxType>, LoxReturn> {
        self.resolve_expr(&expr.left)?;
        self.resolve_expr(&expr.right)?;
        Ok(None)
    }

    fn grouping_visit(&mut self, expr: &Grouping) -> Result<Option<LoxType>, LoxReturn> {
        self.resolve_expr(&expr.expression)?;
        Ok(None)
    }

    fn literal_visit(&mut self, _expr: &Literal) -> Result<Option<LoxType>, LoxReturn> {
        Ok(None)
    }

    fn logical_visit(&mut self, _expr: &Logical) -> Result<Option<LoxType>, LoxReturn> {
        self.resolve_expr(&_expr.left)?;
        self.resolve_expr(&_expr.right)?;
        Ok(None)
    }

    fn unary_visit(&mut self, expr: &Unary) -> Result<Option<LoxType>, LoxReturn> {
        self.resolve_expr(&expr.right.box_clone())?;
        Ok(None)
    }

    fn variable_visit(&mut self, expr: &Variable) -> Result<Option<LoxType>, LoxReturn> {
        if let Some(is_defined) = self.scopes.last().unwrap().get(&expr.name.lexeme)
            && !*is_defined
        {
            Prompt::error(
                &expr.name,
                "Cannot read local variable in its own initializer.",
            );
        }
        self.resolve_local(&expr.box_clone(), &expr.name);
        Ok(None)
    }

    fn call_visit(&mut self, expr: &Call) -> Result<Option<LoxType>, LoxReturn> {
        self.resolve_expr(&expr.callee)?;
        for argument in &expr.arguments {
            self.resolve_expr(argument)?;
        }
        Ok(None)
    }
}

impl StmtVisitor for Resolver {
    fn print_visit(&mut self, stmt: &Print) -> Result<Option<LoxType>, LoxReturn> {
        self.resolve_expr(&stmt.expression)?;
        Ok(None)
    }

    fn if_visit(&mut self, stmt: &If) -> Result<Option<LoxType>, LoxReturn> {
        self.resolve_expr(&stmt.condition)?;
        self.resolve_stmt(&stmt.then_branch)?;
        if let Some(else_branch) = &stmt.else_branch {
            self.resolve_stmt(else_branch)?;
        }
        Ok(None)
    }

    fn block_visit(&mut self, stmt: &Block) -> Result<Option<LoxType>, LoxReturn> {
        self.begin_scope();
        self.resolve_stmts(&stmt.statements)?;
        self.end_scope();
        Ok(None)
    }

    fn expression_visit(&mut self, stmt: &Expression) -> Result<Option<LoxType>, LoxReturn> {
        self.resolve_expr(&stmt.expression)?;
        Ok(None)
    }

    fn var_visit(&mut self, stmt: &Var) -> Result<Option<LoxType>, LoxReturn> {
        self.declare(&stmt.name);
        self.resolve_expr(&stmt.initializer)?;
        self.define(&stmt.name);
        Ok(None)
    }

    fn while_visit(&mut self, stmt: &While) -> Result<Option<LoxType>, LoxReturn> {
        self.resolve_expr(&stmt.condition)?;
        self.resolve_stmt(&stmt.body)?;
        Ok(None)
    }

    fn function_visit(&mut self, stmt: &Function) -> Result<Option<LoxType>, LoxReturn> {
        self.declare(&stmt.name);
        self.define(&stmt.name);

        self.resolve_function(&stmt, FunctionType::Function)?;
        Ok(None)
    }

    fn return_visit(&mut self, stmt: &Return) -> Result<Option<LoxType>, LoxReturn> {
        if self.current_function == FunctionType::None {
            Prompt::error(&stmt.keyword, "Cannot return from top-level code.");
        }
        if let Some(value) = &stmt.value {
            self.resolve_expr(value)?;
        }
        Ok(None)
    }
}
