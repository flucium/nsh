use crate::builtin;
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

    pub fn variable(&mut self, variable: Variable) -> &mut Self {
        self.variable = variable;
        self
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
            parser::Node::Block(mut block) => {
                while let Some(node) = block.take() {
                    self.is_pipe = true;
                    self.node = *node;
                    self.eval()?;
                }
            }
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

            parser::Node::Command(command) => {
                self.run_command(command)?;
            }

            _ => {}
        }

        Ok(())
    }

    fn run_command(&mut self, mut command: parser::Command) -> io::Result<()> {
        let is_child = self.child.is_some();

        let (mut program, mut args): (String, VecDeque<String>) = (String::new(), VecDeque::new());

        //take program
        match command.take_prefix() {
            Some(prefix) => match *prefix {
                parser::Node::String(string) => program.push_str(&string),

                parser::Node::VReference(key) => {
                    program.push_str(&self.variable.get(key).unwrap_or_default())
                }

                _ => return Ok(()),
            },
            None => return Ok(()),
        };

        //take suffix
        if let Some(mut suffix) = command.take_suffix() {
            while let Some(node) = suffix.pop() {
                match node {
                    parser::Node::String(string) => {
                        args.push_front(string);
                    }
                    parser::Node::VReference(key) => {
                        args.push_front(self.variable.get(key).unwrap_or_default())
                    }

                    _ => {}
                }
            }
        }

        match program.to_lowercase().as_str() {
            "set" => {
                let key = match args.pop_front() {
                    Some(string) => string,
                    None => return Ok(()),
                };

                let val = args.pop_front().unwrap_or_default();

                builtin::set(&mut self.variable, key, val);
            }

            "unset" => {
                let key = match args.pop_front() {
                    Some(string) => string,
                    None => return Ok(()),
                };

                builtin::unset(&mut self.variable, key);
            }

            "cd" => {
                builtin::cd(args.pop_front().unwrap_or_default())?;
            }

            _ => {
                //stdin
                let stdin = if is_child {
                    process::Stdio::piped()
                } else {
                    process::Stdio::inherit()
                };

                //stdout
                let stdout = if self.is_pipe {
                    process::Stdio::inherit()
                } else {
                    process::Stdio::piped()
                };

                //stderr
                // if ...{...}else{...};
                let stderr = process::Stdio::inherit();

                match process::Command::new(program.as_str())
                    .args(args)
                    .stdin(stdin)
                    .stdout(stdout)
                    .stderr(stderr)
                    .spawn()
                {
                    Ok(mut ps_result) => {
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

                        self.child = Some(ps_result);
                    }
                    Err(err) => match err.kind() {
                        io::ErrorKind::NotFound => {
                            io::stderr().lock().write(
                                format!("nsh: command not found: {}\n", program).as_bytes(),
                            )?;
                        }
                        _ => {
                            Err(err)?;
                        }
                    },
                }
            }
        }

        Ok(())
    }
}
