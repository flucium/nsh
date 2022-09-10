// use crate::builtin;
use crate::parser::Command;
// use crate::parser::Redirect;
use crate::builtin::cd;
use crate::parser::{lexer::Lexer, Node, Parser};
// use crate::parser::Error;
use crate::prompt;
use crate::terminal::Terminal;
use crate::variable::Variable;
use std::cell::RefCell;
use std::env;
use std::fs;
use std::io;
use std::io::{stderr, stdout};
use std::io::{Read, Stderr, Stdout, Write};
use std::path::PathBuf;
use std::process;

pub struct Shell {
    variable: Variable,
    terminal: Terminal,
    stdout: RefCell<Stdout>,
    stderr: RefCell<Stderr>,
}

impl Shell {
    pub fn new() -> Self {
        Self {
            variable: Variable::new(),
            terminal: Terminal::new(),
            stdout: RefCell::new(stdout()),
            stderr: RefCell::new(stderr()),
        }
    }

    pub fn initialize(&mut self) -> &mut Self {
        self.load_profile();

        self
    }

    fn load_profile(&mut self) {
        let local_profile = Profile::new(ProfileKind::Local);

        let profile = if local_profile.exists() {
            match local_profile.read() {
                Ok(ok) => ok,
                Err(err) => panic!("{err}"),
            }
        } else {
            match local_profile.create() {
                Ok(ok) => ok,
                Err(err) => panic!("{err}"),
            }
        };

        for line in profile.split("\n") {
            if let Some(node) = parse(line) {
                let result = self.eval(node, Vec::default());
                if !result.is_empty() {
                    self.stdout.borrow_mut().lock().write_all(&result).unwrap();
                }
            }
        }
    }

    pub fn repl(&mut self) {
        loop {
            self.rep();
        }
    }

    fn rep(&mut self) {
        self.terminal.prompt(&prompt::decode(
            self.variable.get("NSH_PROMPT").unwrap_or(">"),
        ));

        match self.terminal.read_line() {
            Ok(ok) => {
                if let Some(string) = ok {
                    if let Some(node) = parse(&string) {
                        let result = self.eval(node, Vec::default());

                        if !result.is_empty() {
                            self.stdout.borrow_mut().lock().write_all(&result).unwrap();
                        }
                    }
                }
            }
            Err(err) => {
                panic!("{err}")
            }
        }
    }

    fn eval(&mut self, node: Node, mut pipe: Vec<u8>) -> Vec<u8> {
        match node {
            Node::Pipe(mut pipe_node) => {
                if let Some(left) = pipe_node.left() {
                    if matches!(*left, Node::Pipe(_)) {
                        pipe = self.eval(*left, pipe.clone());
                    } else {
                        match *left {
                            Node::Command(command) => {
                                match self.run_command(command, Some(Vec::default())) {
                                    Ok(ok) => {
                                        if !ok.is_empty() {
                                            pipe = ok;
                                        }
                                    }
                                    Err(err) => {
                                        self.stderr
                                            .borrow_mut()
                                            .lock()
                                            .write_all(format!("{}", err).as_bytes())
                                            .unwrap();
                                    }
                                }
                            }

                            _ => {}
                        }
                    }
                }

                if let Some(right) = pipe_node.right() {
                    if matches!(*right, Node::Pipe(_)) {
                        pipe = self.eval(*right, pipe.clone());
                    } else {
                        match *right {
                            Node::Command(command) => {
                                match self.run_command(command, Some(pipe.clone())) {
                                    Ok(ok) => {
                                        if !ok.is_empty() {
                                            pipe = ok;
                                        }
                                    }
                                    Err(err) => {
                                        self.stderr
                                            .borrow_mut()
                                            .lock()
                                            .write_all(format!("{}", err).as_bytes())
                                            .unwrap();
                                    }
                                }
                            }

                            _ => {}
                        }
                    }
                }
            }

            Node::Command(command) => match self.run_command(command, None) {
                Ok(ok) => {
                    if !ok.is_empty() {
                        pipe = ok;
                    }
                }
                Err(err) => {
                    self.stderr
                        .borrow_mut()
                        .lock()
                        .write_all(format!("{}", err).as_bytes())
                        .unwrap();
                }
            },

            Node::VInsert(mut vinsert) => {
                let key = match vinsert.key() {
                    Some(key) => match *key {
                        Node::String(string) => string,
                        _ => String::new(),
                    },
                    None => return pipe,
                };

                let val = match vinsert.val() {
                    Some(val) => match *val {
                        Node::String(string) => string,
                        _ => String::new(),
                    },
                    None => return pipe,
                };

                self.variable.insert(&key, &val);
            }

            _ => {}
        }

        pipe
    }

