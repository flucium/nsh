use std::collections::HashMap;

// Reserved
// NSH_PROMPT
// NSH_HISTORY
// NSH_HISTORY_FILE
// NSH_HISTORY_MAX_SIZE
// NSH_HISTORY_MAX_MEMORY_SIZE
// NSH_BC_[COMMAND NAME]
// NSH_REGEX

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

    // pub fn get(&mut self, key: &str) -> Option<&String> {
        pub fn get(&mut self, key: &str) -> Option<&str> {
        self.0.get(key).map(|x| &**x)
    }
}
