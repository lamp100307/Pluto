mod core;

use core::lexer::lex;
use core::parser::Parser;
use core::interpreter::Interpreter;

use std::fs::File;
use std::io::Read;
use std::io::{self, Write};
use std::time::Instant;

fn main() {
    let args = std::env::args().collect::<Vec<_>>();

    if args.len() < 2 {
        //start in repl
        run_repl();
    } else {
        //start with file
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
                //debug
                if debug {
                    println!("ast: {:?}", &ast);
                }

                let mut interpreter = Interpreter::new();

                //time
                let start_time = Instant::now();
                let result = interpreter.interpret(&ast);
                let duration = start_time.elapsed();

                if debug {
                    println!("Execution time: {:?}", duration);
                }

                result.expect("Interpreter error");
            }
            Err(err) => {
                eprintln!("Parser error: {}", err);
            }
        };
    }
}

fn run_repl() {
    //repl
    println!("Pluto REPL (Interactive Mode)");
    println!("Type 'exit' or 'quit' to exit");
    println!("-----------------------------");

    let mut interpreter = Interpreter::new();
    let mut input_buffer = String::new();

    loop {
        print!(">>> ");
        io::stdout().flush().unwrap();

        let mut line = String::new();
        io::stdin().read_line(&mut line).unwrap();

        let line = line.trim();

        // commands
        if line == "exit" || line == "quit" {
            println!("Goodbye!");
            break;
        }

        // skip empty lines
        if line.is_empty() {
            continue;
        }

        // adding line to buffer (for multiline expressions)
        if !input_buffer.is_empty() {
            input_buffer.push_str(" ");
        }
        input_buffer.push_str(line);

        // check, if expression is complete (find braces, brackets, etc.)
        if is_complete_expression(&input_buffer) {
            match try_execute_code(&input_buffer, &mut interpreter) {
                Ok(_) => {
                    //If code completed, clear buffer
                    input_buffer.clear();
                }
                Err(err) => {
                    //If it's multiline expression (function, cycle, etc.)
                    if err.contains("Unexpected") || err.contains("Expected") {
                        //Continue to next line
                        continue;
                    } else {
                        eprintln!("Error: {}", err);
                        input_buffer.clear();
                    }
                }
            }
        }
    }
}

fn is_complete_expression(code: &str) -> bool {
    // If have unclosed braces, brackets, etc.
    let mut brace_count = 0;
    let mut paren_count = 0;
    let mut bracket_count = 0;

    for c in code.chars() {
        match c {
            '{' => brace_count += 1,
            '}' => brace_count -= 1,
            '(' => paren_count += 1,
            ')' => paren_count -= 1,
            '[' => bracket_count += 1,
            ']' => bracket_count -= 1,
            _ => {}
        }
    }

    //check ;
    brace_count == 0 && paren_count == 0 && bracket_count == 0
}

fn try_execute_code(code: &str, interpreter: &mut Interpreter) -> Result<(), String> {
    let tokens = lex(code);
    let mut parser = Parser::new(tokens);

    match parser.parse() {
        Ok(ast) => {
            match interpreter.interpret(&ast) {
                Ok(result) => {
                    // dont print voids
                    if !matches!(result, core::interpreter::RuntimeValue::Void) {
                        println!("{:?}", result);
                    }
                    Ok(())
                }
                Err(err) => Err(err),
            }
        }
        Err(err) => Err(err),
    }
}