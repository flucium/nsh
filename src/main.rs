mod ansi;
mod builtin;
mod manifest;
mod parser;
mod prompt;
mod shell;
mod variable;

fn main() {
    let mut parser = parser::Parser::new(parser::lexer::Lexer::new("ls".chars().collect()));

    println!("{:?}",parser.parse());
}
