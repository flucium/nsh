pub mod lexer;
mod token;
use crate::parser::lexer::Lexer;
use crate::parser::token::*;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::iter::Peekable;

#[derive(Debug)]
pub struct Error {
    message: String,
    // tokens:[Token;3],
}

impl Error {
    pub fn new(message: &str) -> Self {
        Self {
            message: String::from(message),
            // tokens:tokens,
        }
    }

    pub fn get(&self) -> &str {
        &self.message
    }

    // pub fn get_detail(&self){

    // }
}

pub struct Parser {
    lexer: Peekable<Lexer>,
    nodes: RefCell<Vec<Node>>,
}

impl Parser {
    pub fn new(lexer: Lexer) -> Self {
        Self {
            lexer: lexer.peekable(),
            nodes: RefCell::new(Vec::new()),
        }
    }

    pub fn parse(&mut self) -> Result<Option<Node>, Error> {
        let mut nodes = Vec::new();

        while let Some(node) = match self.parse_pipe() {
            Some(node) => Some(node),
            None => self.parse_vinsert()?.or(self.parse_command()?),
        } {
            nodes.push(node)
        }

        let mut tree = None;

        let (mut nodes, mut items): (VecDeque<_>, VecDeque<_>) = nodes
            .into_iter()
            .partition(|node| matches!(node, Node::Pipe(_)));

        while let Some(item) = items.pop_front() {
            if items.is_empty() {
                tree = Some(item);
                break;
            }

            match nodes.pop_front() {
                Some(node) => match node {
                    Node::Pipe(mut pipe) => {
                        pipe.insert_left(item);

                        if let Some(item) = items.pop_front() {
                            pipe.insert_right(item);
                        }
                        tree = Some(Node::Pipe(pipe));
                    }

                    _ => {}
                },

                None => tree = Some(item),
            }

            if let Some(node) = tree.take() {
                items.push_front(node);
            }
        }

        
        Ok(tree)

    }

    fn parse_pipe(&mut self) -> Option<Node> {
        self.lexer
            .next_if_eq(&Token::Pipe)
            .and(Some(Node::Pipe(Pipe::new())))
    }

    fn parse_command(&mut self) -> Result<Option<Node>, Error> {
        let prefix = match self.parse_vreference()?.or(self.parse_string()) {
            Some(prefix) => prefix,
            None => return Ok(None),
        };

        let suffix = match self.parse_command_suffix() {
            Ok(ok) => ok,
            Err(err) => return Err(err),
        };

        let mut command = Command::new();

        command.insert_prefix(prefix)?;

        command.insert_suffix(suffix);

        Ok(Some(Node::Command(command)))
    }

    fn parse_command_prefix(&mut self) -> Result<Option<Node>, Error> {
        Ok(self.parse_vreference()?.or(self.parse_string()))
    }

    fn parse_command_suffix(&mut self) -> Result<CommandSuffix, Error> {
        let mut suffix = CommandSuffix::new();

        loop {
            if let Some(node) = self.parse_background() {
                suffix.insert(node)?;
                return Ok(suffix);
            }

            if let Some(node) = self.parse_vreference()?.or(self.parse_string()) {
                suffix.insert(node)?;
            }

            if let Some(node) = self.parse_redirect()? {
                suffix.insert(node)?;
            }

            match self.lexer.peek() {
                Some(peek_token) => {
                    if matches!(peek_token, Token::Pipe | Token::Semicolon) {
                        break;
                    }
                }
                None => break,
            }
        }

        Ok(suffix)
    }

    fn parse_background(&mut self) -> Option<Node> {
        match self.lexer.next_if_eq(&Token::Background).is_some() {
            true => Some(Node::Background(true)),
            false => None,
        }
    }

    fn parse_redirect(&mut self) -> Result<Option<Node>, Error> {
        if !matches!(
            self.lexer.peek().unwrap_or(&Token::Semicolon),
            Token::Redirect(_)
        ) {
            return Ok(None);
        }

        let fd = match self.lexer.next() {
            Some(token) => match token {
                Token::Redirect(n) => n,
                _ => Err(Error::new("unknown error"))?,
            },
            None => Err(Error::new("unknown error"))?,
        };

        let string_node = match self.parse_string() {
            Some(node) => node,
            None => Err(Error::new("file path to redirect to is not specified"))?,
        };

        Ok(Some(Node::Redirect(Redirect::new(fd.into(), string_node))))
    }

