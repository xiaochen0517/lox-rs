use crate::ast::interpreter::Interpreter;
use crate::function::LoxFunction;
use crate::scanner::token::{Callable, OptionLoxType};
use crate::scanner::{LoxType, Token};
use std::any::Any;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct LoxClass {
    pub name: String,
    pub methods: HashMap<String, Box<LoxFunction>>,
}

impl LoxClass {
    pub fn new(name: &str, methods: HashMap<String, Box<LoxFunction>>) -> Self {
        LoxClass {
            name: name.to_string(),
            methods,
        }
    }

    pub fn box_clone(&self) -> Box<LoxClass> {
        Box::new(self.clone())
    }

    pub fn find_method(&self, name: &str) -> Option<Box<LoxFunction>> {
        if let Some(method) = self.methods.get(name) {
            return Some(method.clone());
        }
        None
    }
}

impl Callable for LoxClass {
    fn call(
        &mut self,
        _interpreter: &mut Interpreter,
        _arguments: &[OptionLoxType],
    ) -> OptionLoxType {
        let instance = LoxInstance::new(self.box_clone());
        OptionLoxType::new(Some(LoxType::new_instance(Box::new(instance))))
    }

    fn arity(&self) -> usize {
        0
    }

    fn clone_box(&self) -> Box<dyn Callable> {
        todo!()
    }

    fn eq_callable(&self, _other: &dyn Callable) -> bool {
        todo!()
    }

    fn as_any(&self) -> &dyn Any {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct LoxInstance {
    pub class: Box<LoxClass>,
    pub fields: HashMap<String, OptionLoxType>,
}

impl LoxInstance {
    pub fn new(class: Box<LoxClass>) -> Self {
        LoxInstance {
            class,
            fields: HashMap::new(),
        }
    }

    pub fn get(&self, name: &Token) -> OptionLoxType {
        let lexeme = name.lexeme.as_str();
        if self.fields.contains_key(lexeme) {
            return self.fields.get(lexeme).cloned().unwrap();
        }
        let method = self.class.find_method(lexeme);
        if let Some(method) = method {
            return OptionLoxType::new(Some(LoxType::new_function(method)));
        }
        panic!("Undefined property '{}'.", name.lexeme);
    }

    pub fn set(&mut self, name: &Token, value: &OptionLoxType) {
        self.fields.insert(name.lexeme.clone(), value.clone());
    }
}
