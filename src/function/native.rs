use crate::ast::interpreter::Interpreter;
use crate::scanner::LoxType;
use crate::scanner::token::Callable;
use std::any::Any;
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct ClockNativeFunction;

impl ClockNativeFunction {
    pub fn new() -> Self {
        ClockNativeFunction
    }
}

impl Callable for ClockNativeFunction {
    fn call(
        &mut self,
        _interpreter: &mut Interpreter,
        _arguments: &Vec<Option<LoxType>>,
    ) -> Option<LoxType> {
        let current_timestamp = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs_f64();
        Some(LoxType::new_num(current_timestamp))
    }

    fn arity(&self) -> usize {
        0
    }

    fn clone_box(&self) -> Box<dyn Callable> {
        Box::new(ClockNativeFunction)
    }

    fn eq_callable(&self, other: &dyn Callable) -> bool {
        other.as_any().is::<ClockNativeFunction>()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
