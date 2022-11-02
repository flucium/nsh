use std::collections::HashMap;

#[derive(Debug)]
pub struct Variable(HashMap<String, String>);

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

    pub fn insert(&mut self, key: String, val: String) {
        self.0.insert(key, val);
    }

    pub fn get(&mut self, key: &str) -> Option<&String> {
        self.0.get(key)
    }
}
