use crate::parser::token::Token;
use std::iter::Peekable;
use std::vec::IntoIter;

pub struct Lexer {
    chars: Peekable<IntoIter<char>>,

    tokens: Vec<Token>,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            chars: input
                .chars()
                .map(|ch| ch as char)
                .collect::<Vec<char>>()
                .into_iter()
                .peekable(),
            tokens: Vec::new(),
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        while let Some(ch) = self.chars.next() {
            if ch.is_whitespace() {
                continue;
            }

            if ch.is_numeric() {
                if let Some(peek) = self.chars.peek() {
                    if matches!(peek, '>' | '<') {
                        self.chars.next();

                        let n = String::from(ch)
                            .parse::<u8>()
                            .unwrap_or(0)
                            .try_into()
                            .unwrap_or(0);
                        self.tokens.push(Token::REDIRECT(n));
                        
                        continue;
                    }
                }
            }

            match ch {
                '|' => self.tokens.push(Token::PIPE),
                '>' => self.tokens.push(Token::REDIRECT(1)),
                '<' => self.tokens.push(Token::REDIRECT(0)),
                '&' => self.tokens.push(Token::BACKGROUND),
                '=' => self.tokens.push(Token::EQUAL),
                '$' => self.tokens.push(Token::REFERENCE),
                '"' => {
                    let mut string = String::new();
                    string.push_str(&self.extract_string(true));
                    self.tokens.push(Token::STRING(string));
                }
                _ => {
                    let mut string = String::from(ch);

                    string.push_str(&self.extract_string(false));
                    self.tokens.push(Token::STRING(string));
                }
            }

            if self.chars.peek().is_none() {
                self.tokens.push(Token::EOF);
                break;
            }
        }

        self.tokens.clone()
    }

    fn extract_string(&mut self, esc: bool) -> String {
        let mut buffer = String::new();

        while let Some(ch) = self.chars.next() {
            buffer.push(ch);

            match self.chars.peek() {
                Some(peek) => match esc {
                    true => {
                        if *peek == '"' {
                            self.chars.next();
                            break;
                        }
                    }
                    false => {
                        if peek.is_whitespace() {
                            break;
                        }
                    }
                },
                None => break,
            }
        }

        buffer
    }
}
