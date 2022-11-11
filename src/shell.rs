use crate::builtin;
use crate::error::*;
use crate::parser;
use crate::parser::lexer::Lexer;
use crate::parser::Parser;
use crate::profile;
use crate::terminal::Terminal;
use crate::variable::Variable;
use std::env;
use std::fs::File;
use std::io;
use std::io::Write;
use std::os::unix::io::AsRawFd;
use std::os::unix::io::FromRawFd;
use std::process;

pub struct Shell {
    variable: Variable,
}

impl Shell {
    pub fn new() -> Self {
        Self {
            variable: Variable::new(),
        }
    }

    pub fn initialize(&mut self) -> Result<&mut Self> {
        let node = parse(profile::read()?)?;

        self.variable = Evaluator::new(node)
            .set_variable(self.variable.variable())
            .eval()?
            .take_variable();

        Ok(self)
    }

    pub fn repl(&mut self) {
        loop {
            self.rep();
        }
    }

    fn rep(&mut self) {
        let prompt = parser::prompt::parse(
            self.variable
                .get("NSH_PROMPT".to_owned())
                .unwrap_or(&String::default()),
        );
        let mut terminal = Terminal::new();

        terminal.prompt(prompt);

        let source = match terminal.read_line() {
            Ok(string) => string,
            Err(err) => panic!("{err}"),
        };
        drop(terminal);

        match parse(source) {
            Ok(node) => match Evaluator::new(node)
                .set_variable(self.variable.variable())
                .eval()
            {
                Ok(eval) => self.variable = eval.take_variable(),
                Err(err) => {
                    io::stderr()
                        .lock()
                        .write(format!("{err}\n").as_bytes())
                        .unwrap();
                }
            },
            Err(err) => {
                io::stderr()
                    .lock()
                    .write(format!("{err}\n").as_bytes())
                    .unwrap();
            }
        }
    }
}

fn parse(source: String) -> Result<parser::Node> {
    Parser::new(Lexer::new(
        source
            .replace('\n', " ;")
            .replace('~', &env::var("HOME").unwrap_or("/".to_owned()))
            .chars()
            .collect(),
    ))
    .parse()
}

struct Evaluator {
    node: parser::Node,
    variable: Variable,
    stdin: Option<process::Stdio>,
    stdout: Option<process::Stdio>,
    stderr: Option<process::Stdio>,
}

impl Evaluator {
    pub fn new(node: parser::Node) -> Self {
        Self {
            node: node,
            variable: Variable::new(),
            stdin: None,
            stdout: None,
            stderr: None,
        }
    }

    pub fn set_variable(&mut self, variable: Variable) -> &mut Self {
        self.variable = variable;
        self
    }

    pub fn take_variable(&mut self) -> Variable {
        self.variable.variable()
    }

    pub fn eval(&mut self) -> Result<&mut Self> {
        match self.node.to_owned() {
            parser::Node::Tree(mut tree) => {
                self.stdout = None;
                while let Some(node) = tree.take() {
                    self.node = *node;
                    self.eval()?;
                }
            }

            parser::Node::Pipe(mut pipe) => {
                while let Some(node) = pipe.take() {
                    if pipe.is_pipe() {
                        self.stdout = Some(process::Stdio::piped())
                    }

                    self.node = node;
                    self.eval()?;
                }
            }

            parser::Node::Command(command) => self.run_command(command)?,

            parser::Node::Insert(mut insert) => {
                let key = match insert.take_key() {
                    Some(node) => match node {
                        parser::Node::String(string) => string,
                        _ => Err(Error::new(ErrorKind::ExecutionFailed, "".to_owned()))?,
                    },
                    None => return Ok(self),
                };

                let val = match insert.take_val() {
                    Some(node) => match node {
                        parser::Node::String(string) => string,
                        _ => Err(Error::new(ErrorKind::ExecutionFailed, "".to_owned()))?,
                    },
                    None => return Ok(self),
                };
                self.variable.insert(key, val);
            }

            _ => {}
        }

        Ok(self)
    }

