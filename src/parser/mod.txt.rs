pub mod lexer;
pub mod token;
use crate::parser::lexer::Lexer;
use crate::parser::token::*;
use std::cmp;
use std::iter::Peekable;
use std::mem::swap;
use std::path::PathBuf;
use std::vec::IntoIter;
pub struct Parser {
    tokens: Peekable<IntoIter<Token>>,
    curr_tokens: Option<Peekable<IntoIter<Token>>>,
    ast: Node,
}

impl Parser {
    pub fn new(mut lexer: Lexer) -> Self {
        Self {
            tokens: lexer.tokenize().into_iter().peekable(),
            curr_tokens: None,
            ast: Node::Pipe {
                left: None,
                right: None,
            },
        }
    }

    pub fn parse(&mut self) {
        loop {
            if let Err(err) = self.get_tokens() {
                panic!("エラー")
            }

            if self.curr_tokens.is_none() {
                println!("BREAK");
                break;
            }

            self.new_command_tree();
        }
    }

    fn parse_command(&mut self) {}

    fn new_command_tree(&mut self) {
        let mut tokens = match self.curr_tokens.take() {
            Some(tokens) => tokens,
            None => {
                panic!("パニックぅ〜")
            }
        };

        if tokens.next_if_eq(&Token::REFERENCE).is_some() {
            match tokens.next() {
                Some(token) => match token {
                    Token::STRING(string) => Node::Command {
                        prefix: Some(Box::new(Node::Reference(string))),
                        suffix: Some(Box::new(CommandSuffix {
                            node: None,
                            suffix: None,
                        })),
                    },
                    _ => {
                        panic!("エラーだ！")
                    }
                },
                None => {
                    panic!("エラーだ！")
                }
            }
        } else {
            if let Some(token) = tokens.next() {
                match token {
                    Token::STRING(string) => Node::Command {
                        prefix: Some(Box::new(Node::String(string))),
                        suffix: Some(Box::new(CommandSuffix {
                            node: None,
                            suffix: None,
                        })),
                    },
                    _ => {
                        panic!("エラーだ！")
                    }
                }
            } else {
                panic!("エラーかも")
            }
        };

        let mut node = Node::Command {
            prefix: None,
            suffix: None,
        };

        // let mut args = Vec::new();

        // let mut redirect: (Option<String>, Option<String>, Option<String>) = (None, None, None);

        // let mut background = false;

        // while let Some(token) = tokens.next() {
        //     match token {
        //         Token::STRING(string) => args.push(Node::String(string)),

        //         Token::REDIRECT(n) => {
        //             if let Some(token) = tokens.next() {
        //                 match token {
        //                     Token::STRING(string) => match n {
        //                         0 => {
        //                             redirect.0 = Some(String::from(string));
        //                         }
        //                         1 => {
        //                             redirect.1 = Some(String::from(string));
        //                         }
        //                         2 => {
        //                             redirect.2 = Some(String::from(string));
        //                         }
        //                         _ => {
        //                             panic!("エラーかも")
        //                         }
        //                     },
        //                     _ => {
        //                         panic!("エラー")
        //                     }
        //                 }
        //             } else {
        //                 panic!("エラー")
        //             }
        //         }

        //         Token::REFERENCE => {
        //             if let Some(token) = tokens.next() {
        //                 match token {
        //                     Token::STRING(string) => {
        //                         args.push(Node::Reference(string));
        //                     }
        //                     _ => {
        //                         panic!("エラー")
        //                     }
        //                 }
        //             } else {
        //                 panic!("エラー")
        //             }
        //         }

        //         Token::BACKGROUND => {
        //             background = true;
        //             break;
        //         }

        //         Token::EOF => {
        //             println!("EOF, BREAK");
        //             break;
        //         }

        //         Token::PIPE => {
        //             println!("PIPE, BREAK");
        //             break;
        //         }

        //         _ => {}
        //     }
        // }

        // let mut args_iter = args.into_iter();

        // let mut suffix = CommandSuffix {
        //     node: None,
        //     suffix: None,
        // };

        // while let Some(arg) = args_iter.next() {
        //     suffix.insert(arg)
        // }

        // suffix.insert(Node::Redirect(redirect.0, redirect.1, redirect.2));

        // suffix.insert(Node::Bool(background));

        // node.suffix = Some(Box::new(suffix));
    }

    fn get_tokens(&mut self) -> Result<(), ()> {
        let mut tokens = Vec::new();
        while let Some(token) = self.tokens.next() {
            match token {
                Token::PIPE | Token::EOF => {
                    tokens.push(token);
                    break;
                }

                Token::BACKGROUND => {
                    match self
                        .tokens
                        .next_if(|token| matches!(token, Token::PIPE | Token::EOF))
                    {
                        Some(token) => {
                            tokens.push(token);
                            break;
                        }
                        None => {
                            return Err(());
                        }
                    }
                }

                _ => tokens.push(token),
            }
        }

        if tokens.len() != 0 {
            self.curr_tokens = Some(tokens.into_iter().peekable());
        } else {
            self.curr_tokens = None;
        }

        Ok(())
    }
}


// #[derive(Debug, Clone)]
// pub enum Node {
//     Pipe {
//         left: Option<Box<Node>>,
//         right: Option<Box<Node>>,
//     },

//     Command {
//         prefix: Option<Box<Node>>,
//         suffix: Option<Box<CommandSuffix>>,
//     },

//     Insert {
//         key: Box<Node>,
//         val: Box<Node>,
//     },
//     Reference(String),
//     String(String),
//     Bool(bool),
//     Redirect(Option<String>, Option<String>, Option<String>),
// }

// #[derive(Debug, Clone)]
// pub struct CommandSuffix {
//     node: Option<Box<Node>>,
//     suffix: Option<Box<CommandSuffix>>,
// }

// impl CommandSuffix {
//     pub fn new() -> Self {
//         Self {
//             node: None,
//             suffix: None,
//         }
//     }

//     pub fn insert(&mut self, node: Node) {
//         match &self.node {
//             Some(_) => {
//                 if let Some(suffix) = &mut self.suffix {
//                     suffix.insert(node)
//                 } else {
//                     self.suffix = Some(Box::new(CommandSuffix {
//                         node: Some(Box::new(node)),
//                         suffix: None,
//                     }))
//                 }
//             }
//             None => self.node = Some(Box::new(node)),
//         }
//     }
// }
