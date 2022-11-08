use crate::error::*;
use crate::evaluator::Evaluator;
use crate::history::History;
use crate::parser;
use crate::parser::lexer::Lexer;
use crate::parser::Parser;
use crate::prompt;
use crate::terminal::Terminal;
use crate::variable::Variable;
use crate::profile;
use std::env;
use std::io;
use std::io::Write;


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

    pub fn initialize(&mut self) -> Result<&mut Self >{

        let node = parse( profile::read()?) ?;

        let mut evaluator = Evaluator::new(node);

        evaluator.set_variable(self.variable.to_owned());

        evaluator.eval()?;

        self.variable = evaluator.variable().to_owned();

        self.init_history();

        Ok(self)
    }

    fn init_history(&mut self) {
        if let Some(val) = self.variable.get(&"NSH_HISTORY".into()) {
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
                .get(&"NSH_PROMPT".into())
                .unwrap_or(&">".to_owned())
                .into(),
        ));

        let source = match self.terminal.read_line() {
            Ok(string) => string,
            Err(err) => panic!("{err}"),
        };

        match parse(source) {
            Ok(node) => {
                let mut evaluator = Evaluator::new(node);

                evaluator.set_variable(self.variable.to_owned());

                if let Err(err) = evaluator.eval() {
                    io::stderr()
                        .write_all(format!("{err}\n").as_bytes())
                        .unwrap();
                }

                self.variable = evaluator.variable();
            }
            Err(err) => {
                eprintln!("{:?}", err);
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


