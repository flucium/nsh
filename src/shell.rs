use crate::evaluator::Evaluator;
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
use std::os::unix::prelude::{AsRawFd, FromRawFd};
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
        self
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

                if let Err(err) = evaluator.eval() {
                    eprintln!("{:?}",err);
                }

                // if let Err(err) = evaluator.wait() {
                //     eprintln!("{:?}",err);    
                // }
            }
            Err(err) => {
                eprintln!("{:?}",err);
            }
        }
    }
}

fn parse(source: String) -> Result<parser::Node, parser::Error> {
    Parser::new(Lexer::new(
        source
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

fn open_from_raw_fd(fd: i32) -> File {
    unsafe { File::from_raw_fd(fd) }
}

fn libc_dup2(src_fd: i32, dst_fd: i32) {
    unsafe {
        libc::dup2(src_fd, dst_fd);
    }
}

fn libc_close(fd: u32) {
    unsafe {
        libc::close(fd as i32);
    }
}
