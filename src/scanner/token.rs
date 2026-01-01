use crate::Lox;
use crate::ast::interpreter::Interpreter;
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
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

pub trait Callable: Debug + Send + Sync {
    fn call(
        &mut self,
        interpreter: &mut Interpreter,
        arguments: &Vec<Option<LoxType>>,
    ) -> Option<LoxType>;

    fn arity(&self) -> usize;

    // 支持克隆
    fn clone_box(&self) -> Box<dyn Callable>;

    // 支持比较
    fn eq_callable(&self, other: &dyn Callable) -> bool;

    // 用于 downcast，支持比较具体类型
    fn as_any(&self) -> &dyn Any;
}

impl Clone for Box<dyn Callable> {
    fn clone(&self) -> Box<dyn Callable> {
        self.clone_box()
    }
}

impl PartialEq for dyn Callable {
    fn eq(&self, other: &Self) -> bool {
        self.eq_callable(other)
    }
}

pub struct LoxReturn {
    pub value: Option<LoxType>,
}

impl LoxReturn {
    pub fn new(value: Option<LoxType>) -> Self {
        LoxReturn { value }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoxType {
    Str(Box<String>),
    Num(Box<f64>),
    Bool(Box<bool>),
    Function(Box<dyn Callable>),
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

    pub fn new_function(func: Box<dyn Callable>) -> Self {
        LoxType::Function(func)
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
