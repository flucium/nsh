use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Token {
    String(String),   // any string
    FD(i32),          //0 ~ 9
    Variable(String), //$A
    Equal,            // =
    Ampersand,        // &
    // Let,              // let a = b
    Gt,        // >
    Lt,        // <
    // Include,   //include
    Pipe,      // |
    Semicolon, // ;
}

impl Display for Token {
    fn fmt(&self, tkn: &mut Formatter) -> Result {
        match self {
            Token::String(string) => write!(tkn, "{string}"),
            Token::FD(n) => write!(tkn, "{n}"),
            Token::Variable(string) => write!(tkn, "{string}"),
            Token::Equal => write!(tkn, "="),
            Token::Ampersand => write!(tkn, "&"),
            // Token::Let => write!(tkn, "let"),
            Token::Gt => write!(tkn, ">"),
            Token::Lt => write!(tkn, "<"),
            // Token::Include => write!(tkn, "include"),
            Token::Pipe => write!(tkn, "|"),
            Token::Semicolon => write!(tkn, ";"),
        }
    }
}
