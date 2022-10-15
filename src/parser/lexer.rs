use crate::parser::token::Token;
use std::collections::VecDeque;

pub struct Lexer {
    input: VecDeque<char>,
    peek_token: Option<Token>,
}

impl Iterator for Lexer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.pop_front()
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
        
        let token = self.peek_token.take().or_else(|| self.read());

        self.peek_token = self.read();

        token
    }

    fn read(&mut self) -> Option<Token> {
        let mut token: Option<Token> = None;

        while let Some(ch) = self.input.pop_front() {
            if ch.is_whitespace() {
                continue;
            }

            match ch {
                ';' => {
                    token = Some(Token::Semicolon);
                    break;
                }

                '|' => {
                    token = Some(Token::Pipe);
                    break;
                }

                '>' => {
                    token = Some(Token::Gt);
                    break;
                }

                '<' => {
                    token = Some(Token::Lt);
                    break;
                }

                '=' => {
                    token = Some(Token::Equal);
                    break;
                }

                '$' => {
                    token = Some(Token::String(String::from(ch)));

                    if self.input_peek_is_eof() == false && self.input_peek_is_whitespace() == false
                    {
                        let is_esc = self.input_peek_is('"');
                        if is_esc {
                            self.input.pop_front().unwrap();
                        }

                        let string = self.read_string(is_esc);

                        token = Some(Token::Reference(string));
                    }

                    break;
                }

                '&' => {
                    token = Some(Token::Ampersand);

                    if self.input_peek_is_whitespace() == false {
                        let mut string = self.read_string(false);

                        match string.parse::<i32>() {
                            Ok(n) => {
                                token = Some(Token::FD(n));
                            }
                            Err(_) => {
                                while let Some(ch) = string.pop() {
                                    self.input.push_front(ch);
                                }
                            }
                        }
                    }

                    break;
                }

                '"' => {
                    token = Some(Token::String(self.read_string(true)));
                    break;
                }

                _ => {
                    (ch.is_whitespace() == false).then(|| self.input.push_front(ch));

                    let string = self.read_string(false);

                    if let Ok(numeric) = string.parse::<i32>() {
                        if self.input_peek_is_eof() == false
                            && self.input_peek_is_whitespace() == false
                        {
                            token = Some(Token::FD(numeric));
                            break;
                        }
                    }

                    token = Some(Token::String(string));

                    break;
                }
            }
        }

        token
    }

    fn read_string(&mut self, esc: bool) -> String {
        let mut string = String::new();

        while let Some(ch) = self.input.pop_front() {
            if esc {
                if ch == '"' {
                    break;
                }
            } else {
                if ch.is_whitespace() {
                    self.input.push_front(ch);
                    break;
                }

                if matches!(ch, ';' | '=' | '|' | '>' | '<') {
                    self.input.push_front(ch);
                    break;
                }
            }

            string.push(ch)
        }

        string
    }

    fn input_peek_is_whitespace(&self) -> bool {
        match self.input.iter().peekable().peek() {
            Some(peek_ch) => peek_ch.is_whitespace(),
            None => false,
        }
    }

    fn input_peek_is_eof(&self) -> bool {
        self.input.iter().peekable().peek().is_none()
    }

    fn input_peek_is(&self, ch: char) -> bool {
        match self.input.iter().peekable().peek() {
            Some(peek_ch) => peek_ch == &&ch,
            None => false,
        }
    }
}
