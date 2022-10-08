use std::collections::HashMap;

#[derive(Clone)]
pub struct Variable(HashMap<String, String>);

impl Variable {
    pub fn new() -> Self {
        Self { 0: HashMap::new() }
    }

    pub fn insert(&mut self, key: String, val: String) {
        let mut value = String::new();

        if let Some(string) = self.0.get(&key) {
            value.push_str(string);
            value.push(':');
        }

        value.push_str(&val);

        self.0.insert(key.to_string(), value);
    }

    //pub fn get(&mut self, key: &str) -> Option<String>
    pub fn get(&mut self, key: String) -> Option<String> {
        match self.0.get(&key) {
            Some(val) => Some(val.clone()),
            None => None,
        }
    }

    pub fn remove(&mut self,key:String){
        self.0.remove(&key);
    }

    // pub fn keys(&mut self) -> Vec<String> {
    //     self.0.keys().map(|k| k.to_string()).collect()
    // }
}