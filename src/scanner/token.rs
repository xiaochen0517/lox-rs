#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    String,
    Number,

    // Keywords.
    And,
    Class,
    Else,

    // EOF
    Eof,
}

#[derive(Debug)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
    pub literal: Option<String>,
}

impl Token {
    pub fn new(
        token_type: TokenType,
        lexeme: String,
        line: usize,
        literal: Option<String>,
    ) -> Self {
        Token {
            token_type,
            lexeme,
            line,
            literal,
        }
    }

    pub fn to_string(&self) -> String {
        format!("{:?} {} {:?}", self.token_type, self.lexeme, self.literal)
    }
}
