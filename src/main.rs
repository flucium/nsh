use shell::Shell;
mod ansi;
mod builtin;
mod manifest;
mod parser;
mod prompt;
mod shell;
mod terminal;
mod variable;


fn main() {
    Shell::new().initialize().repl();
}
