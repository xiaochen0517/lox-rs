use crate::scanner::LoxType;
use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
pub struct Environment {
    enclosing: Option<Rc<RefCell<Environment>>>,
    values: HashMap<String, Option<LoxType>>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            enclosing: None,
            values: HashMap::new(),
        }
    }

    pub fn new_with_enclosing(enclosing: Rc<RefCell<Environment>>) -> Self {
        Environment {
            enclosing: enclosing.into(),
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: Option<LoxType>) {
        self.values.insert(name, value);
        println!("(define)Environment Values: {:?}", self.values);
    }

    pub fn get(&self, name: &str) -> Option<LoxType> {
        println!("(get)Environment Values: {:?}", self.values);
        if let Some(value) = self.values.get(name) {
            return value.clone();
        }
        if let Some(enclosing) = &self.enclosing {
            return enclosing.borrow().get(name);
        }
        panic!("Undefined variable '{}'.", name);
    }

    pub fn assign(&mut self, name: String, value: Option<LoxType>) -> Result<(), String> {
        println!("(assign)Environment Values: {:?}", self.values);
        if self.values.contains_key(&name) {
            self.values.insert(name.clone(), value);
            return Ok(());
        }
        if let Some(enclosing) = &self.enclosing {
            return enclosing
                .borrow_mut()
                .assign(name.clone(), value);
        }

        Err(format!("Undefined variable '{}'.", name))
    }
}
