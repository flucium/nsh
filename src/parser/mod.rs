pub mod lexer;
pub mod token;
use std::cmp::Ordering;



pub enum Most {
    Max,
    Min,
}

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
        self.value().cmp(&other.value())
    }
}

impl PartialOrd for NodeKind {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.value().cmp(&other.value()))
    }
}

impl PartialEq for NodeKind {
    fn eq(&self, other: &Self) -> bool {
        self.value() == other.value()
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

    pub fn get_most_node(&mut self, most: Most) -> Node {
        let mut current_min_node = self.clone();

        let mut buffer = Vec::new();
        buffer.push(self);

        while let Some(node) = buffer.pop() {
            match most {
                Most::Max => {
                    if node.kind > current_min_node.kind {
                        current_min_node = node.clone();
                    }
                }
                Most::Min => {
                    if node.kind < current_min_node.kind {
                        current_min_node = node.clone();
                    }
                }
            }

            if let Some(left) = &mut node.left {
                buffer.push(left);
            }

            if let Some(right) = &mut node.right {
                buffer.push(right);
            }
        }

        current_min_node
    }

    pub fn find(&mut self, target: Node) -> Option<Node> {
        self.dfs(target)
    }

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

impl Eq for Node {}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.kind.value()).cmp(&(other.kind.value()))
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.kind.value().cmp(&other.kind.value()))
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.kind.value() == other.kind.value()
    }
}
