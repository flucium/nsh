use crate::parser;
use crate::variable::Variable;
use std::collections::VecDeque;
// use std::fs::File;
use std::io;
use std::io::Read;
use std::io::Write;
// use std::mem::ManuallyDrop;
// use std::os::unix::prelude::AsRawFd;
// use std::os::unix::prelude::FromRawFd;
use std::process;

pub struct Evaluator {
    node: parser::Node,
    child: Option<process::Child>,
    is_pipe: bool,
    variable: Variable,
}

impl Evaluator {
    pub fn new(node: parser::Node) -> Self {
        Self {
            node: node,
            child: None,
            is_pipe: false,
            variable: Variable::new(),
        }
    }

    pub fn variable(&mut self, variable: Variable) {
        self.variable = variable;
    }

    pub fn get_variable(&mut self) -> &Variable {
        &self.variable
    }

    pub fn wait(&mut self) -> io::Result<()> {
        if let Some(mut child) = self.child.take() {
            child.wait()?;
        }
        Ok(())
    }

    pub fn eval(&mut self) -> io::Result<()> {
        match self.node.to_owned() {
            parser::Node::Pipe(pipe) => {
                let mut pipe = pipe.peekable();

                while let Some(node) = pipe.next() {
                    self.is_pipe = !pipe.peek().is_some();

                    self.node = node;

                    self.eval()?;
                }
            }

            parser::Node::VInsert(mut vinsert) => {
                let key = if let Some(key) = vinsert.take_key() {
                    match *key {
                        parser::Node::String(string) => string,
                        _ => return Ok(()),
                    }
                } else {
                    return Ok(());
                };

                let val = if let Some(key) = vinsert.take_val() {
                    match *key {
                        parser::Node::String(string) => string,
                        _ => return Ok(()),
                    }
                } else {
                    return Ok(());
                };

                self.variable.insert(key, val);
            }

            parser::Node::Command(mut command) => {
                let mut program = String::new();
                let mut args = VecDeque::new();
                // let mut ifiles = VecDeque::new();
                // let mut ofiles = VecDeque::new();
                // let mut efiles = VecDeque::new();

                match command.take_prefix() {
                    Some(prefix) => match *prefix {
                        parser::Node::String(string) => {
                            program = string;
                        }

                        parser::Node::VReference(key) => {
                            program = self.variable.get(key).unwrap_or_default();
                        }
                        _ => {}
                    },
                    None => {}
                }

                if let Some(mut suffix) = command.take_suffix() {
                    while let Some(node) = suffix.pop() {
                        match node {
                            parser::Node::String(string) => {
                                args.push_front(string);
                            }
                            // parser::Node::VReference(key)=>{}
                            // parser::Node::Redirect(redirect) => {}
                            _ => {}
                        }
                    }
                }

                let is_child = self.child.is_some();

                let mut ps_result = process::Command::new(program.clone())
                    .args(args)
                    .stdin(if is_child {
                        process::Stdio::piped()
                    } else {
                        process::Stdio::inherit()
                    })
                    .stdout(if self.is_pipe {
                        process::Stdio::inherit()
                    } else {
                        process::Stdio::piped()
                    })
                    .stderr(process::Stdio::inherit())
                    .spawn()?;

                if let Some(stdin) = ps_result.stdin.as_mut() {
                    if let Some(mut child) = self.child.take() {
                        if let Some(stdout) = child.stdout.take() {
                            let bytes = stdout.bytes();
                            for byte in bytes {
                                let byte = vec![byte.unwrap()];
                                stdin.write(&byte)?;
                            }
                        }
                    }
                }

                if is_child {
                    ps_result.wait()?;
                }

                self.child = Some(ps_result)
            }

            _ => {}
        }

        Ok(())
    }
}
