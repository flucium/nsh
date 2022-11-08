use std::collections::HashMap;

// Reserved
// NSH_PROMPT
// NSH_HISTORY
// NSH_HISTORY_FILE
// NSH_HISTORY_MAX_SIZE
// NSH_HISTORY_MAX_MEMORY_SIZE
// NSH_BC_[COMMAND NAME]

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

    pub fn get(&mut self, key: &String) -> Option<&String> {
        self.0.get(key)
    }
}
