use std::collections::HashMap;

pub struct Variable(HashMap<String, String>);

impl Variable {
    pub fn new() -> Self {
        Self { 0: HashMap::new() }
    }

    pub fn variable(&mut self) -> Self {
        Self {
            0: self.0.to_owned(),
        }
    }

    pub fn insert(&mut self,key:String,val:String){
        self.0.insert(key, val);
    }

    //pub fn get(&self,key:String)->Option<&String>{
    //    self.0.get(&key)
    pub fn get(&self,key:String)->Option<&str>{
        self.0.get(&key).map(|x| &**x)
    }
}
