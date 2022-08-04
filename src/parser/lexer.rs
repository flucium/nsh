use crate::parser::token::Token;
use crate::parser::token::Tokens;
use std::iter::Peekable;
use std::vec::IntoIter;

pub struct Lexer {
    chars: Peekable<IntoIter<char>>,

    tokens: Tokens,
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
            tokens: Tokens::new(),
        }
    }

    pub fn tokenize(&mut self) -> Tokens {
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
                        self.tokens.push_back(Token::REDIRECT(n));
                        
                        continue;
                    }
                }
            }

            match ch {
                '|' => self.tokens.push_back(Token::PIPE),
                '>' => self.tokens.push_back(Token::REDIRECT(1)),
                '<' => self.tokens.push_back(Token::REDIRECT(0)),
                '&' => self.tokens.push_back(Token::BACKGROUND),
                '=' => self.tokens.push_back(Token::EQUAL),
                '$' => self.tokens.push_back(Token::REFERENCE),
                '"' => {
                    let mut string = String::new();
                    string.push_str(&self.extract_string(true));
                    self.tokens.push_back(Token::STRING(string));
                }
                _ => {
                    let mut string = String::from(ch);

                    string.push_str(&self.extract_string(false));
                    self.tokens.push_back(Token::STRING(string));
                }
            }

            if self.chars.peek().is_none() {
                self.tokens.push_back(Token::EOF);
                break;
            }
        }

        self.tokens.clone().into()
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
