use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    String(String), // any string
    Reference,      // $
    Equal,          // =
    Pipe,           // |
    Semicolon,      //;
    Background,     // &
    Redirect(u8),   // > <
}

impl Display for Token {
    fn fmt(&self, tkn: &mut Formatter) -> Result {
        match self {
            Token::Pipe => write!(tkn, "|"),
            Token::Equal => write!(tkn, "="),
            Token::Redirect(n) => write!(tkn, "<{}", n),
            Token::Reference => write!(tkn, "$"),
            Token::Semicolon => write!(tkn, ";"),
            Token::Background => write!(tkn, "&"),
            Token::String(string) => write!(tkn, "{}", string),
        }
    }
}
