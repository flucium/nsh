use shell::Shell;
mod ansi;
mod builtin;
mod manifest;
mod parser;
mod prompt;
mod shell;
mod variable;

use std::borrow::Borrow;
use std::convert::TryInto;
use std::io::stdout;
use std::io::Read;
use std::io::Write;

fn main() {
    Shell::new().initialize().repl();

}
