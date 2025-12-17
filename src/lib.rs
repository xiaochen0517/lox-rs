use crate::ast::interpreter::Interpreter;
use crate::parser::Parser;

mod ast;
mod parser;
mod prompt;
mod scanner;

#[derive(Debug)]
pub struct Lox {
    inerpreter: Interpreter,
}

impl Lox {
    pub fn new() -> Self {
        Lox {
            inerpreter: Interpreter::new(),
        }
    }

    fn run(&self, content: String) {
        let mut scanner = scanner::Scanner::new(content);
        let tokens = scanner.scan_tokens();
        for token in tokens.iter() {
            println!("{:?}", token);
        }
        let mut parser = Parser::new(tokens);
        let statements = parser.parse();
        self.inerpreter.interpret(&statements);
    }

    fn error(line: usize, message: &str) {
        eprintln!("[line {}] Error: {}", line, message);
    }

    pub fn run_file(&self, path: &str) {
        let file_content_string = std::fs::read_to_string(path).expect("Reader File Error");
        self.run(file_content_string);
    }

    pub fn run_prompt(&self) {
        let stdin = std::io::stdin();
        loop {
            print!("> ");
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
            let mut line = String::new();
            match stdin.read_line(&mut line) {
                Ok(0) => break, // EOF
                Ok(_) => {
                    self.run(line);
                }
                Err(error) => {
                    eprintln!("Error reading line: {}", error);
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_loxr() {
        let lox = Lox::new();
        lox.run_file("lox/main.lox");
    }
}
