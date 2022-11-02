use std::collections::HashMap;

pub struct Variable(HashMap<Key, Val>);

impl Variable {
    pub fn new() -> Self {
        Self { 0: HashMap::new() }
    }

    pub fn insert(&mut self, key: Key, val: Val) {
        self.0.insert(key, val);
    }

    pub fn get(&mut self, key: &Key) -> Option<&Val> {
        self.0.get(key)
    }
}

#[derive(Debug,Eq, Hash, PartialEq)]
pub struct Key(String);

impl From<String> for Key {
    fn from(key: String) -> Self {
        Self { 0: key }
    }
}

impl From<&str> for Key {
    fn from(key: &str) -> Self {
        Self { 0: key.to_owned() }
    }
}

impl ToString for Key {
    fn to_string(&self) -> String {
        self.0.to_owned()
    }
}

impl Key {
    pub fn new() -> Self {
        Self { 0: String::new() }
    }

    pub fn default() -> Self {
        Self {
            0: String::default(),
        }
    }

    pub fn push_str(&mut self, key: &str) {
        self.0.push_str(key)
    }
}

#[derive(Debug)]
pub struct Val(String);

impl From<String> for Val {
    fn from(val: String) -> Self {
        Self { 0: val }
    }
}

impl From<&str> for Val {
    fn from(val: &str) -> Self {
        Self { 0: val.to_owned() }
    }
}

impl ToString for Val {
    fn to_string(&self) -> String {
        self.0.to_owned()
    }
}

impl Val {
    pub fn new() -> Self {
        Self { 0: String::new() }
    }

    pub fn default() -> Self {
        Self {
            0: String::default(),
        }
    }

    pub fn push_str(&mut self, val: &str) {
        self.0.push_str(val)
    }

    pub fn insert(&mut self, val: &str) {
        if self.0.len() > 0 {
            self.0.push(':');
        }

        self.0.push_str(val)
    }

    pub fn get(&self) -> Vec<&str> {
        let mut buffer = Vec::new();

        for val in self.0.split(':') {
            buffer.push(val)
        }

        buffer
    }
}

