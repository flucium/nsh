use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Token {
    String(String), // any string
    FD(i32),
    Reference(String), //$A
    Equal,             // =
    Pipe,              // |
    Ampersand,         // &
    Gt,                // >
    Lt,                // <
    Semicolon,         // ;
}

impl Display for Token {
    fn fmt(&self, tkn: &mut Formatter) -> Result {
        match self {
            Token::Pipe => write!(tkn, "|"),
            Token::Equal => write!(tkn, "="),
            Token::Reference(string)=>write!(tkn,"{string}"),
            Token::Gt => write!(tkn, ">"),
            Token::Lt => write!(tkn, "<"),
            Token::FD(n) => write!(tkn, "{n}"),
            Token::Ampersand => write!(tkn, "&"),
            Token::String(string) => write!(tkn, "{string}"),
            Token::Semicolon => write!(tkn, ";"),
        }
    }
}
