use crate::scanner::Token;

pub struct Prompt {}

impl Prompt {
    pub fn error_by_line(line: usize, source: &str, column: usize, message: &str) {
        let line_indicator = format!("{} |", line);
        eprintln!("{}{}", line_indicator, source);
        let line_indicator_len = line_indicator.len();
        let pointer_spacing = " ".repeat(line_indicator_len + column);
        eprintln!("{}^", pointer_spacing);
        eprintln!("{}Error: {}", pointer_spacing, message);
    }

    pub fn error(token: &Token, message: &str) {
        if token.token_type == crate::scanner::TokenType::Eof {
            Prompt::error_by_line(
                token.line,
                "<end of file>",
                0,
                &format!("at end {}", message),
            );
        } else {
            Prompt::error_by_line(
                token.line,
                &token.lexeme,
                0,
                &format!("at '{}' {}", token.lexeme, message),
            );
        }
    }
}
