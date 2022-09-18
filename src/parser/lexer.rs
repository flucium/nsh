use crate::parser::token::Token;
use std::collections::VecDeque;
use std::mem::swap;

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
        let mut token = self.peek_token.take().or_else(|| self.read());

        self.peek_token = self.read();

        if self.peek_token.as_ref() == Some(&Token::Equal) {
            swap(&mut token, &mut self.peek_token);
        }

        token
    }

    //read chars from self.input and return as Token
    fn read(&mut self) -> Option<Token> {
        let mut token: Option<Token> = None;

        while let Some(ch) = self.input.pop_front() {
            if ch.is_whitespace() {
                continue;
            }

            if let Some(n) = ch.is_numeric().then(|| {
                self.input.pop_front().and_then(|next_ch| {
                    let n = String::from(ch).parse::<u8>().unwrap_or(0);
                    if matches!(next_ch, '>' | '<') {
                        Some(n)
                    } else {
                        self.input.push_front(next_ch);
                        None
                    }
                })
            }) {
                token = Some(Token::Redirect(n.unwrap_or(0)));
                break;
            }

            match ch {
                '|' => {
                    token = Some(Token::Pipe);
                    break;
                }

                '>' => {
                    token = Some(Token::Redirect(1));
                    break;
                }

                '<' => {
                    token = Some(Token::Redirect(0));
                    break;
                }

                '&' => {
                    token = Some(Token::Background);
                    break;
                }

                '=' => {
                    token = Some(Token::Equal);

                    break;
                }

                '$' => {
                    token = Some(Token::Reference);
                    break;
                }

                ';' => {
                    token = Some(Token::Semicolon);
                    break;
                }

                '"' => {
                    // (ch.is_whitespace() == false).then(|| self.input.push_front(ch));

                    token = Some(Token::String(self.read_string(true)));
                    break;
                }

                _ => {
                    (ch.is_whitespace() == false).then(|| self.input.push_front(ch));

                    token = Some(Token::String(self.read_string(false)));

                    break;
                }
            }
        }

        token
    }

    fn read_string(&mut self, esc: bool) -> String {
        let mut string = String::new();

        while let Some(ch) = self.input.pop_front() {
            if ch.is_whitespace() && !esc{
                break;
            }

            if esc && ch == '"'{
                break;
            }
            string.push(ch)
        }

        string
    }
}

// #[test]
// fn lexer_test() {
//     let mut lexer = Lexer::new(
//         "ls -a > output.txt 2> err.txt | cat -b | rev | rev ; echo hello    ; KEY = VAL ; $KEY"
//             .chars()
//             .collect(),
//     );

//     assert_eq!(Some(Token::String("ls".to_string())), lexer.pop_front());
//     assert_eq!(Some(Token::String("-a".to_string())), lexer.pop_front());
//     assert_eq!(Some(Token::Redirect(1)), lexer.pop_front());
//     assert_eq!(
//         Some(Token::String("output.txt".to_string())),
//         lexer.pop_front()
//     );
//     assert_eq!(Some(Token::Redirect(2)), lexer.pop_front());
//     assert_eq!(
//         Some(Token::String("err.txt".to_string())),
//         lexer.pop_front()
//     );
//     assert_eq!(Some(Token::Pipe), lexer.pop_front());
//     assert_eq!(Some(Token::String("cat".to_string())), lexer.pop_front());
//     assert_eq!(Some(Token::String("-b".to_string())), lexer.pop_front());
//     assert_eq!(Some(Token::Pipe), lexer.pop_front());
//     assert_eq!(Some(Token::String("rev".to_string())), lexer.pop_front());
//     assert_eq!(Some(Token::Pipe), lexer.pop_front());
//     assert_eq!(Some(Token::String("rev".to_string())), lexer.pop_front());
//     assert_eq!(Some(Token::Semicolon), lexer.pop_front());
//     assert_eq!(Some(Token::String("echo".to_string())), lexer.pop_front());
//     assert_eq!(Some(Token::String("hello".to_string())), lexer.pop_front());
//     assert_eq!(Some(Token::Semicolon), lexer.pop_front());
//     assert_eq!(Some(Token::Equal), lexer.pop_front());
//     assert_eq!(Some(Token::String("KEY".to_string())), lexer.pop_front());
//     assert_eq!(Some(Token::String("VAL".to_string())), lexer.pop_front());
//     assert_eq!(Some(Token::Semicolon), lexer.pop_front());
//     assert_eq!(Some(Token::Reference), lexer.pop_front());
//     assert_eq!(Some(Token::String("KEY".to_string())), lexer.pop_front());

//     assert_eq!(
//         Some(Token::String("hello rust".to_string())),
//         Lexer::new("\"hello rust\"".chars().collect()).pop_front()
//     );
// }