    fn parse_vreference(&mut self) -> Result<Option<Node>, Error> {
        if self.lexer.next_if_eq(&Token::Reference).is_none() {
            return Ok(None);
        }

        match self.lexer.next() {
            Some(token) => match token {
                Token::String(string) => Ok(Some(Node::VReference(string))),
                _ => Err(Error::new(
                    "shell variable reference key token must be string",
                ))?,
            },
            None => Err(Error::new("shell variable reference key not found")),
        }
    }

    fn parse_vinsert(&mut self) -> Result<Option<Node>, Error> {
        if self.lexer.next_if_eq(&Token::Equal).is_none() {
            return Ok(None);
        }

        let key = match self.parse_string() {
            Some(key) => key,
            None => Err(Error::new("shell variable key not found"))?,
        };

        let val = match self.parse_string() {
            Some(val) => val,
            None => Err(Error::new("shell variable val not found"))?,
        };

        let mut node = VInsert::new();
        node.insert_key(key);
        node.insert_val(val);

        Ok(Some(Node::VInsert(node)))
    }

    fn parse_string(&mut self) -> Option<Node> {
        if !matches!(self.lexer.peek(), Some(Token::String(_))) {
            return None;
        }

        Some(Node::String(self.lexer.next()?.to_string()))
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Node {
    String(String),
    VReference(String),
    VInsert(VInsert),
    Redirect(Redirect),
    Command(Command),
    Pipe(Pipe),
    Background(bool),
}

#[derive(Debug, Eq, PartialEq)]
struct VInsert {
    key: Option<Box<Node>>,
    val: Option<Box<Node>>,
}

impl VInsert {
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
}
#[derive(Debug, Eq, PartialEq)]
struct Redirect {
    fd: usize,
    file: Box<Node>,
}

impl Redirect {
    fn new(fd: usize, file: Node) -> Self {
        Self {
            fd: fd,
            file: Box::new(file),
        }
    }
}
#[derive(Debug, Eq, PartialEq)]
struct Command {
    prefix: Option<Box<Node>>,
    suffix: Option<CommandSuffix>,
}

impl Command {
    fn new() -> Self {
        Self {
            prefix: None,
            suffix: None,
        }
    }

    fn insert_prefix(&mut self, node: Node) -> Result<(), Error> {
        match node {
            Node::Pipe(_) | Node::VInsert(_) | Node::Command(_) => {
                Err(Error::new("some tokens cannot be passed as commands"))
            }
            _ => {
                self.prefix = Some(Box::new(node));
                Ok(())
            }
        }
    }

    fn insert_suffix(&mut self, suffix: CommandSuffix) {
        self.suffix = Some(suffix);
    }
}
#[derive(Debug, Eq, PartialEq)]
struct CommandSuffix {
    v: Option<Box<Node>>,
    suffix: Option<Box<CommandSuffix>>,
}
impl CommandSuffix {
    fn new() -> Self {
        Self {
            v: None,
            suffix: None,
        }
    }

    fn insert(&mut self, node: Node) -> Result<(), Error> {
        match node {
            Node::Pipe(_) | Node::VInsert(_) | Node::Command(_) => Err(Error::new(
                "there is a token that cannot be passed as a command argument",
            )),

            _ => {
                if self.v.is_some() {
                    if let Some(suffix) = &mut self.suffix {
                        suffix.insert(node)?;
                    } else {
                        self.suffix = Some(Box::new(CommandSuffix {
                            v: Some(Box::new(node)),
                            suffix: None,
                        }));
                    }
                } else {
                    self.v = Some(Box::new(node))
                }

                Ok(())
            }
        }
    }
}
#[derive(Debug, Eq, PartialEq)]
struct Pipe {
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

impl Pipe {
    fn new() -> Self {
        Self {
            left: None,
            right: None,
        }
    }

    fn insert_left(&mut self, node: Node) {
        self.left = Some(Box::new(node))
    }

    fn insert_right(&mut self, node: Node) {
        self.right = Some(Box::new(node))
    }
}
