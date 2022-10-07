pub mod lexer;
pub mod token;
use self::lexer::Lexer;
use self::token::Token;
use std::fmt;
use std::fmt::Display;
use std::iter::Peekable;

pub struct Parser {
    lexer: Peekable<Lexer>,
    curr_tokens: Vec<Token>,
}

impl Parser {
    pub fn new(lexer: Lexer) -> Self {
        Self {
            lexer: lexer.peekable(),
            curr_tokens: Vec::new(),
        }
    }

    pub fn parse(&mut self) -> Result<Node, Error> {
        let mut nodes = Vec::new();

        loop {
            self.curr_tokens = self.read_tokens();

            if self.curr_tokens.is_empty() {
                break;
            }

            // if let Some(node) = self.parse_vinsert()? {
            //     nodes.push(node);
            //     continue;
            // }

            if let Some(node) = self.parse_close_fd() {
                nodes.push(node);
                continue;
            }

            if let Some(node) = self.parse_redirect()? {
                nodes.push(node);
                continue;
            }

            // if let Some(node) = self.parse_block() {
            //     nodes.push(node);
            //     continue;
            // }

            if let Some(node) = self.parse_pipe().or(self.parse_command()?) {
                nodes.push(node);
                continue;
            }

            if let Some(token) = self.curr_tokens.pop() {
                Err(Error::new(format!("illegal token: {token}").as_str()))?
            }
        }

        let mut tree = Node::Pipe(Pipe::new());

        let (mut left_nodes, mut right_nodes): (Vec<_>, Vec<_>) = nodes
            .into_iter()
            .partition(|node| matches!(node, Node::Pipe(_)));

        left_nodes.reverse();

        right_nodes.reverse();

        while let Some(item) = right_nodes.pop() {
            if right_nodes.is_empty() {
                tree = item;
                break;
            }

            match left_nodes.pop() {
                Some(node) => match node {
                    Node::Pipe(mut pipe) => {
                        pipe.insert_left(item);

                        if let Some(item) = right_nodes.pop() {
                            pipe.insert_right(item);
                        }
                        // tree = Some(Node::Pipe(pipe));
                        tree = Node::Pipe(pipe);
                    }

                    _ => {}
                },

                None => tree = item,
            }

            right_nodes.push(tree);
            tree = Node::Pipe(Pipe::new());
        }

        Ok(tree)
    }

    fn parse_pipe(&mut self) -> Option<Node> {
        match self.curr_tokens.last() {
            Some(Token::Pipe) => {
                self.curr_tokens.pop().unwrap();
                Some(Node::Pipe(Pipe::new()))
            }
            _ => None,
        }
    }

    // fn parse_block(&mut self) -> Option<Node> {
    //     match self.curr_tokens.last() {
    //         Some(Token::Semicolon) => {
    //             self.curr_tokens.pop().unwrap();
    //             Some(Node::Block(Block::new()))
    //         }
    //         _ => None,
    //     }
    // }

    fn parse_string(&mut self) -> Option<Node> {
        match self.curr_tokens.last() {
            Some(Token::String(_)) => {
                Some(Node::String(self.curr_tokens.pop().unwrap().to_string()))
            }
            _ => None,
        }
    }

    fn parse_fd(&mut self) -> Option<Node> {
        let token = self.curr_tokens.pop()?;

        match token {
            Token::FD(n) => {
                if n >= 0 {
                    Some(Node::FD(n.try_into().unwrap()))
                } else {
                    self.curr_tokens.push(token);
                    None
                }
            }
            _ => {
                self.curr_tokens.push(token);
                None
            }
        }
    }

    fn parse_close_fd(&mut self) -> Option<Node> {
        let token = self.curr_tokens.pop()?;

        match token {
            Token::FD(n) => {
                if n < 0 {
                    Some(Node::CloseFD(
                        i32::to_string(&n)
                            .pop()
                            .unwrap()
                            .to_string()
                            .parse::<u32>()
                            .unwrap(),
                    ))
                } else {
                    self.curr_tokens.push(token);
                    None
                }
            }
            _ => {
                self.curr_tokens.push(token);
                None
            }
        }
    }

