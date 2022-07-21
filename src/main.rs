mod parser;
mod prompt;
mod variable;
mod ansi;
mod manifest;
mod builtin;
mod shell;

fn main() {
    parser::parse();
}
