use crate::parser;
use std::collections::VecDeque;
use std::fs::File;
use std::io;
use std::io::stdout;
use std::io::Read;
use std::io::Stdout;
use std::io::Write;
use std::mem::ManuallyDrop;
use std::os::unix::prelude::AsRawFd;
use std::os::unix::prelude::FromRawFd;
use std::process;

pub struct Evaluator {
    node: parser::Node,
    buffer: Option<Vec<u8>>,
    child: Option<process::Child>,
}

impl Evaluator {
    pub fn new(node: parser::Node) -> Self {
        Self {
            node: node,
            buffer: None,
            child: None,
        }
    }


    pub fn eval(&mut self) -> io::Result<()> {
        match self.node.to_owned() {
            parser::Node::Pipe(mut pipe) => {
                while let Some(node) = pipe.take_left().or_else(|| pipe.take_right()) {
                    self.node = *node;
                    self.eval()?;
                }
            }

            parser::Node::Command(mut command) => {
                let mut program = String::new();
                let mut args = VecDeque::new();
                // let mut ifiles = VecDeque::new();
                // let mut ofiles = VecDeque::new();
                // let mut efiles = VecDeque::new();

                match command.take_prefix() {
                    Some(prefix) => {
                        match *prefix {
                            parser::Node::String(string) => {
                                program = string;
                            }
                            // parser::Node::VReference(key)=>{}
                            _ => {}
                        }
                    }
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


                process::Command::new(program).spawn().unwrap().wait();
                
                // self.child = Some(ps_result);
            }

            _ => {}
        }

        Ok(())
    }
}
