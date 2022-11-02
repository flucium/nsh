pub mod lexer;
pub mod token;
use self::lexer::Lexer;
use self::token::Token;
use crate::error::*;
use std::iter::Peekable;

pub struct Parser {
    lexer: Peekable<Lexer>,
}

impl Parser {
    pub fn new(lexer: Lexer) -> Self {
        Self {
            lexer: lexer.peekable(),
        }
    }

    pub fn parse(&mut self) -> Result<Tree> {
        let mut is_pipe = false;

        let mut tree = Tree::new();

        loop {
            let mut nodes = Vec::new();
            loop {
                if let Some(node) = self.parse_insert()? {
                    nodes.push(node);
                    continue;
                }

                if let Some(node) = self.parse_close_fd() {
                    nodes.push(node);
                    continue;
                }

                if let Some(node) = self.parse_redirect()? {
                    nodes.push(node);
                    continue;
                }

                if let Some(node) = self.parse_command()? {
                    nodes.push(node);
                    continue;
                }

                if self.lexer.next_if_eq(&Token::Pipe).is_some() {
                    is_pipe = true;
                    continue;
                }

                if self.lexer.next_if_eq(&Token::Semicolon).is_some() {
                    break;
                }

                
                Err(Error::new(
                    ErrorKind::WrongSyntax,
                    format!("unknown token: {}", self.lexer.peek().unwrap()),
                ))?;

                if self.lexer.peek().is_none() {
                    break;
                }
            }

            if is_pipe {
                let mut pipe = Pipe::new();

                for node in nodes {
                    pipe.insert(node)
                }

                tree.insert(Node::Pipe(pipe));
            } else {
                for node in nodes {
                    tree.insert(node)
                }
            }

            if self.lexer.peek().is_none() {
                break;
            }

            is_pipe = false;
        }

        Ok(tree)
    }

    fn parse_command(&mut self) -> Result<Option<Node>> {
        let prefix = match self.parse_reference().or_else(|| self.parse_string()) {
            Some(prefix) => prefix,
            None => return Ok(None),
        };

        let suffix = self.parse_command_suffix()?;

        let mut command = Command::new();
        command.insert_prefix(prefix);
        command.insert_suffix(suffix);

        Ok(Some(Node::Command(command)))
    }

    fn parse_command_suffix(&mut self) -> Result<CommandSuffix> {
        let mut suffix = CommandSuffix::new();

        loop {
            match self.lexer.peek() {
                Some(peek_token) => {
                    if matches!(peek_token, Token::Pipe | Token::Semicolon) {
                        break;
                    }
                }
                None => break,
            }

            if self.lexer.next_if_eq(&Token::Ampersand).is_some() {
                suffix.insert(Node::Background(true));
                break;
            }

            if let Some(node) = self.parse_reference().or(self.parse_string()) {
                suffix.insert(node);
            }

            if let Some(node) = self.parse_string().or_else(|| self.parse_reference()) {
                suffix.insert(node);
            }

            if let Some(node) = self.parse_redirect()? {
                suffix.insert(node);
            }
        }

        Ok(suffix)
    }

    fn parse_redirect(&mut self) -> Result<Option<Node>> {
        let left = match self.parse_fd().or_else(|| match self.lexer.peek() {
            Some(peek_token) => match peek_token {
                Token::Lt => Some(Node::FD(0)),
                Token::Gt => Some(Node::FD(1)),
                _ => None,
            },
            None => None,
        }) {
            Some(fd) => fd,
            None => return Ok(None),
        };

        
        let kind = match self.lexer.next() {
            Some(token) => match token {
                Token::Lt => RedirectKind::Input,
                Token::Gt => RedirectKind::Output,
                _ => Err(Error::new(ErrorKind::WrongSyntax, format!("{} token cannot be used",token)))?,
            },
            
            
            None => Err(Error::new(ErrorKind::WrongSyntax, "redirect direction is not specified".to_owned()))?,
        };

        let right = match self.parse_fd().or_else(|| self.parse_string()) {
            Some(right) => right,
            None => Err(Error::new(ErrorKind::WrongSyntax, "no redirect destination specified".to_owned()))?,
        };

        let mut redirect = Redirect::new(kind);
        redirect.insert_left(left);
        redirect.insert_right(right);

        Ok(Some(Node::Redirect(redirect)))
    }

