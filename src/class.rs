use crate::ast::interpreter::Interpreter;
use crate::function::LoxFunction;
use crate::scanner::token::{Callable, OptionLoxType};
use crate::scanner::{LoxType, Token};
use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct LoxClass {
    pub name: String,
    pub superclass: Option<Box<LoxClass>>,
    pub methods: HashMap<String, Box<LoxFunction>>,
}

impl LoxClass {
    pub fn new(
        name: &str,
        superclass: Option<Box<LoxClass>>,
        methods: HashMap<String, Box<LoxFunction>>,
    ) -> Self {
        LoxClass {
            name: name.to_string(),
            superclass,
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
        if let Some(superclass) = &self.superclass {
            return superclass.find_method(name);
        }
        None
    }
}

impl Callable for LoxClass {
    fn call(
        &mut self,
        interpreter: &mut Interpreter,
        arguments: &[OptionLoxType],
    ) -> OptionLoxType {
        let instance = LoxInstance::new(self.box_clone());
        let initializer = self.find_method("init");
        if let Some(initializer) = initializer {
            let lox_type = initializer.bind(&instance);
            if let Some(LoxType::Function(mut func)) = lox_type.get().clone() {
                func.call(interpreter, arguments);
            }
        }
        OptionLoxType::new(Some(LoxType::new_instance(Box::new(instance))))
    }

    fn arity(&self) -> usize {
        let initializer = self.find_method("init");
        if let Some(initializer) = initializer {
            return initializer.arity();
        }
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

#[derive(Debug)]
pub struct LoxInstance {
    pub class: Box<LoxClass>,
    pub fields: Arc<Mutex<HashMap<String, OptionLoxType>>>,
}

impl LoxInstance {
    pub fn new(class: Box<LoxClass>) -> Self {
        LoxInstance {
            class,
            fields: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get(&self, name: &Token) -> OptionLoxType {
        let lexeme = name.lexeme.as_str();
        if self.fields.lock().unwrap().contains_key(lexeme) {
            return self.fields.lock().unwrap().get(lexeme).cloned().unwrap();
        }
        let method = self.class.find_method(lexeme);
        if let Some(method) = method {
            return method.bind(self);
        }
        panic!("Undefined property '{}'.", name.lexeme);
    }

    pub fn set(&mut self, name: &Token, value: &OptionLoxType) {
        self.fields
            .lock()
            .unwrap()
            .insert(name.lexeme.clone(), value.clone());
    }
}

impl Clone for LoxInstance {
    fn clone(&self) -> Self {
        LoxInstance {
            class: self.class.box_clone(),
            fields: Arc::clone(&self.fields),
        }
    }
}
