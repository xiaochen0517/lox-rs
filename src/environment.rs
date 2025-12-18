use crate::scanner::LoxType;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Environment {
    values: HashMap<String, Option<LoxType>>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: Option<LoxType>) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &str) -> &Option<LoxType> {
        if let Some(value) = self.values.get(name) {
            return value;
        }
        panic!("Undefined variable '{}'.", name);
    }
}
