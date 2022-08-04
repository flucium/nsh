pub mod lexer;
pub mod token;
use crate::parser::lexer::Lexer;
use crate::parser::token::*;
use std::mem::swap;

pub struct Parser {
    tokens: Tokens,
    tree: Node,
}

impl Parser {
    pub fn new(mut lexer: Lexer) -> Self {
        let tokens = lexer.tokenize();
        Self {
            tokens: tokens,
            tree: Node::new(NodeKind::Semicolon),
        }
    }

    pub fn parse_command(&mut self) -> Result<Node, ()> {
        let mut token_buffer = Vec::new();

        let left_node = match self.parse_reference() {
            Ok(ok) => ok,

            Err(_) => {
                if let Some(token) = self.tokens.pop_front() {
                    token_buffer.push(token.clone());
                    match token {
                        Token::String(string) => {
                            let mut node = Node::new(NodeKind::Arg);

                            node.value = Some(vec![string]);

                            node
                        }
                        _ => {
                            for token in token_buffer {
                                self.tokens.push_front(token)
                            }
                            return Err(());
                        }
                    }
                } else {
                    return Err(());
                }
            }
        };

        let right_node = match self.parse_reference() {
            Ok(ok) => ok,

            Err(_) => {
                let mut args = Vec::new();
                while let Some(token) = self.tokens.pop_front() {
                    match token {
                        Token::String(string) => args.push(string),
                        _ => {
                            self.tokens.push_front(token);
                            break;
                        }
                    }
                }

                let node_kind = if args.len() > 1 {
                    NodeKind::Args
                } else {
                    NodeKind::Arg
                };

                let mut node = Node::new(node_kind);

                node.value = Some(args);

                
                node
            }
        };

        let mut node = Node::new(NodeKind::Command);

        node.insert_left(left_node);
        
        
        node.insert_right(right_node);
        

        Ok(node)
    }

    fn parse_insert(&mut self) -> Result<Node, ()> {
        let mut token_buffer = Vec::new();

        let left_node = if let Some(token) = self.tokens.pop_front() {
            token_buffer.push(token.clone());

            match token {
                Token::String(string) => {
                    let mut node = Node::new(NodeKind::Arg);
                    node.value = Some(vec![string]);
                    node
                }
                _ => {
                    for token in token_buffer {
                        self.tokens.push_front(token)
                    }
                    return Err(());
                }
            }
        } else {
            return Err(());
        };

        let mut node = if let Some(token) = self.tokens.pop_front() {
            token_buffer.push(token.clone());

            if token == Token::Equal {
                Node::new(NodeKind::Insert)
            } else {
                for token in token_buffer {
                    self.tokens.push_front(token)
                }
                return Err(());
            }
        } else {
            for token in token_buffer {
                self.tokens.push_front(token)
            }
            return Err(());
        };

        let right_node = match self.parse_reference() {
            Ok(ok) => ok,

            Err(_) => {
                if let Some(token) = self.tokens.pop_front() {
                    token_buffer.push(token.clone());
                    match token {
                        Token::String(string) => {
                            let mut node = Node::new(NodeKind::Arg);

                            node.value = Some(vec![string]);

                            node
                        }
                        _ => {
                            for token in token_buffer {
                                self.tokens.push_front(token)
                            }
                            return Err(());
                        }
                    }
                } else {
                    return Err(());
                }
            }
        };

        node.insert_left(left_node);
        node.insert_right(right_node);

        Ok(node)
    }

