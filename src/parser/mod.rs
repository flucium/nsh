pub mod lexer;
pub mod token;
use std::cmp::Ordering;

#[derive(Debug, Clone)]
pub enum NodeKind {
    Semicolon(usize),
    Pipe(usize),
    Insert(usize),
    Reference(usize),
    Command(usize),
    Arg(usize),
    Args(usize),
}

impl NodeKind {
    pub fn value(&self) -> usize {
        return match self {
            NodeKind::Pipe(n) => *n,
            NodeKind::Command(n) => *n,
            NodeKind::Args(n) => *n,
            NodeKind::Semicolon(n) => *n,
            NodeKind::Insert(n) => *n,
            NodeKind::Reference(n) => *n,
            NodeKind::Arg(n) => *n,
        };
    }
}

impl Eq for NodeKind {}

impl Ord for NodeKind {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.value()).cmp(&(other.value()))
    }
}

impl PartialOrd for NodeKind {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.value().cmp(&other.value()))
    }
}

impl PartialEq for NodeKind {
    fn eq(&self, other: &Self) -> bool {
        (self.value()) == (other.value())
    }
}

//#[derive(Debug, Clone)]
// pub struct AST{}
#[derive(Debug, Clone)]
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

    pub fn insert(&mut self, node: Node) {
        if self.kind >= node.kind {
            match self.left.clone() {
                Some(mut left) => left.insert(node),
                None => self.left = Some(Box::new(node)),
            }
        } else {
            match self.right.clone() {
                Some(mut right) => right.insert(node),
                None => self.right = Some(Box::new(node)),
            }
        }
    }

    
}
