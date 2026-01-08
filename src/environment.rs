use crate::log_info;
use crate::scanner::Token;
use crate::scanner::token::OptionLoxType;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct Environment {
    enclosing: Option<Arc<Mutex<Environment>>>,
    values: Arc<Mutex<HashMap<String, OptionLoxType>>>,
}

impl Environment {
    pub fn new() -> Self {
        log_info!("创建新环境");
        Environment {
            enclosing: None,
            values: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn new_with_enclosing(enclosing: Arc<Mutex<Environment>>) -> Self {
        log_info!("创建新环境（携带封闭环境）");
        Environment {
            enclosing: Some(enclosing),
            values: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn new_with_values(values: HashMap<String, OptionLoxType>) -> Self {
        log_info!("创建环境（携带默认值）: {:?}", values);
        Environment {
            enclosing: None,
            values: Arc::new(Mutex::new(values)),
        }
    }

    pub fn get_enclosing(&self) -> Option<Arc<Mutex<Environment>>> {
        if let Some(enclosing) = &self.enclosing {
            log_info!("获取封闭环境");
            return Some(Arc::clone(enclosing));
        }
        None
    }

    pub fn define(&mut self, name: String, value: OptionLoxType) {
        log_info!("定义变量: {}", name);
        self.values.lock().unwrap().insert(name, value);
    }

    pub fn get(&self, name: &str) -> OptionLoxType {
        if let Some(value) = self.values.lock().unwrap().get(name) {
            log_info!("获取变量 {}", name);
            return value.clone();
        }
        if let Some(enclosing) = &self.enclosing {
            return enclosing.lock().unwrap().get(name);
        }
        panic!("Undefined variable '{}'.", name);
    }

    pub fn get_at(&self, distance: usize, name: &str) -> OptionLoxType {
        log_info!("获取变量 {} 在距离 {} 的环境中", name, distance);
        self.ancestor(distance)
            .lock()
            .unwrap()
            .values
            .lock()
            .unwrap()
            .get(name)
            .unwrap_or(&OptionLoxType::none())
            .clone()
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

    pub fn assign(&mut self, name: String, value: OptionLoxType) -> Result<(), String> {
        log_info!("分配变量 {} 值为 {:?}", name, value);
        if self.values.lock().unwrap().contains_key(&name) {
            self.values.lock().unwrap().insert(name.clone(), value);
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
        value: OptionLoxType,
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
            .lock()
            .unwrap()
            .insert(name.lexeme.clone(), value);
        Ok(())
    }
}

impl Clone for Environment {
    fn clone(&self) -> Self {
        let enclosing = if let Some(enclosing) = &self.enclosing {
            Some(Arc::clone(enclosing))
        } else {
            None
        };
        Environment {
            enclosing,
            values: Arc::clone(&self.values),
        }
    }
}
