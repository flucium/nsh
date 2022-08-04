mod ansi;
mod builtin;
mod manifest;
mod parser;
mod prompt;
mod shell;
mod variable;

fn main() {
    // println!("{:?}", parser::parse_args(tkns));

    // parser::parse(tkns);

    println!("{:?}", parser::Parser::new(parser::lexer::Lexer::new("echo $PWD")).parse_command());

}
