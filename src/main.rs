use shell::Shell;
mod ansi;
mod builtin;
mod parser;
mod prompt;
mod shell;
mod variable;
mod manifest;

fn main() {
    Shell::new().initialize().repl();
}
