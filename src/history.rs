pub struct History(usize, Vec<String>);

impl History {
    pub fn new() -> Self {
        Self {
            0: 0,
            1: Vec::new(),
        }
    }

    pub fn insert(&mut self, command: String) {
        self.1.push(command);
    }

    pub fn next(&mut self) -> Option<&str> {
        match self.1.get(self.0) {
            Some(val) => {
                self.0 += 1;
                Some(val)
            }
            None => None,
        }
    }

    pub fn prev(&mut self) -> Option<&str> {
        
        if self.0 > 0{
            self.0 -=1;
        }

        match self.1.get(self.0) {
            Some(val) => Some(val),
            None => None,
        }
    }

    // pub fn len(&self)->usize{
    //     self.1.len()
    // }
}

// impl From<String> for History{}
// impl From<fs::File> for History{}
// impl ToString for History