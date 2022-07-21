use std::collections::HashMap;

#[derive(Clone)]
pub struct Variable(HashMap<String, String>);

impl Variable {
    pub fn new() -> Self {
        Self { 0: HashMap::new() }
    }

    pub fn insert(&mut self, key: &str, val: &str) {
        let mut value = String::new();

        if let Some(string) = self.0.get(key) {
            value.push_str(string);
            value.push(':');
        }

        value.push_str(val);

        self.0.insert(key.to_string(), value);
    }

    pub fn get(&mut self, key: &str) -> Option<String> {
        match self.0.get(key) {
            Some(val) => Some(val.to_string()),
            None => None,
        }
    }

    // pub fn remove(&mut self, key: &str) {
    //     self.0.remove(key);
    // }
}