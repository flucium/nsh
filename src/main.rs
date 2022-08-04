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

    // println!("{:?}", parser::Parser::new(parser::lexer::Lexer::new("A = $B")).parse());

    //println!("{:?}",parser::lexer::Lexer::new("ls -a 2> err.txt | cat -b | rev | rev > test.txt").tokenize());
}
