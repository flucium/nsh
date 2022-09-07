use shell::Shell;
mod ansi;
mod builtin;
mod parser;
mod prompt;
mod shell;
mod variable;

fn main() {
    Shell::new().initialize().repl();
}