    fn parse_command(&mut self) -> Result<Option<Node>, Error> {
        let prefix = match self.parse_vreference().or_else(|| self.parse_string()) {
            Some(prefix) => prefix,
            None => return Ok(None),
        };

        let suffix = self.parse_command_suffix()?;

        Ok(Some(Node::Command(Command {
            prefix: Some(Box::new(prefix)),
            suffix: Some(suffix),
        })))
    }

    // fn parse_command_prefix(&mut self) -> Result<Option<Node>, Error> {
    //     Ok(self.parse_vreference()?.or(self.parse_string()))
    // }

    fn parse_command_suffix(&mut self) -> Result<CommandSuffix, Error> {
        let mut suffix = CommandSuffix::new();

        while !self.curr_tokens.is_empty() {
            if let Some(node) = self.parse_vreference().or(self.parse_string()) {
                suffix.push(node)?;
            }

            if let Some(node) = self.parse_string().or_else(|| self.parse_vreference()) {
                suffix.push(node)?;
            }

            if let Some(node) = self.parse_redirect()? {
                suffix.push(node)?;
            }

            if let Some(token) = self.curr_tokens.pop() {
                if token == Token::Ampersand {
                    suffix.push(Node::Background(true))?;
                } else {
                    self.curr_tokens.push(token);
                }
            }
        }

        Ok(suffix)
    }

    fn parse_redirect(&mut self) -> Result<Option<Node>, Error> {
        let left_fd = match self.parse_fd().or_else(|| {
            self.curr_tokens.last().and_then(|token| match token {
                Token::Gt => Some(Node::FD(1)),
                Token::Lt => Some(Node::FD(0)),
                _ => None,
            })
        }) {
            Some(left) => left,
            None => return Ok(None),
        };

        let symbol = match self.curr_tokens.pop() {
            Some(symbol) => symbol,
            None => return Ok(None),
        };

        let kind = match symbol {
            Token::Gt => RedirectKind::Output,
            Token::Lt => RedirectKind::Input,
            _ => {
                match left_fd {
                    Node::FD(n) => {
                        self.curr_tokens.push(Token::FD(n.try_into().unwrap()));
                        self.curr_tokens.push(symbol);
                    }
                    _ => {}
                }
                return Err(Error::new(""));
            }
        };

        let right = match self.parse_string().or_else(|| self.parse_fd()) {
            Some(right) => right,
            None => {
                // match left_fd {
                //     Node::FD(n) => {
                //         self.curr_tokens.push(Token::FD(n.try_into().unwrap()));
                //         self.curr_tokens.push(symbol);
                //     }
                //     _ => {

                //     }
                // }

                return Err(Error::new(""));
            }
        };

        Ok(Some(Node::Redirect(Redirect {
            kind: kind,
            left: Some(Box::new(left_fd)),
            right: Some(Box::new(right)),
        })))
    }

    // fn parse_vinsert(&mut self) -> Result<Option<Node>, Error> {
    //     let key = match self.parse_string() {
    //         Some(key) => key,
    //         None => return Ok(None),
    //     };

    //     match self.curr_tokens.pop() {
    //         Some(symbol) => {
    //             if symbol != Token::Equal {
    //                 match key {
    //                     Node::String(string) => {
    //                         self.curr_tokens.push(symbol);

    //                         self.curr_tokens.push(Token::String(string));
    //                     }
    //                     _ => {}
    //                 }

    //                 return Ok(None);
    //             }
    //         }
    //         None => return Ok(None),
    //     }

    //     let val = match self.parse_string() {
    //         Some(val) => val,
    //         None => return Err(Error::new("")),
    //     };

    //     Ok(Some(Node::VInsert(VInsert {
    //         key: Some(Box::new(key)),
    //         val: Some(Box::new(val)),
    //     })))
    // }