    fn parse_reference(&mut self) -> Result<Node, ()> {
        let mut node = if let Some(token) = self.tokens.pop_front() {
            match token {
                Token::Reference => Node::new(NodeKind::Reference),
                _ => {
                    self.tokens.push_front(token);
                    return Err(());
                }
            }
        } else {
            return Err(());
        };

        let left_node = if let Some(token) = self.tokens.pop_front() {
            match token {
                Token::String(string) => {
                    let mut node = Node::new(NodeKind::Arg);
                    node.value = Some(vec![string.to_string()]);
                    node
                }
                _ => {
                    self.tokens.push_front(token);
                    return Err(());
                }
            }
        } else {
            return Err(());
        };
        node.insert_left(left_node);

        Ok(node)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum NodeKind {
    Semicolon,
    Pipe,
    Insert,
    Reference,
    Command,
    Arg,
    Args,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Node {
    kind: NodeKind,
    value: Option<Vec<String>>,
    redirect: (Option<String>, Option<String>, Option<String>),
    background: bool,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

impl Node {
    pub fn new(kind: NodeKind) -> Self {
        Self {
            kind: kind,
            value: None,
            redirect: (None, None, None),
            background: false,
            left: None,
            right: None,
        }
    }

    pub fn kind(&self) -> &NodeKind {
        &self.kind
    }

    pub fn left(&self) -> &Option<Box<Node>> {
        &self.left
    }

    pub fn right(&self) -> &Option<Box<Node>> {
        &self.right
    }

    pub fn is_left(&self) -> bool {
        self.left.is_some()
    }

    pub fn is_right(&self) -> bool {
        self.right.is_some()
    }

    // pub fn insert(&mut self, node: Node) {

    // }

    pub fn insert_left(&mut self, node: Node) {
        match self.left.clone() {
            Some(mut left) => left.insert_left(node),
            None => self.left = Some(Box::new(node)),
        }
    }

    pub fn insert_right(&mut self, node: Node) {
        match self.right.clone() {
            Some(mut right) => right.insert_right(node),
            None => self.right = Some(Box::new(node)),
        }
    }
    pub fn pop(&mut self) {}

    pub fn find(&mut self) {}

    fn dfs(&mut self, target: Node) -> Option<Node> {
        let mut buffer = Vec::new(); //FILO
        buffer.push(self);

        while let Some(node) = buffer.pop() {
            if node.kind() == target.kind() {
                return Some(node.clone());
            }

            if let Some(left) = &mut node.left {
                buffer.push(left);
            }

            if let Some(right) = &mut node.right {
                buffer.push(right);
            }
        }

        None
    }

    // pub fn insert(&mut self, node: Node) {
    //     if self.kind >= node.kind {
    //         match self.left.clone() {
    //             Some(mut left) => left.insert(node),
    //             None => self.left = Some(Box::new(node)),
    //         }
    //     } else {
    //         match self.right.clone() {
    //             Some(mut right) => right.insert(node),
    //             None => self.right = Some(Box::new(node)),
    //         }
    //     }
    // }

    // pub fn get_most_node(&mut self, most: Most) -> Node {
    //     let mut current_min_node = self.clone();

    //     let mut buffer = Vec::new();
    //     buffer.push(self);

    //     while let Some(node) = buffer.pop() {
    //         match most {
    //             Most::Max => {
    //                 if node.kind > current_min_node.kind {
    //                     current_min_node = node.clone();
    //                 }
    //             }
    //             Most::Min => {
    //                 if node.kind < current_min_node.kind {
    //                     current_min_node = node.clone();
    //                 }
    //             }
    //         }

    //         if let Some(left) = &mut node.left {
    //             buffer.push(left);
    //         }

    //         if let Some(right) = &mut node.right {
    //             buffer.push(right);
    //         }
    //     }

    //     current_min_node
    // }

    // pub fn find(&mut self, target: Node) -> Option<Node> {
    //     self.dfs(target)
    // }

    // fn dfs(&mut self, target: Node) -> Option<Node> {
    //     let mut buffer = Vec::new(); //FILO
    //     buffer.push(self);

    //     while let Some(node) = buffer.pop() {
    //         if node.kind() == target.kind() {
    //             return Some(node.clone());
    //         }

    //         if let Some(left) = &mut node.left {
    //             buffer.push(left);
    //         }

    //         if let Some(right) = &mut node.right {
    //             buffer.push(right);
    //         }
    //     }

    //     None
    // }
}

// impl Eq for Node {}

// impl Ord for Node {
//     fn cmp(&self, other: &Self) -> Ordering {
//         (self.kind.value()).cmp(&(other.kind.value()))
//     }
// }

// impl PartialOrd for Node {
//     fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
//         Some(self.kind.value().cmp(&other.kind.value()))
//     }
// }

// impl PartialEq for Node {
//     fn eq(&self, other: &Self) -> bool {
//         self.kind.value() == other.kind.value()
//     }
// }

// let mut list = Vec::new();

// let a= tokens.binary_search(&Token::Pipe);
// println!("{:?}",a);
// println!("{:?}",tokens);

// fn split(tokens: Vec<Token>, p: Token) {}

// pub enum Most {
//     Max,
//     Min,
// }

// #[derive(Debug, Clone)]
// pub enum NodeKind {
//     Semicolon(usize),
//     Pipe(usize),
//     Insert(usize),
//     Reference(usize),
//     Command(usize),
//     Arg(usize),
//     Args(usize),
// }

// impl NodeKind {
//     pub fn value(&self) -> usize {
//         return match self {
//             NodeKind::Pipe(n) => *n,
//             NodeKind::Command(n) => *n,
//             NodeKind::Args(n) => *n,
//             NodeKind::Semicolon(n) => *n,
//             NodeKind::Insert(n) => *n,
//             NodeKind::Reference(n) => *n,
//             NodeKind::Arg(n) => *n,
//         };
//     }
// }

// impl Eq for NodeKind {}

// impl Ord for NodeKind {
//     fn cmp(&self, other: &Self) -> Ordering {
//         self.value().cmp(&other.value())
//     }
// }

// impl PartialOrd for NodeKind {
//     fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
//         Some(self.value().cmp(&other.value()))
//     }
// }

// impl PartialEq for NodeKind {
//     fn eq(&self, other: &Self) -> bool {
//         self.value() == other.value()
//     }
// }
