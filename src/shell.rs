use crate::evaluator::Evaluator;
use crate::history::History;
use crate::parser;
use crate::parser::lexer::Lexer;
use crate::parser::Parser;
use crate::prompt;
use crate::terminal::Terminal;
use crate::variable::Variable;
use std::env;
use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::path::PathBuf;

pub struct Shell {
    terminal: Terminal,
    variable: Variable,
}

impl Shell {
    pub fn new() -> Self {
        Self {
            terminal: Terminal::new(),
            variable: Variable::new(),
        }
    }

    pub fn initialize(&mut self) -> &mut Self {
        let local_profile = Profile::new(ProfileKind::Local);

        let value = if local_profile.exists() {
            local_profile.read()
        } else {
            local_profile.create()
        };

        //とりあえずPANIC!
        match value {
            Ok(value) => match parse(value.to_owned()) {
                Ok(node) => {
                    let mut evaluator = Evaluator::new(node);
                    evaluator.variable(self.variable.to_owned());
                    if let Err(err) = evaluator.eval() {
                        panic!("{err}")
                    }
                    if let Err(err) = evaluator.wait() {
                        panic!("{err}")
                    }
                    self.variable = evaluator.get_variable().to_owned();
                }
                Err(err) => {
                    panic!("{err}")
                }
            },
            Err(err) => panic!("{err}"),
        }

        self.init_history();

        self
    }

    fn init_history(&mut self) {
        if let Some(val) = self.variable.get("NSH_HISTORY".to_owned()) {
            if val.parse::<bool>().unwrap_or(false) {
                self.terminal.history(History::new());
            }
        }
    }

    pub fn repl(&mut self) {
        loop {
            self.rep()
        }
    }

    fn rep(&mut self) {
        self.terminal.prompt(prompt::decode(
            self.variable
                .get("NSH_PROMPT".to_owned())
                .unwrap_or(">".to_owned()),
        ));

        let source = match self.terminal.read_line() {
            Ok(string) => string,
            Err(err) => panic!("{err}"),
        };

        match parse(source) {
            Ok(node) => {
                let mut evaluator = Evaluator::new(node);
                evaluator.variable(self.variable.to_owned());
                if let Err(err) = evaluator.eval() {
                    eprintln!("{:?}", err);
                }

                if let Err(err) = evaluator.wait() {
                    eprintln!("{:?}", err);
                }

                self.variable = evaluator.get_variable().to_owned();
            }
            Err(err) => {
                eprintln!("{:?}", err);
            }
        }
    }
}

fn parse(source: String) -> Result<parser::Node, parser::Error> {
    Parser::new(Lexer::new(
        source
            .replace('\n', " ;")
            .replace('~', &env::var("HOME").unwrap_or("/".to_owned()))
            .chars()
            .collect(),
    ))
    .parse()
}

struct Profile(ProfileKind);

impl Profile {
    fn new(profile_type: ProfileKind) -> Self {
        Self { 0: profile_type }
    }

    fn exists(&self) -> bool {
        if let Ok(path) = self.lookup() {
            return path.exists();
        }

        false
    }

    fn create(&self) -> io::Result<String> {
        const DEFAULT_VALUE: &str = "NSH_PROMPT = \"w\\# \"";

        let path = self.lookup()?;

        File::create(path)?.write_all(DEFAULT_VALUE.as_bytes())?;

        Ok(DEFAULT_VALUE.to_string())
    }

    fn read(&self) -> io::Result<String> {
        let path = self.lookup()?;

        let mut buffer = Vec::new();

        File::open(path)?.read_to_end(&mut buffer)?;

        Ok(String::from_utf8_lossy(&buffer).to_string())
    }

    fn lookup(&self) -> io::Result<PathBuf> {
        match self.0 {
            ProfileKind::Local => match env::var("HOME") {
                Ok(val) => {
                    let mut path = PathBuf::from(val);
                    path.push(".nsh_profile");
                    Ok(path)
                }
                Err(err) => panic!("{err}"),
            },
        }
    }
}

enum ProfileKind {
    Local,
}

// fn open_from_raw_fd(fd: i32) -> File {
//     unsafe { File::from_raw_fd(fd) }
// }

// fn libc_dup2(src_fd: i32, dst_fd: i32) {
//     unsafe {
//         libc::dup2(src_fd, dst_fd);
//     }
// }

// fn libc_close(fd: u32) {
//     unsafe {
//         libc::close(fd as i32);
//     }
// }
