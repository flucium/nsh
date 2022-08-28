mod lexer;
mod token;
use crate::parser::lexer::Lexer;
use crate::parser::token::*;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::io::stdout;
use std::io::Write;
use std::iter::Peekable;

pub struct Parser {
    lexer: Peekable<Lexer>,
    nodes: RefCell<Vec<Node>>,
}

impl Parser {
    pub fn new(input: VecDeque<char>) -> Self {
        Self {
            lexer: Lexer::new(input).peekable(),
            nodes: RefCell::new(Vec::new()),
        }
    }

    pub fn parse(&mut self) {
        loop {
            let node = self.parse_block().or_else(|| {
                match self.parse_vinsert() {
                    Ok(ok) => ok,
                    Err(err) => {
                        panic!("")
                    }
                }
                .or_else(|| match self.parse_command() {
                    Ok(ok) => ok,
                    Err(err) => panic!(""),
                })
                .or_else(|| self.parse_pipe())
            });

            match node {
                Some(node) => self.nodes.borrow_mut().push(node),

                None => {
                    match self.create_tree() {
                        Ok(ok) => {
                            if let Some(node) = ok {
                                self.nodes.borrow_mut().push(node)
                            }
                        }
                        Err(err) => panic!(""),
                    }

                    break;
                }
            }
        }

        writeln!(stdout(), "{:?}", self.nodes);
    }

    // pub fn parse(&mut self) {
    //     loop {
    //         if let Some(node) = self.parse_block() {
    //             self.nodes.borrow_mut().push(node)
    //         }

    //         let node = match self.parse_vinsert() {
    //             Ok(ok) => ok,
    //             Err(_) => panic!(""),
    //         };

    //         if let Some(node ) = node{
    //             self.nodes.borrow_mut().push(node);
    //         }

    //         let node = match self.parse_command() {
    //             Ok(ok) => ok,
    //             Err(_) => panic!(""),
    //         }
    //         .or_else(|| self.parse_pipe());

    //         if let Some(node) = node {
    //             self.nodes.borrow_mut().push(node);
    //         } else {
    //             match self.create_tree() {
    //                 Ok(ok) => {
    //                     if let Some(node) = ok {
    //                         self.nodes.borrow_mut().push(node)
    //                     }
    //                 }
    //                 Err(_) => panic!(""),
    //             }

    //             break;
    //         }
    //     }

    //     writeln!(stdout(), "{:?}", self.nodes);
    // }

    fn create_tree(&mut self) -> Result<Option<Node>, ()> {
        let mut buf_node: Option<Node> = None;

        let (mut nodes, mut items): (VecDeque<_>, VecDeque<_>) = self
            .nodes
            .take()
            .into_iter()
            .partition(|node| matches!(node, Node::Pipe(_) | Node::Block(_)));

        while let Some(item) = items.pop_front() {
            if items.is_empty() {
                buf_node = Some(item);
                break;
            }

            match nodes.pop_front() {
                Some(node) => match node {
                    Node::Pipe(mut pipe) => {
                        pipe.insert_left(item);

                        if let Some(item) = items.pop_front() {
                            pipe.insert_right(item);
                        }
                        buf_node = Some(Node::Pipe(pipe));
                    }

                    Node::Block(mut block) => {
                        block.insert_left(item);

                        if let Some(item) = items.pop_front() {
                            block.insert_right(item)
                        }

                        buf_node = Some(Node::Block(block))
                    }
                    _ => {}
                },
                None => buf_node = Some(item),
            }

            if let Some(node) = buf_node.take() {
                items.push_front(node);
            }
        }

        Ok(buf_node)
    }

    fn parse_block(&mut self) -> Option<Node> {
        self.lexer
            .next_if_eq(&Token::Semicolon)
            .and(Some(Node::Block(Block::new())))
    }

    fn parse_pipe(&mut self) -> Option<Node> {
        self.lexer
            .next_if_eq(&Token::Pipe)
            .and(Some(Node::Pipe(Pipe::new())))
    }

    fn parse_command(&mut self) -> Result<Option<Node>, ()> {
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

    fn parse_command_prefix(&mut self) -> Result<Option<Node>, ()> {
        Ok(self.parse_vreference()?.or(self.parse_string()))
    }

    fn parse_command_suffix(&mut self) -> Result<CommandSuffix, ()> {
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

    fn parse_redirect(&mut self) -> Result<Option<Node>, ()> {
        if !matches!(
            self.lexer.peek().unwrap_or(&Token::Semicolon),
            Token::Redirect(_)
        ) {
            return Ok(None);
        }

        let fd = match self.lexer.next() {
            Some(token) => match token {
                Token::Redirect(n) => n,
                _ => return Err(()),
            },
            None => return Err(()),
        };

        let string_node = match self.parse_string() {
            Some(node) => node,
            None => return Err(()),
        };

        Ok(Some(Node::Redirect(Redirect::new(fd.into(), string_node))))
    }

    fn parse_vreference(&mut self) -> Result<Option<Node>, ()> {
        if self.lexer.next_if_eq(&Token::Reference).is_none() {
            return Ok(None);
        }

        match self.lexer.next() {
            Some(token) => match token {
                Token::String(string) => Ok(Some(Node::VReference(string))),
                _ => Err(()),
            },
            None => Err(()),
        }
    }

    fn parse_vinsert(&mut self) -> Result<Option<Node>, ()> {
        if self.lexer.next_if_eq(&Token::Equal).is_none() {
            return Ok(None);
        }

        let node = self.parse_string().and_then(|key| {
            self.parse_string().and_then(|val| {
                let mut vinsert = VInsert::new();
                vinsert.insert_key(key);
                vinsert.insert_val(val);

                Some(vinsert)
            })
        });

        match node {
            Some(node) => Ok(Some(Node::VInsert(node))),
            None => Err(()),
        }
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
    // Number(usize),
    VReference(String),
    VInsert(VInsert),
    Redirect(Redirect),
    Command(Command),
    Pipe(Pipe),
    Block(Block),
    Background(bool),
}

impl Node {
    pub fn get(self) -> Node {
        self
    }
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
    fn insert_prefix(&mut self, node: Node) -> Result<(), ()> {
        match node {
            Node::Pipe(_) | Node::VInsert(_) | Node::Command(_) => Err(()),
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

    fn insert(&mut self, node: Node) -> Result<(), ()> {
        match node {
            Node::Pipe(_) | Node::VInsert(_) | Node::Command(_) => Err(()),

            _ => {
                if self.v.is_none() {
                    self.v = Some(Box::new(node))
                } else {
                    if let Some(suffix) = &mut self.suffix {
                        suffix.insert(node)?
                    } else {
                        self.suffix = Some(Box::new(CommandSuffix {
                            v: Some(Box::new(node)),
                            suffix: None,
                        }));
                    }
                }

                Ok(())
            }
        }
    }
}
#[derive(Debug, Eq, PartialEq)]
pub struct Pipe {
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

    pub fn left(&self) -> Option<&Node> {
        self.left.as_deref()
    }

    pub fn right(&self) -> Option<&Node> {
        self.right.as_deref()
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Block {
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

impl Block {
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

    pub fn left(&self) -> Option<&Node> {
        self.left.as_deref()
    }

    pub fn right(&self) -> Option<&Node> {
        self.right.as_deref()
    }
}
