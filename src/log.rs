pub struct Log;

impl Log {
    pub fn info(message: &str) {
        println!("LOX: [INFO]: {}", message);
    }

    pub fn warn(message: &str) {
        println!("LOX: [WARN]: {}", message);
    }

    pub fn error(message: &str) {
        eprintln!("LOX: [ERROR]: {}", message);
    }
}

// Macro for logging
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => ({
        $crate::log::Log::info(&format!($($arg)*));
    })
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => ({
        $crate::log::Log::warn(&format!($($arg)*));
    })
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => ({
        $crate::log::Log::error(&format!($($arg)*));
    })
}
