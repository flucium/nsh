pub mod lexer;
pub mod token;
use crate::parser::lexer::Lexer;
use crate::parser::token::*;
use std::iter::Peekable;
use std::mem::swap;

pub struct Parser {
    tokens: Tokens,
    ast: Node,
    curr_node: Option<Node>,
}

impl Parser {
    pub fn new(mut lexer: Lexer) -> Self {
        Self {
            tokens: lexer.tokenize(),
            ast: Node::new(NodeKind::Semicolon),
            curr_node: None,
        }
    }

    fn peek(&mut self) -> Option<Token> {
        match self.tokens.pop_front() {
            Some(token) => {
                self.tokens.push_front(token.clone());

                Some(token)
            }
            None => None,
        }
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
    // pub fn pop(&mut self) {}

    // pub fn find(&mut self) {}

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
}

pub struct Error {
    kind: ErrorKind,
    msg: String,
}

impl Error {
    pub fn new(kind: ErrorKind, message: &str) -> Self {
        Self {
            kind: kind,
            msg: message.to_string(),
        }
    }

    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    pub fn message(&self) -> &str {
        &self.msg
    }
}

pub enum ErrorKind {
    SyntaxError,
}
