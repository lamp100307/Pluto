mod core;

use core::lexer::lex;
use core::parser::Parser;
use core::interpreter::Interpreter;

use std::fs::File;
use std::io::Read;

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() < 2 {
        panic!("Usage: {} <filename> <debug (can be empty)>", args[0]);
    }

    let mut debug = false;

    if args.len() > 2 {
        debug = args[2] == "debug";
    }

    let mut file = File::open(&args[1]).unwrap();
    let mut code = String::new();
    file.read_to_string(&mut code).unwrap();

    let tokens = lex(&*code);
    if debug {
        println!("tokens: {:?}", tokens);
    }

    let mut parser = Parser::new(tokens);
    match parser.parse() {
        Ok(ast) => {
            if debug {
                println!("ast: {:?}", &ast);
            }
            let mut interpreter = Interpreter::new();
            interpreter.interpret(&ast).expect("Interpreter error");
        }
        Err(err) => {
            eprintln!("Parser error: {}", err);
        }
    };
}