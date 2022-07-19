use crate::parser::token::Token;

pub struct Lexer {
    input: Vec<char>,
    index: usize,
    curr_ch: Option<char>,
    peek_ch: Option<char>,
    tokens: Vec<Token>,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().map(|ch| ch as char).collect(),
            index: 0,
            curr_ch: None,
            peek_ch: None,
            tokens: Vec::new(),
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        loop {
            self.read_char();

            if let Some(curr_ch) = self.curr_ch {
                if curr_ch.is_whitespace() {
                    continue;
                }

                if self.is_fd() {
                    let n = String::from(curr_ch)
                        .parse::<u8>()
                        .unwrap_or(0)
                        .try_into()
                        .unwrap_or(0);

                    self.tokens.push(Token::Redirect(n));
                    self.read_char();
                    continue;
                }

                match curr_ch {
                    '|' => self.tokens.push(Token::Pipe),
                    '>' => self.tokens.push(Token::Redirect(1)),
                    '<' => self.tokens.push(Token::Redirect(0)),
                    '&' => self.tokens.push(Token::Background),
                    '=' => self.tokens.push(Token::Equal),
                    '$' => self.tokens.push(Token::Reference),
                    ';' => self.tokens.push(Token::Semicolon),
                    '"' => {
                        let string = self.extract_string(true);
                        self.tokens.push(Token::String(string));
                    }
                    _ => {
                        let string = self.extract_string(false);
                        self.tokens.push(Token::String(string));
                    }
                }
                if self.peek_ch.is_none() {
                    break;
                }
            } else {
                break;
            }
        }


        self.tokens.clone()
    }

    fn is_fd(&mut self) -> bool {
        let temp_index = self.index;
        self.read_char();
        for ch in self.extract_string(false).chars() {
            if !ch.is_numeric() {
                self.index = temp_index - 1;
                self.read_char();
                return matches!(ch, '>' | '<');
            }
        }

        false
    }

    fn extract_string(&mut self, esc: bool) -> String {
        if self.curr_ch.unwrap_or_default() == '"' {
            self.read_char();
        }

        let mut buffer = String::from(self.curr_ch.unwrap_or_default());

        loop {
            self.read_char();
            match self.curr_ch {
                Some(ch) => {
                    match esc {
                        true => {
                            if ch == '"' {
                                break;
                            }
                        }
                        false => {
                            if ch.is_whitespace() {
                                break;
                            }
                        }
                    }

                    buffer.push(ch);
                }
                None => break,
            }
        }

        buffer
    }

    fn read_char(&mut self) {
        if let Some(ch) = self.input.get(self.index) {
            self.curr_ch = Some(*ch);
            self.peek_ch = match self.input.get(self.index + 1) {
                Some(ch) => Some(*ch),
                None => None,
            };

            self.index += 1;
        } else {
            self.curr_ch = None;
            self.peek_ch = None;
        }
    }
}
