use crate::ast::Function;
use crate::ast::interpreter::Interpreter;
use crate::class::LoxInstance;
use crate::environment::Environment;
use crate::log_info;
use crate::scanner::LoxType;
use crate::scanner::token::{Callable, OptionLoxType};
use std::any::Any;
use std::sync::{Arc, Mutex};

pub mod native;

#[derive(Debug, Clone)]
pub struct LoxFunction {
    declaration: Function,
    closure: Option<Arc<Mutex<Environment>>>,
    is_initializer: bool,
}

impl LoxFunction {
    pub fn new(
        declaration: Function,
        closure: Option<Arc<Mutex<Environment>>>,
        is_initializer: bool,
    ) -> Self {
        LoxFunction {
            declaration,
            closure,
            is_initializer,
        }
    }

    pub fn bind(&self, instance: &LoxInstance) -> OptionLoxType {
        let mut environment = if let Some(closure) = &self.closure {
            Environment::new_with_enclosing(Arc::clone(closure))
        } else {
            Environment::new()
        };
        environment.define(
            "this".to_string(),
            OptionLoxType::new(Some(LoxType::new_instance(Box::new(instance.clone())))),
        );
        OptionLoxType::new(Some(LoxType::new_function(Box::new(LoxFunction::new(
            self.declaration.clone(),
            Some(Arc::new(Mutex::new(environment))),
            self.is_initializer,
        )))))
    }
}

impl Callable for LoxFunction {
    fn call(
        &mut self,
        interpreter: &mut Interpreter,
        arguments: &[OptionLoxType],
    ) -> OptionLoxType {
        let mut local_env;
        if let Some(closure) = &self.closure {
            local_env = Environment::new_with_enclosing(Arc::clone(closure));
        } else {
            local_env = Environment::new();
        }
        for index in 0..self.declaration.params.len() {
            let declaration_param = self.declaration.params.get(index).expect("param exist");
            let argument = arguments.get(index).expect("argument exist");
            local_env.define(declaration_param.lexeme.clone(), argument.clone())
        }
        let result = match interpreter.execute_block(&self.declaration.body, local_env) {
            Ok(_) => OptionLoxType::none(),
            Err(lox_return) => {
                log_info!("函数存在返回值: {:?}", lox_return.value);
                lox_return.value
            }
        };
        if self.is_initializer {
            log_info!("作为初始化函数返回 this");
            if let Some(closure) = &self.closure {
                return closure.lock().unwrap().get_at(0, "this");
            }
        }
        result
    }

    fn arity(&self) -> usize {
        self.declaration.params.len()
    }

    fn clone_box(&self) -> Box<dyn Callable> {
        Box::new(self.clone())
    }

    fn eq_callable(&self, other: &dyn Callable) -> bool {
        if let Some(other_func) = other.as_any().downcast_ref::<LoxFunction>() {
            return self.declaration.name.lexeme == other_func.declaration.name.lexeme
                && self.declaration.params.len() == other_func.declaration.params.len();
        }
        false
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
