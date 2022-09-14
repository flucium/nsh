use shell::Shell;
mod ansi;
mod builtin;
mod manifest;
mod parser;
mod prompt;
mod shell;
mod terminal;
mod variable;
use std::io::{stdout, Write};

fn main() {
    stdout()
        .lock()
        .write_all(format!("{}{}\n", manifest::name(), manifest::version()).as_bytes())
        .unwrap();

    Shell::new().initialize().repl();

}
