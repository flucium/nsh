use shell::Shell;
mod ansi;
mod builtin;
mod evaluator;
mod manifest;
mod parser;
mod prompt;
mod shell;
mod terminal;
mod variable;
mod history;
fn main() {
    Shell::new().initialize().repl();
}
