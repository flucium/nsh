pub mod lexer;
pub mod token;

#[derive(Debug, Clone)]
pub enum NodeKind {
    Semicolon,
    Pipe,
    Insert,
    Reference,
    Command,
    Arg,
    Args,
}

//#[derive(Debug, Clone)]
// pub struct AST{}
#[derive(Debug, Clone)]
pub struct Node {
    ord: usize,
    kind: NodeKind,
    value: Option<Vec<String>>,
    redirect: (Option<String>, Option<String>, Option<String>),
    background: bool,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

impl Node {
    pub fn new(ord: usize, kind: NodeKind) -> Self {
        Self {
            ord: ord,
            kind: kind,
            value: None,
            redirect: (None, None, None),
            background: false,
            left: None,
            right: None,
        }
    }
}
