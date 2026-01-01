use crate::ast::Function;
use crate::ast::interpreter::Interpreter;
use crate::environment::Environment;
use crate::log::Log;
use crate::log_info;
use crate::scanner::LoxType;
use crate::scanner::token::Callable;
use std::any::Any;

pub mod native;

#[derive(Debug, Clone)]
pub struct LoxFunction {
    declaration: Function,
}

impl LoxFunction {
    pub fn new(declaration: Function) -> Self {
        LoxFunction { declaration }
    }
}

impl Callable for LoxFunction {
    fn call(
        &mut self,
        interpreter: &mut Interpreter,
        arguments: &Vec<Option<LoxType>>,
    ) -> Option<LoxType> {
        let mut environment = Environment::new_with_enclosing(interpreter.environment.clone());
        for index in 0..self.declaration.params.len() {
            let declaration_param = self.declaration.params.get(index).expect("param exist");
            let argument = arguments.get(index).expect("argument exist");
            environment.define(declaration_param.lexeme.clone(), argument.clone())
        }
        match interpreter.execute_block(&self.declaration.body, environment) {
            Ok(_) => None,
            Err(lox_return) => {
                log_info!("Function returned with value: {:?}", lox_return.value);
                return lox_return.value;
            }
        }
    }

    fn arity(&self) -> usize {
        self.declaration.params.len()
    }

    fn clone_box(&self) -> Box<dyn Callable> {
        Box::new(self.clone())
    }

    fn eq_callable(&self, other: &dyn Callable) -> bool {
        if let Some(other_func) = other.as_any().downcast_ref::<LoxFunction>() {
            return self.declaration.name == other_func.declaration.name;
        }
        false
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
