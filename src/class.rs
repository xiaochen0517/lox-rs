#[derive(Debug, Clone, PartialEq)]
pub struct LoxClass {
    pub name: String,
}

impl LoxClass {
    pub fn new(name: &str) -> Self {
        LoxClass {
            name: name.to_string(),
        }
    }

    pub fn box_clone(&self) -> Box<LoxClass> {
        Box::new(self.clone())
    }
}
