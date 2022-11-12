use std::collections::HashMap;

pub struct Variable(HashMap<String, String>);

impl ToOwned for Variable {
    type Owned = Variable;

    fn to_owned(&self) -> Self::Owned {
        Self {
            0: self.0.to_owned(),
        }
    }
}

impl Variable {
    pub fn new() -> Self {
        Self { 0: HashMap::new() }
    }

    pub fn remove(&mut self,key:String){
        self.0.remove(&key);
    }

    pub fn append(&mut self, key: String, val: String) {
        match self.0.get_mut(&key) {
            Some(string) => {
                string.push(':');
                string.push_str(&val)
            }
            None => {
                self.0.insert(key, val);
            }
        }
    }

    pub fn insert(&mut self, key: String, val: String) {
        self.0.insert(key, val);
    }

    //pub fn get(&self,key:String)->Option<&String>{
    //    self.0.get(&key)
    pub fn get(&self, key: String) -> Option<&str> {
        self.0.get(&key).map(|x| &**x)
    }
}