    fn run_command(&mut self, command: Command, mut pipe: Option<Vec<u8>>) -> io::Result<Vec<u8>> {
        let command = match self.expand_command_node(command) {
            Some(command) => command,
            None => return Ok(Vec::default()),
        };

        // !!!added as a test!!!
        if command.0 == "cd" {
            return match cd(command
                .1
                .get(0)
                .unwrap_or(&env::var("HOME").unwrap_or_default().to_string()))
            {
                Ok(_) => Ok(Vec::default()),
                Err(err) => Err(err),
            };
        }

        let stdin = if command.2 .0.is_some() || pipe.is_some() {
            process::Stdio::piped()
        } else {
            process::Stdio::inherit()
        };

        let stdout = if command.2 .1.is_some() || pipe.is_some() {
            process::Stdio::piped()
        } else {
            process::Stdio::inherit()
        };

        let stderr = if command.2 .2.is_some() {
            process::Stdio::piped()
        } else {
            process::Stdio::inherit()
        };

        match process::Command::new(command.0.clone())
            .args(command.1)
            .stdin(stdin)
            .stdout(stdout)
            .stderr(stderr)
            .spawn()
        {
            Ok(mut result) => {
                if let Some(mut stdin) = result.stdin.take() {
                    if let Some(pipe) = pipe.clone().take() {
                        stdin.write(&pipe)?;
                    }

                    if let Some(string) = command.2 .0 {
                        let mut buffer = Vec::new();
                        fs::File::open(string)?.read_to_end(&mut buffer)?;
                        stdin.write(&buffer)?;
                    }
                }

                if let Some(mut stdout) = result.stdout.take() {
                    let mut buffer = Vec::new();

                    stdout.read_to_end(&mut buffer)?;

                    if pipe.is_some() {
                        pipe = Some(buffer.clone());
                    }

                    if let Some(string) = command.2 .1 {
                        fs::File::create(string)?.write(&mut buffer)?;
                    }
                }

                if let Some(mut stderr) = result.stderr.take() {
                    let mut buffer = Vec::new();
                    stderr.read_to_end(&mut buffer)?;
                    if let Some(string) = command.2 .2 {
                        fs::File::create(string)?.write(&mut buffer)?;
                    }
                }

                if !command.3 {
                    result.wait()?;
                }
            }
            Err(err) => {
                if matches!(err.kind(), io::ErrorKind::NotFound) {
                    let err_string = format!("command not found : {}\n", command.0);

                    match command.2 .2 {
                        Some(string) => {
                            fs::File::create(string)?.write_all(err_string.as_bytes())?;
                        }
                        None => {
                            io::stderr()
                                .lock()
                                .write_all(format!("{}", err_string).as_bytes())
                                .unwrap();
                        }
                    }
                } else {
                    return Err(err);
                }
            }
        }

        Ok(pipe.unwrap_or_default())
    }

    fn expand_command_node(
        &mut self,
        mut command: Command,
    ) -> Option<(
        String,
        Vec<String>,
        (Option<String>, Option<String>, Option<String>),
        bool,
    )> {
        let (mut program, mut args, mut redirect, mut background): (
            String,
            Vec<String>,
            (Option<String>, Option<String>, Option<String>),
            bool,
        ) = (String::new(), Vec::new(), (None, None, None), false);

        if let Some(prefix) = command.prefix() {
            match *prefix {
                Node::String(string) => {
                    program = string;
                }
                Node::VReference(key) => {
                    // program = self.variable.get(&key).unwrap_or_default();
                    program.push_str(&self.variable.get(&key).unwrap_or_default().to_string());
                }
                _ => return None,
            }
        } else {
            return None;
        }

        if let Some(mut suffix) = command.suffix() {
            while let Some(node) = suffix.pop() {
                match node {
                    Node::String(string) => args.push(string),
                    Node::VReference(key) => {
                        if let Some(val) = self.variable.get(&key) {
                            // args.push(val);
                            args.push(val.to_string());
                        }
                    }
                    Node::Redirect(rd) => match rd.file().as_ref() {
                        Node::String(string) => match rd.fd() {
                            0 => redirect.0 = Some(string.to_string()),
                            1 => redirect.1 = Some(string.to_string()),
                            2 => redirect.2 = Some(string.to_string()),
                            _ => continue,
                        },
                        _ => continue,
                    },
                    Node::Background(_) => background = true,
                    _ => {}
                }
            }
        }

        args.reverse();

        Some((program, args, redirect, background))
    }
}

fn parse(source: &str) -> Option<Node> {
    let source = source.replace("~", &env::var("HOME").unwrap_or_default());

    match Parser::new(Lexer::new(source.chars().collect())).parse() {
        Ok(ok) => ok,
        Err(err) => panic!("{:?}", err),
    }
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
        const DEFAULT_VALUE: &str = "NSH_PROMPT = \\w#";

        let path = self.lookup()?;

        fs::File::create(path)?.write_all(DEFAULT_VALUE.as_bytes())?;
        Ok(DEFAULT_VALUE.to_string())
    }

    fn read(&self) -> io::Result<String> {
        let path = self.lookup()?;

        let mut buffer = Vec::new();

        fs::File::open(path)?.read_to_end(&mut buffer)?;

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
