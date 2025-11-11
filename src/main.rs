extern crate colored;
mod core;
mod debug_func;

use std::fs::File;
use std::io::Read;
use colored::*;
use core::lexer::lexer::lex;
use core::parser::Parser;
use core::interpreter::interpreter::Interpreter;
use debug_func::debug_func::{debug_tokens, parse_error, parse_parser_error, print_ast, print_runtime_value};
 
fn main() {
    let args = std::env::args().collect::<Vec<_>>();

    if args.len() < 2 {
        eprintln!("{}: {}", "ERROR".red().bold(), "Usage: program <filename> [debug]");
        std::process::exit(1);
    }
    let debug = args.get(2).map(|arg| arg.as_str() == "debug").unwrap_or(false);

    let mut file = match File::open(&args[1]) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("{}: Failed to open file '{}': {}",
                      "ERROR".red().bold(),
                      args[1].yellow(),
                      e.to_string().red());
            std::process::exit(1);
        }
    };

    let mut code = String::new();
    if let Err(e) = file.read_to_string(&mut code) {
        eprintln!("{}: Failed to read file '{}': {}",
                  "ERROR".red().bold(),
                  args[1].yellow(),
                  e.to_string().red());
        std::process::exit(1);
    }

    let tokens_result = lex(&code);

    let tokens = match tokens_result {
        Ok(tokens) => {
            println!("{}", "✅ LEXING COMPLETED SUCCESSFULLY".green().bold());

            if debug {
                debug_tokens(&tokens);
            }

            tokens
        }
        Err(errors) => {
            parse_error(errors, &code);
            std::process::exit(1);
        }
    };

    let mut parser = Parser::new(tokens);
    let parse_result = parser.parse();
    
    let ast = match parse_result {
        Ok(ast) => {
            println!("{}", "✅ PARSING COMPLETED SUCCESSFULLY".green().bold());
            
            if debug {
                print_ast(&ast);
            }

            ast
        }
        Err(errors) => {
            parse_parser_error(errors, &code);
            std::process::exit(2);
        }
    };

    // Исправленная часть для интерпретатора
    let mut interpreter = Interpreter::new();
    let result = interpreter.interpret(ast);
    
    match result {
        Ok(value) => {
            println!("{}", "✅ INTERPRETATION COMPLETED SUCCESSFULLY".green().bold());
            print_runtime_value(value);
        }
        Err(error) => {
            eprintln!("{}: {}", "❌ INTERPRETATION FAILED".red().bold(), error);
            std::process::exit(3);
        }
    }
}