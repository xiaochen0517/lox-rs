pub struct Prompt {}

impl Prompt {
    pub fn error(line: usize, source: &str, column: usize, message: &str) {
        let line_indicator = format!("{} |", line);
        eprintln!("{}{}", line_indicator, source);
        let line_indicator_len = line_indicator.len();
        let pointer_spacing = " ".repeat(line_indicator_len + column);
        eprintln!("{}^", pointer_spacing);
        eprintln!("{}Error: {}", pointer_spacing, message);
    }
}