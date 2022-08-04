use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Token {
    STRING(String), // any string
    REFERENCE,      // $
    EQUAL,          // =
    PIPE,           // |
    BACKGROUND,     // &
    REDIRECT(u8),   // > <
    EOF,            // end of file
}

impl Display for Token {
    fn fmt(&self, tkn: &mut Formatter) -> Result {
        match self {
            Token::PIPE => write!(tkn, "|"),
            Token::EQUAL => write!(tkn, "="),
            Token::REDIRECT(n) => write!(tkn, "<{}", n),
            Token::REFERENCE => write!(tkn, "$"),
            Token::BACKGROUND => write!(tkn, "&"),
            Token::STRING(string) => write!(tkn, "{}", string),
            Token::EOF => write!(tkn, "EOF"),
        }
    }
}