    fn run_command(&mut self, mut command: parser::Command) -> Result<()> {
        let (mut program, mut args, mut is_background): (String, Vec<String>, bool) =
            (String::default(), Vec::default(), false);

        if let Some(prefix) = command.take_prefix() {
            match prefix {
                parser::Node::String(string) => {
                    program = string;
                }

                parser::Node::Reference(key) => {
                    program = self
                        .variable
                        .get(key)
                        .unwrap_or(&String::default())
                        .to_owned();
                }
                _ => Err(Error::new(ErrorKind::ExecutionFailed, "".to_owned()))?,
            }
        } else {
            return Ok(());
        }

        if program.is_empty() {
            return Ok(());
        }

        if let Some(mut suffix) = command.take_suffix() {
            while let Some(node) = suffix.take() {
                match node {
                    parser::Node::String(string) => args.push(string),
                    parser::Node::Reference(key) => {
                        if let Some(val) = self.variable.get(key) {
                            args.push(val.to_owned())
                        }
                    }
                    parser::Node::Redirect(mut redirect) => {
                        let left_fd = match redirect.take_left() {
                            Some(left) => match *left {
                                parser::Node::FD(fd) => fd,
                                _ => continue,
                            },
                            None => {
                                continue;
                            }
                        };

                        let right_file = match redirect.take_right() {
                            Some(right) => match *right {
                                parser::Node::String(string) => {
                                    match File::options()
                                        .create(true)
                                        .read(true)
                                        .write(true)
                                        .open(string)
                                    {
                                        Ok(file) => file,
                                        Err(err) => {
                                            Err(Error::new(ErrorKind::OpenFailed, err.to_string()))?
                                        }
                                    }
                                }

                                parser::Node::FD(fd) => unsafe { File::from_raw_fd(fd as i32) },
                                _ => {
                                    continue;
                                }
                            },
                            None => {
                                continue;
                            }
                        };

                        if matches!(left_fd, 0 | 1 | 2) == false {
                            unsafe {
                                libc::dup2(left_fd as i32, right_file.as_raw_fd());
                            }
                        }

                        match redirect.kind() {
                            parser::RedirectKind::Input => {
                                self.stdin = Some(process::Stdio::from(right_file));
                            }
                            parser::RedirectKind::Output => {
                                if left_fd == 2 {
                                    self.stderr = Some(process::Stdio::from(right_file));
                                } else {
                                    self.stdout = Some(process::Stdio::from(right_file));
                                }
                            }
                        }
                    }

                    parser::Node::Background(_) => {
                        is_background = true;
                    }
                    _ => {}
                }
            }
        }

        match program.as_str() {
            "abort" => builtin::abort(),
            "exit" => {
                let code = match args.pop().unwrap_or("0".to_owned()).parse::<i32>() {
                    Ok(code) => code,
                    Err(_) => Err(Error::new(
                        ErrorKind::ExecutionFailed,
                        format!("only i32 is allowed for the exit argument"),
                    ))?,
                };
                builtin::exit(code);
            }
            "cd" => {
                if let Err(err) = builtin::cd(args.pop().unwrap_or("./".to_owned())) {
                    Err(Error::new(ErrorKind::ExecutionFailed, err.to_string()))?
                }
            }
            _ => {
                match process::Command::new(&program)
                    .args(args)
                    .env(
                        "PATH",
                        self.variable
                            .get("PATH".to_owned())
                            .unwrap_or(&String::default())
                            .to_owned(),
                    )
                    .stdin(self.stdin.take().unwrap_or(process::Stdio::inherit()))
                    .stdout(self.stdout.take().unwrap_or(process::Stdio::inherit()))
                    .stderr(self.stderr.take().unwrap_or(process::Stdio::inherit()))
                    .spawn()
                {
                    Ok(mut child) => {
                        if let Some(stdout) = child.stdout {
                            self.stdin = Some(process::Stdio::from(stdout));
                        } else {
                            if is_background == false {
                                if let Err(err) = child.wait() {
                                    Err(Error::new(ErrorKind::ExecutionFailed, err.to_string()))?
                                }
                            }
                        }
                    }
                    Err(err) => {
                        if err.kind() == io::ErrorKind::NotFound {
                            Err(Error::new(
                                ErrorKind::NotFound,
                                format!("nsh command not found: {}", program),
                            ))?
                        } else {
                            Err(Error::new(ErrorKind::ExecutionFailed, err.to_string()))?
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
