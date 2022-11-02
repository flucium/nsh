use std::collections::HashMap;

pub type Key=String;
pub type Val=String;

#[derive(Debug)]
pub struct Variable(HashMap<Key, Val>);

impl ToOwned for Variable {
    type Owned = Variable;

    fn to_owned(&self) -> Self::Owned {
        Variable {
            0: self.0.to_owned(),
        }
    }
}

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
