pub mod lexer;
pub mod token;
use self::lexer::Lexer;
use self::token::Token;
use std::fmt;
use std::iter::Peekable;

type Result<T> = std::result::Result<T, Error>;

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
        
        Ok(Tree::new())
    }
}

#[derive(Debug, Clone)]
pub enum Node {
    String(String),
    FD(u32),
    CloseFD(u32),
    Command(Command),
    VReference(String),
    VInsert(VInsert),
    Redirect(Redirect),
    Background(bool),
    Tree(Tree),
    Pipe(Pipe),
}

#[derive(Debug, Clone)]
pub struct Redirect {
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

impl Redirect {
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

    pub fn take_suffix(&mut self) -> Option<Node> {
        if let Some(suffix) = self.suffix.as_mut() {
            return suffix.take();
        }
        None
    }

    fn insert_prefix(&mut self, node: Node) {
        self.prefix = Some(Box::new(node))
    }

    fn insert_suffix(&mut self, node: Node) {
        if let Some(suffix) = self.suffix.as_mut() {
            suffix.insert(node)
        } else {
            self.suffix = Some(Box::new(CommandSuffix::new()));
            self.insert_suffix(node)
        }
    }
}

// #[derive(Debug, Clone)]
// pub struct CommandSuffix {
//     node: Option<Node>,
//     suffix: Option<Box<CommandSuffix>>,
// }

// impl CommandSuffix {
//     fn new() -> Self {
//         Self {
//             node: None,
//             suffix: None,
//         }
//     }

//     pub fn take(&mut self) -> Option<Node> {
//         if let Some(node) = self.node.take() {
//             return Some(node);
//         }

//         if let Some(node) = self.suffix.as_mut() {
//             return node.take();
//         }

//         None
//     }

//     fn insert(&mut self, node: Node) {
//         if self.node.is_none() {
//             self.node = Some(node)
//         } else if self.suffix.is_none() {
//             self.suffix = Some(Box::new(CommandSuffix {
//                 node: Some(node),
//                 suffix: None,
//             }))
//         } else {
//             if let Some(suffix) = self.suffix.as_mut() {
//                 suffix.insert(node)
//             }
//         }
//     }
// }
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

    pub fn take(&mut self) -> (Option<Node>, Option<Node>) {
        (self.take_key(), self.take_val())
    }

    fn take_key(&mut self) -> Option<Node> {
        match self.key.take() {
            Some(key) => Some(*key),
            None => None,
        }
    }

    fn take_val(&mut self) -> Option<Node> {
        match self.val.take() {
            Some(val) => Some(*val),
            None => None,
        }
    }

    fn insert_key(&mut self, node: Node) {
        self.key = Some(Box::new(node))
    }

    fn insert_val(&mut self, node: Node) {
        self.val = Some(Box::new(node))
    }
}

// #[derive(Debug, Clone)]
// pub struct Pipe {
//     node: Option<Box<Node>>,
//     pipe: Option<Box<Pipe>>,
// }

// impl Pipe {
//     fn new() -> Self {
//         Self {
//             node: None,
//             pipe: None,
//         }
//     }

//     pub fn is_pipe(&self) -> bool {
//         if self.node.is_some() {
//             return self.pipe.is_some();
//         }

//         if let Some(node) = self.pipe.as_ref() {
//             if node.node.is_some() == false {
//                 return node.is_pipe();
//             } else {
//                 return true;
//             }
//         }

//         false
//     }

//     pub fn take(&mut self) -> Option<Node> {
//         if let Some(node) = self.node.take() {
//             return Some(*node);
//         }

//         if let Some(node) = self.pipe.as_mut() {
//             return node.take();
//         }

//         None
//     }

//     fn insert(&mut self, node: Node) {
//         if self.node.is_none() {
//             self.node = Some(Box::new(node))
//         } else if self.pipe.is_none() {
//             self.pipe = Some(Box::new(Pipe {
//                 node: Some(Box::new(node)),
//                 pipe: None,
//             }))
//         } else {
//             if let Some(pipe) = self.pipe.as_mut() {
//                 pipe.insert(node)
//             }
//         }
//     }
// }

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
            if let Some(pipe) = self.child.as_mut() {
                pipe.insert(node)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Error {
    message: String,
}

impl Error {
    fn new(message: String) -> Self {
        Self { message: message }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
