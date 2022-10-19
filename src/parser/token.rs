use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Token {
    String(String),   // any string
    FD(u32),          //0 ~ 9
    Variable(String), //$A
    Equal,            // =
    Ampersand,        // &
    Gt,               // >
    Lt,               // <
    Pipe,             // |
    Semicolon,        // ;
}

impl Display for Token {
    fn fmt(&self, tkn: &mut Formatter) -> Result {
        match self {
            Token::String(string) => write!(tkn, "{string}"),
            Token::FD(n) => write!(tkn, "{n}"),
            Token::Variable(string) => write!(tkn, "{string}"),
            Token::Equal => write!(tkn, "="),
            Token::Ampersand => write!(tkn, "&"),
            Token::Gt => write!(tkn, ">"),
            Token::Lt => write!(tkn, "<"),
            Token::Pipe => write!(tkn, "|"),
            Token::Semicolon => write!(tkn, ";"),
        }
    }
}
