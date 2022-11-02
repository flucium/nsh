use crate::parser::token::Token;
use std::collections::VecDeque;

#[derive(Clone)]
pub struct Lexer {
    input: VecDeque<char>,
    peek_token: Option<Token>,
}

impl Iterator for Lexer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.peek_token.take().or_else(|| self.pop_front());

        self.peek_token = self.pop_front();

        token
    }
}

impl Lexer {
    pub fn new(input: VecDeque<char>) -> Self {
        Self {
            input: input,
            peek_token: None,
        }
    }
    fn pop_front(&mut self) -> Option<Token> {
        while let Some(ch) = self.input.pop_front() {
            if ch.is_whitespace() {
                continue;
            }

            match ch {
                '#' => {
                    while let Some(ch) = self.input.pop_front() {
                        if ch == '\n' {
                            break;
                        }
                    }
                }

                ch @ '0'..='9' => {
                    let mut string = String::from(ch);

                    string.push_str(&self.read_string(false));

                    match string.parse::<i32>() {
                        Ok(number) => {
                            let front_ch = self.input.front().unwrap_or(&' ');

                            if front_ch.is_whitespace() || matches!(front_ch, '>' | '<') == false {
                                return Some(Token::String(string));
                            } else {
                                return Some(Token::FD(number));
                            }
                        }

                        Err(_) => return Some(Token::String(string)),
                    }
                }

                '|' => return Some(Token::Pipe),

                ';' => return Some(Token::Semicolon),

                '&' => {
                    if self.input.front().unwrap_or(&' ').is_whitespace() == false {
                        let mut string = self.read_string(false);

                        match string.parse::<i32>() {
                            Ok(number) => return Some(Token::FD(number)),
                            Err(_) => {
                                while let Some(ch) = string.pop() {
                                    self.input.push_front(ch)
                                }
                            }
                        }
                    }

                    return Some(Token::Ampersand);
                }

                '>' => return Some(Token::Gt),

                '<' => return Some(Token::Lt),

                '=' => return Some(Token::Equal),

                // '$' => return Some(Token::Variable(self.read_string(false))),
                '$' => {
                    let front_ch = self.input.front().unwrap_or(&' ');

                    if front_ch.is_whitespace() {
                        return Some(Token::String("$".to_owned()));
                    } else {
                        return Some(Token::Variable(self.read_string(front_ch == &'"')));
                    }
                }

                '"' => return Some(Token::String(self.read_string(true))),

                _ => {
                    let mut string = String::from(ch);

                    string.push_str(&self.read_string(false));
                    
                    if string.to_lowercase() == "let" {
                        return Some(Token::Let);
                    } else {
                        return Some(Token::String(string));
                    }
                }
            }
        }

        None
    }

    fn read_string(&mut self, esc: bool) -> String {
        let mut string_buffer = String::new();

        while let Some(ch) = self.input.pop_front() {
            if esc {
                if ch == '"' {
                    break;
                }
            } else {
                if ch.is_whitespace() || matches!(ch, ';' | '=' | '|' | '>' | '<') {
                    self.input.push_front(ch);
                    break;
                }
            }

            string_buffer.push(ch);
        }

        string_buffer
    }
}
