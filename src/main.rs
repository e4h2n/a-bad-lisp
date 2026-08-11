use clap::Parser;
use std::fs::File;
use std::io;
use std::io::prelude::*;

mod lexer;

enum Input {
    File(File),
    Stdin(io::Stdin),
}

impl Read for Input {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        return match self {
            Input::File(file) => file.read(buf),
            Input::Stdin(stdin) => stdin.read(buf),
        };
    }
}

#[derive(Parser)]
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
    for token in lexer {
        print!("{:?} ", token);
    }
    println!("");
    Ok(())
}
