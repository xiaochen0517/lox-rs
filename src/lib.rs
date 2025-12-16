use crate::parser::Parser;

mod expression;
mod parser;
mod prompt;
mod scanner;

#[derive(Debug)]
pub struct Lox {}

impl Lox {
    fn run(content: String) {
        let mut scanner = scanner::Scanner::new(content);
        let tokens = scanner.scan_tokens();
        for token in tokens.iter() {
            println!("{:?}", token);
        }
        let mut parser = Parser::new(tokens);
        let expression = parser.parse();
        println!("{:#?}", expression);
    }

    fn error(line: usize, message: &str) {
        eprintln!("[line {}] Error: {}", line, message);
    }

    pub fn run_file(path: &str) {
        let file_content_string = std::fs::read_to_string(path).expect("Reader File Error");
        Lox::run(file_content_string);
    }

    pub fn run_prompt() {
        let stdin = std::io::stdin();
        loop {
            print!("> ");
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
            let mut line = String::new();
            match stdin.read_line(&mut line) {
                Ok(0) => break, // EOF
                Ok(_) => {
                    Lox::run(line);
                }
                Err(error) => {
                    eprintln!("Error reading line: {}", error);
                    break;
                }
            }
        }
    }
}
