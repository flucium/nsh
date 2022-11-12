pub struct History(usize, Vec<String>);

impl ToString for History {
    fn to_string(&self) -> String {
        let mut buffer = String::new();

        let len = self.1.len();
        for i in 0..len {
            if let Some(string) = self.1.get(i) {
                buffer.push_str(string);

                if i < len - 1 {
                    buffer.push('\n');
                }
            }
        }

        buffer
    }
}

impl From<String> for History {
    fn from(string: String) -> Self {
        let mut vector: Vec<String> = Vec::new();

        for string in string.split('\n') {
            vector.push(string.to_owned())
        }

        Self { 0: 0, 1: vector }
    }
}

impl From<&str> for History {
    fn from(string: &str) -> Self {
        let mut vector: Vec<String> = Vec::new();

        for string in string.split('\n') {
            vector.push(string.to_owned())
        }

        Self { 0: 0, 1: vector }
    }
}

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
        if self.0 > 0 {
            self.0 -= 1;
        }

        match self.1.get(self.0) {
            Some(val) => Some(val),
            None => None,
        }
    }
}
