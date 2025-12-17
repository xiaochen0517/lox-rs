use std::collections::HashMap;
use std::sync::OnceLock;

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
    False,
    For,
    Fun,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    // EOF
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoxType {
    Str(Box<String>),
    Num(Box<f64>),
    Bool(Box<bool>),
}

impl LoxType {
    pub fn new_str(s: &str) -> Self {
        LoxType::Str(Box::new(s.to_string()))
    }

    pub fn new_num(n: f64) -> Self {
        LoxType::Num(Box::new(n))
    }

    pub fn new_bool(b: bool) -> Self {
        LoxType::Bool(Box::new(b))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
    pub col_start: usize,
    pub col_end: usize,
    pub literal: Option<LoxType>,
}

impl Token {
    pub fn new(
        token_type: TokenType,
        lexeme: String,
        line: usize,
        col_start: usize,
        col_end: usize,
        literal: Option<LoxType>,
    ) -> Self {
        Token {
            token_type,
            lexeme,
            line,
            col_start,
            col_end,
            literal,
        }
    }

    pub fn to_string(&self) -> String {
        format!("{:?} {} {:?}", self.token_type, self.lexeme, self.literal)
    }
}

static KEYWORDS_MAP: OnceLock<HashMap<&'static str, TokenType>> = OnceLock::new();
pub fn get_keywords_map() -> &'static HashMap<&'static str, TokenType> {
    KEYWORDS_MAP.get_or_init(|| {
        let mut m = HashMap::new();
        m.insert("and", TokenType::And);
        m.insert("class", TokenType::Class);
        m.insert("else", TokenType::Else);
        m.insert("false", TokenType::False);
        m.insert("for", TokenType::For);
        m.insert("fun", TokenType::Fun);
        m.insert("if", TokenType::If);
        m.insert("nil", TokenType::Nil);
        m.insert("or", TokenType::Or);
        m.insert("print", TokenType::Print);
        m.insert("return", TokenType::Return);
        m.insert("super", TokenType::Super);
        m.insert("this", TokenType::This);
        m.insert("true", TokenType::True);
        m.insert("var", TokenType::Var);
        m.insert("while", TokenType::While);
        m
    })
}
