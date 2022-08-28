use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Token {
    String(String), // any string
    Reference,      // $
    Equal,          // =
    Pipe,           // |
    Background,     // &
    Redirect(u8),   // > <
    Semicolon,      // ;
    // Plus,           // +
    // Minus,          //-
    // Slash,          // /
    // Asterisk,       // *
}

impl Display for Token {
    fn fmt(&self, tkn: &mut Formatter) -> Result {
        match self {
            Token::Pipe => write!(tkn, "|"),
            Token::Equal => write!(tkn, "="),
            Token::Redirect(n) => write!(tkn, "<{}", n),
            Token::Reference => write!(tkn, "$"),
            Token::Background => write!(tkn, "&"),
            Token::String(string) => write!(tkn, "{}", string),
            Token::Semicolon => write!(tkn, ";"),
            // Token::Plus => write!(tkn, "+"),
            // Token::Minus => write!(tkn, "+"),
            // Token::Slash => write!(tkn, "+"),
            // Token::Asterisk => write!(tkn, "+"),
            
        }
    }
}
