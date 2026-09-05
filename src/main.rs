use clap::Parser;
use std::fs::File;
use std::io;
use std::io::prelude::*;

mod data;
mod interpreter;
mod lexer;
mod parser;

enum Input {
    File(File),
    Stdin(std::io::Stdin),
}

impl Read for Input {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        return match self {
            Input::File(file) => file.read(buf),
            Input::Stdin(stdin) => stdin.read(buf),
        };
    }
}

#[derive(clap::Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(default_value = None)]
    filename: Option<String>,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let input = match args.filename {
        Some(filename) => match File::open(filename) {
            Ok(file) => io::BufReader::new(Input::File(file)),
            Err(err) => return Err(io::Error::from(err)),
        },
        None => io::BufReader::new(Input::Stdin(io::stdin())),
    };
    let input_chars = input
        .bytes()
        .filter_map(|byte| byte.ok())
        .map(|byte| byte as char);

    let lexer = lexer::Lexer::new(input_chars);
    let result: Result<parser::AstNode, parser::ParserError> = lexer.collect();
    match result {
        Ok(ast) => {
            println!("AST:\n{:#?}", ast);
            println!("EVALS TO:\n{:#?}", ast.eval(&interpreter::starting_env()));
        }
        Err(parser::ParserError(error)) => {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, error));
        }
    }
    Ok(())
}
