use crate::builtin;
use crate::error::*;
use crate::parser;
use crate::variable;
use std::fs::File;
use std::io;
use std::os::unix::io::AsRawFd;
use std::os::unix::io::FromRawFd;
use std::process;
pub struct Evaluator {
    node: parser::Node,
    variable: variable::Variable,
    stdin: Option<process::Stdio>,
    stdout: Option<process::Stdio>,
    stderr: Option<process::Stdio>,
}

impl Evaluator {
    pub fn new(node: parser::Node) -> Self {
        Self {
            node: node,
            variable: variable::Variable::new(),
            stdin: None,
            stdout: None,
            stderr: None,
        }
    }

    pub fn set_variable(&mut self, variable: variable::Variable) {
        self.variable = variable
    }

    pub fn variable(&mut self) -> variable::Variable {
        self.variable.to_owned()
    }

    pub fn eval(&mut self) -> Result<()> {
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

            _ => {}
        }

        Ok(())
    }

    fn run_command(&mut self, mut command: parser::Command) -> Result<()> {
        let program = match command.take_prefix() {
            Some(prefix) => match prefix {
                parser::Node::String(string) => string,
                parser::Node::Reference(key) => match self.variable.get(&key) {
                    Some(val) => val.clone(),
                    None => return Ok(()),
                },
                _ => Err(Error::new(
                    ErrorKind::ExecutionFailed,
                    "a string that cannot be interpreted as a command may have been entered"
                        .to_owned(),
                ))?,
            },
            None => return Ok(()),
        };

        let mut args = Vec::new();

        let mut is_background = false;

        if let Some(mut suffix) = command.take_suffix() {
            while let Some(node) = suffix.take() {
                match node {
                    parser::Node::String(string) => args.push(string),
                    parser::Node::Reference(key) => {
                        if let Some(val) = self.variable.get(&key) {
                            args.push(val.clone());
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
                match process::Command::new(program.as_str())
                    .args(args)
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

// pub enum Stdio {
//     Stdio(process::Stdio),
//     File(File),
//     Buffer(Vec<u8>),
// }