    fn parse_close_fd(&mut self) -> Option<Node> {
        match self.lexer.next_if(|token| match token {
            Token::FD(fd) => fd < &0,
            _ => false,
        }) {
            Some(token) => match token {
                Token::FD(fd) => Some(Node::CloseFD(
                    i32::to_string(&fd)
                        .pop()
                        .unwrap()
                        .to_string()
                        .parse::<u32>()
                        .unwrap(),
                )),
                _ => None,
            },

            None => None,
        }
    }

    fn parse_background(&mut self) -> Option<Node> {
        match self.lexer.next_if_eq(&Token::Ampersand).is_some() {
            true => Some(Node::Background(true)),
            false => None,
        }
    }

    fn parse_fd(&mut self) -> Option<Node> {
        match self.lexer.next_if(|token| match token {
            Token::FD(fd) => fd >= &0,
            _ => false,
        }) {
            Some(token) => match token {
                Token::FD(fd) => Some(Node::FD(fd.try_into().unwrap())),
                _ => None,
            },

            None => None,
        }
    }

    fn parse_string(&mut self) -> Option<Node> {
        match self
            .lexer
            .next_if(|token| matches!(token, Token::String(_)))
        {
            Some(token) => match token {
                Token::String(string) => Some(Node::String(string)),
                _ => None,
            },
            None => None,
        }
    }

    fn parse_reference(&mut self) -> Option<Node> {
        match self
            .lexer
            .next_if(|token| matches!(token, Token::Variable(_)))
        {
            Some(token) => match token {
                Token::Variable(string) => Some(Node::Reference(string)),
                _ => None,
            },
            None => None,
        }
    }

    fn parse_insert(&mut self) -> Result<Option<Node>> {
        if self.lexer.next_if_eq(&Token::Let).is_none() {
            return Ok(None);
        }

        let left = match self.parse_string() {
            Some(node) => node,
            None => Err(Error::new(ErrorKind::WrongSyntax, "the prefix of = in the let statement was not found".to_owned()))?,
        };

        if self.lexer.next_if_eq(&Token::Equal).is_none() {
            Err(Error::new(ErrorKind::WrongSyntax, "the = in the let statement could not be found".to_owned()))?
        }

        let right = match self.parse_string() {
            Some(node) => node,
            None => Err(Error::new(ErrorKind::WrongSyntax, "the suffix of = in the let statement was not found".to_owned()))?,
        };

        let mut insert = Insert::new();
        insert.insert_key(left);
        insert.insert_val(right);

        Ok(Some(Node::Insert(insert)))
    }
}

#[derive(Debug, Clone)]
pub enum Node {
    String(String),
    FD(u32),
    CloseFD(u32),
    Command(Command),
    Reference(String),
    Insert(Insert),
    Redirect(Redirect),
    Background(bool),
    Tree(Tree),
    Pipe(Pipe),
}

#[derive(Debug, Clone)]
pub struct Insert {
    key: Option<Box<Node>>,
    val: Option<Box<Node>>,
}

impl Insert {
    fn new() -> Self {
        Self {
            key: None,
            val: None,
        }
    }

    fn insert_key(&mut self, key: Node) {
        self.key = Some(Box::new(key))
    }

    fn insert_val(&mut self, val: Node) {
        self.val = Some(Box::new(val))
    }

    // pub fn take(&mut self) -> (Option<Node>, Option<Node>) {
    //     (
    //         match self.key.to_owned() {
    //             Some(node) => Some(*node),
    //             None => None,
    //         },
    //         match self.val.to_owned() {
    //             Some(node) => Some(*node),
    //             None => None,
    //         },
    //     )
    // }

    pub fn take_key(&mut self) -> Option<Node> {
        match self.key.take() {
            Some(key) => Some(*key),
            None => None,
        }
    }