    fn parse_vreference(&mut self) -> Option<Node> {
        let token = self.curr_tokens.pop()?;

        match token {
            Token::Reference(string) => Some(Node::VReference(string)),
            _ => {
                self.curr_tokens.push(token);
                None
            }
        }
    }

    fn read_tokens(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while let Some(token) = self.lexer.next() {
            tokens.push(token);

            if matches!(tokens.last(), Some(Token::Ampersand)) {
                break;
            }

            if matches!(
                tokens.first(),
                Some(Token::Ampersand) | Some(Token::Pipe) | Some(Token::Semicolon)
            ) {
                break;
            }

            if matches!(
                self.lexer.peek(),
                Some(Token::Pipe) | Some(Token::Semicolon)
            ) {
                break;
            }
        }

        tokens.reverse();

        tokens
    }
}


#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Node {
    String(String),
    FD(u32),
    CloseFD(u32),
    VReference(String),
    VInsert(VInsert),
    Redirect(Redirect),
    Command(Command),
    Pipe(Pipe),
    // Block(Block),
    Background(bool),
}


#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RedirectKind {
    Input,
    Output,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Redirect {
    kind: RedirectKind,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

impl Redirect {
    pub fn new(kind: RedirectKind) -> Self {
        Self {
            kind: kind,
            left: None,
            right: None,
        }
    }

    pub fn left(&self) -> Option<&Box<Node>> {
        self.left.as_ref()
    }

    pub fn right(&self) -> Option<&Box<Node>> {
        self.right.as_ref()
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

    pub fn get_kind(&mut self) -> &RedirectKind {
        &self.kind
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Command {
    prefix: Option<Box<Node>>,
    suffix: Option<CommandSuffix>,
}

impl Command {
    pub fn new() -> Self {
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

    pub fn take_prefix(&mut self) -> Option<Box<Node>> {
        self.prefix.take()
    }

    pub fn take_suffix(&mut self) -> Option<CommandSuffix> {
        self.suffix.take()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CommandSuffix(Vec<Node>);

impl CommandSuffix {
    pub fn new() -> Self {
        Self(Vec::new())
    }
    fn push(&mut self, node: Node) -> Result<(), Error> {
        match node {
            Node::Pipe(_) | Node::VInsert(_) | Node::Command(_) => Err(Error::new(
                "there is a token that cannot be passed as a command argument",
            )),

            _ => {
                self.0.push(node);

                Ok(())
            }
        }
    }

    pub fn pop(&mut self) -> Option<Node> {
        self.0.pop()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
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

    pub fn take_left(&mut self) -> Option<Box<Node>> {
        self.left.take()
    }

    pub fn take_right(&mut self) -> Option<Box<Node>> {
        self.right.take()
    }
}

// #[derive(Debug, Clone, Eq, PartialEq)]
// pub struct Block {
//     left: Option<Box<Node>>,
//     right: Option<Box<Node>>,
// }

// impl Block {
//     fn new() -> Self {
//         Self {
//             left: None,
//             right: None,
//         }
//     }

//     fn insert_left(&mut self, node: Node) {
//         self.left = Some(Box::new(node))
//     }

//     fn insert_right(&mut self, node: Node) {
//         self.right = Some(Box::new(node))
//     }

//     pub fn take_left(&mut self) -> Option<Box<Node>> {
//         self.left.take()
//     }

//     pub fn take_right(&mut self) -> Option<Box<Node>> {
//         self.right.take()
//     }
// }

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct VInsert {
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
    pub fn take_key(&mut self) -> Option<Box<Node>> {
        self.key.take()
    }

    pub fn take_val(&mut self) -> Option<Box<Node>> {
        self.val.take()
    }
}

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

    // pub fn get(&self) -> &str {
    //     &self.message
    // }

    // pub fn get_detail(&self){

    // }
}

// impl ToString for Error {
//     fn to_string(&self) -> String {
//         self.message.to_string()
//     }
// }

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}
