use crate::scanner::{LoxType, Token};
use crate::{log_error, log_info};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct Environment {
    enclosing: Option<Arc<Mutex<Environment>>>,
    values: HashMap<String, Option<LoxType>>,
}

impl Environment {
    pub fn new() -> Self {
        log_info!("创建新环境");
        Environment {
            enclosing: None,
            values: HashMap::new(),
        }
    }

    pub fn new_with_enclosing(enclosing: Arc<Mutex<Environment>>) -> Self {
        log_info!("创建新环境（携带封闭环境）");
        Environment {
            enclosing: enclosing.into(),
            values: HashMap::new(),
        }
    }

    pub fn new_with_values(values: HashMap<String, Option<LoxType>>) -> Self {
        log_info!("创建环境（携带默认值）: {:?}", values);
        Environment {
            enclosing: None,
            values,
        }
    }

    pub fn define(&mut self, name: String, value: Option<LoxType>) {
        log_info!("定义变量: {}", name);
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<LoxType> {
        log_info!("获取变量 {}", name);
        if let Some(value) = self.values.get(name) {
            return value.clone();
        }
        if let Some(enclosing) = &self.enclosing {
            return enclosing.lock().unwrap().get(name);
        }
        panic!("Undefined variable '{}'.", name);
    }

    pub fn get_at(&self, distance: usize, name: &str) -> Option<LoxType> {
        log_info!("获取变量 {} 在距离 {} 的环境中", name, distance);
        self.ancestor(distance)
            .lock()
            .unwrap()
            .values
            .get(name)
            .cloned()
            .unwrap_or(None)
    }

    fn ancestor(&self, distance: usize) -> Arc<Mutex<Environment>> {
        let mut environment = Arc::new(Mutex::new(self.clone()));
        for _ in 0..distance {
            let env_clone = Arc::clone(&environment);
            let env_lock = env_clone.lock().unwrap();
            if let Some(enclosing) = &(env_lock.enclosing) {
                environment = Arc::clone(enclosing);
            } else {
                panic!("No enclosing environment at distance {}", distance);
            }
        }
        environment
    }

    pub fn assign(&mut self, name: String, value: Option<LoxType>) -> Result<(), String> {
        log_info!("分配变量 {} 值为 {:?}", name, value);
        if self.values.contains_key(&name) {
            self.values.insert(name.clone(), value);
            return Ok(());
        }
        if let Some(enclosing) = &self.enclosing {
            return enclosing.lock().unwrap().assign(name.clone(), value);
        }

        Err(format!("Undefined variable '{}'.", name))
    }

    pub fn assign_at(
        &mut self,
        distance: usize,
        name: &Token,
        value: Option<LoxType>,
    ) -> Result<(), String> {
        log_info!(
            "在距离 {} 的环境中分配变量 {} 值为 {:?}",
            distance,
            name.lexeme,
            value
        );
        self.ancestor(distance)
            .lock()
            .unwrap()
            .values
            .insert(name.lexeme.clone(), value);
        Ok(())
    }
}