    pub fn take_val(&mut self) -> Option<Node> {
        match self.val.take() {
            Some(val) => Some(*val),
            None => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum RedirectKind {
    Input,
    Output,
}

#[derive(Debug, Clone)]
pub struct Redirect {
    kind: RedirectKind,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

impl Redirect {
    fn new(kind: RedirectKind) -> Self {
        Self {
            kind: kind,
            left: None,
            right: None,
        }
    }

    pub fn kind(&self) -> &RedirectKind {
        &self.kind
    }

    pub fn take_left(&mut self) -> Option<Box<Node>> {
        self.left.take()
    }

    pub fn take_right(&mut self) -> Option<Box<Node>> {
        self.right.take()
    }

    fn insert_left(&mut self, node: Node) {
        self.left = Some(Box::new(node))
    }

    fn insert_right(&mut self, node: Node) {
        self.right = Some(Box::new(node))
    }
}
#[derive(Debug, Clone)]
pub struct Command {
    prefix: Option<Box<Node>>,
    suffix: Option<Box<CommandSuffix>>,
}

impl Command {
    fn new() -> Self {
        Self {
            prefix: None,
            suffix: None,
        }
    }

    pub fn take_prefix(&mut self) -> Option<Node> {
        match self.prefix.to_owned() {
            Some(node) => Some(*node),
            None => None,
        }
    }

    pub fn take_suffix(&mut self) -> Option<CommandSuffix> {
        match self.suffix.take() {
            Some(suffix) => Some(*suffix),
            None => None,
        }
    }

    fn insert_prefix(&mut self, prefix: Node) {
        self.prefix = Some(Box::new(prefix))
    }

    fn insert_suffix(&mut self, suffix: CommandSuffix) {
        self.suffix = Some(Box::new(suffix))
    }
}

#[derive(Debug, Clone)]
pub struct CommandSuffix(StraightBTree);

impl CommandSuffix {
    fn new() -> Self {
        Self {
            0: StraightBTree::new(),
        }
    }

    fn insert(&mut self, node: Node) {
        self.0.insert(node)
    }

    pub fn take(&mut self) -> Option<Node> {
        self.0.take()
    }
}

#[derive(Debug, Clone)]
pub struct Pipe(StraightBTree);

impl Pipe {
    fn new() -> Self {
        Self {
            0: StraightBTree::new(),
        }
    }

    fn insert(&mut self, node: Node) {
        self.0.insert(node)
    }

    pub fn take(&mut self) -> Option<Node> {
        self.0.take()
    }

    pub fn is_pipe(&self) -> bool {
        self.0.is_child()
    }
}

#[derive(Debug, Clone)]
pub struct Tree {
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

impl Tree {
    fn new() -> Self {
        Self {
            left: None,
            right: None,
        }
    }

    pub fn take(&mut self) -> Option<Box<Node>> {
        self.take_left().or_else(|| self.take_right())
    }

    fn take_left(&mut self) -> Option<Box<Node>> {
        self.left.take()
    }

    fn take_right(&mut self) -> Option<Box<Node>> {
        self.right.take()
    }

    fn insert(&mut self, node: Node) {
        if self.left.is_none() {
            self.left = Some(Box::new(node))
        } else if self.right.is_none() {
            self.right = Some(Box::new(node))
        } else {
            let mut tree = Tree::new();
            tree.insert(Node::Tree(self.clone()));
            tree.insert(node);
            *self = tree;
        }
    }
}

// used with Pipe and CommandSuffix.
// use it when creating a structure that does not require a large heap memory like Vector(Vec etc..),
// and where the left is a meaningful node and the right falls unilaterally.
// since it is a FIFO, do not use it for structures that make the stack absolute.
// can't Insert or Get or Remove from any position.
#[derive(Debug, Clone)]
struct StraightBTree {
    node: Option<Box<Node>>,
    child: Option<Box<StraightBTree>>,
}

impl StraightBTree {
    fn new() -> Self {
        Self {
            node: None,
            child: None,
        }
    }

    pub fn is_child(&self) -> bool {
        if self.node.is_some() {
            return self.child.is_some();
        }

        if let Some(node) = self.child.as_ref() {
            if node.node.is_some() == false {
                return node.is_child();
            } else {
                return true;
            }
        }

        false
    }

    pub fn take(&mut self) -> Option<Node> {
        if let Some(node) = self.node.take() {
            return Some(*node);
        }

        if let Some(node) = self.child.as_mut() {
            return node.take();
        }

        None
    }

    fn insert(&mut self, node: Node) {
        if self.node.is_none() {
            self.node = Some(Box::new(node))
        } else if self.child.is_none() {
            self.child = Some(Box::new(StraightBTree {
                node: Some(Box::new(node)),
                child: None,
            }))
        } else {
            if let Some(child) = self.child.as_mut() {
                child.insert(node)
            }
        }
    }
}
