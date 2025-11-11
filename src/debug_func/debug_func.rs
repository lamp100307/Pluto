extern crate colored;

use colored::Colorize;
use crate::core::lexer::lexer_error::LexerError;
use crate::core::parser::ast_nodes::AstNode;
use crate::core::parser::parser_error::ParserError;
use crate::core::interpreter::interpreter::RuntimeValue;

// Вспомогательная функция для показа контекста ошибки
fn get_error_context(code: &str, pos: usize) -> Option<String> {
    if pos >= code.len() {
        return None;
    }

    // Находим строку и позицию в строке
    let mut line_start = 0;
    let mut line_num = 1;
    let mut col_num = 1;

    for (i, ch) in code.chars().enumerate() {
        if i == pos {
            break;
        }
        if ch == '\n' {
            line_start = i + 1;
            line_num += 1;
            col_num = 1;
        } else {
            col_num += 1;
        }
    }

    // Находим конец строки
    let line_end = code[line_start..].find('\n')
        .map(|pos| line_start + pos)
        .unwrap_or(code.len());

    let line_content = &code[line_start..line_end];

    Some(format!("line {}:{} - '{}'", line_num, col_num, line_content.trim()))
}

pub fn debug_tokens(tokens: &[(String, String)]) {
    println!("{}", "=== TOKENS ===".cyan().bold());
    for (i, (token_type, token_value)) in tokens.iter().enumerate() {
        println!("{:3}: {:12} = '{}'",
                 i.to_string().blue(),
                 token_type.green(),
                 token_value.yellow()
        );
    }
    println!();
}


pub fn parse_error(errors: Vec<LexerError>, code: &str) {
    eprintln!("{}:", "LEXICAL ANALYSIS FAILED".red().bold());
    for error in errors {
        match error {
            LexerError::UnexpectedCharacter { char, pos } => {
                eprintln!("  {}: at position {}", "Unexpected Character".red().bold(), pos);
                eprintln!("  {}: '{}'", "Character".red(), char);
                if let Some(context) = get_error_context(&code, pos) {
                    eprintln!("  {}: {}", "Context".blue(), context.blue());
                }
            }
        }
        eprintln!();
    }
}

// Функция для красивого вывода ошибок
pub fn parse_parser_error(errors: Vec<ParserError>, code: &str) {
    eprintln!("{}", "AST BUILDING FAILED".red().bold());
    eprintln!("{}", "=".repeat(50).yellow());
    
    for (i, error) in errors.iter().enumerate() {
        eprintln!("{} {}{}", "Error".red().bold(), (i + 1).to_string().red(), ":".red());
        
        match error {
            ParserError::NotImplemented { function, node_info, pos } => {
                eprintln!("  {}: {}", "Type".yellow(), "Not implemented".red().bold());
                eprintln!("  {}: {}", "Function".yellow(), function);
                eprintln!("  {}: {}", "Node info".yellow(), node_info);
                eprintln!("  {}: {}", "Position".yellow(), pos);
                if let Some(context) = get_error_context(code, *pos) {
                    eprintln!("  {}: {}", "Context".blue(), context);
                    // Показываем указатель на позицию ошибки
                    let pointer_pos = context.find("...").map_or(20, |p| p + 3);
                    eprintln!("  {}: {}{}", "Where".blue(), " ".repeat(pointer_pos), "^".red().bold());
                }
            },
            ParserError::SyntaxError { pos, expected, found } => {
                eprintln!("  {}: {}", "Type".yellow(), "Syntax error".red().bold());
                eprintln!("  {}: {}", "Expected".yellow(), expected.green());
                eprintln!("  {}: {}", "Found".yellow(), found.red());
                eprintln!("  {}: {}", "Position".yellow(), pos);
                if let Some(context) = get_error_context(code, *pos) {
                    eprintln!("  {}: {}", "Context".blue(), context);
                    let pointer_pos = context.find("...").map_or(20, |p| p + 3);
                    eprintln!("  {}: {}{}", "Where".blue(), " ".repeat(pointer_pos), "^".red().bold());
                }
            },
            ParserError::UnexpectedEof { pos } => {
                eprintln!("  {}: {}", "Type".yellow(), "Unexpected end of file".red().bold());
                eprintln!("  {}: {}", "Position".yellow(), pos);
                if let Some(context) = get_error_context(code, *pos) {
                    eprintln!("  {}: {}", "Context".blue(), context);
                    eprintln!("  {}: {}", "Where".blue(), "↑ (unexpected end)".red().bold());
                }
            },
            ParserError::TypeError { expected, actual, context } => {
                eprintln!("  {}: {}", "Type".yellow(), "Type error".red().bold());
                eprintln!("  {}: {}", "Expected type".yellow(), expected.green());
                eprintln!("  {}: {}", "Actual type".yellow(), actual.red());
                eprintln!("  {}: {}", "Context".yellow(), context);
            },
            ParserError::UnexpectedToken { token_type, token_value, pos } => {
                eprintln!("  {}: {}", "Type".yellow(), "Unexpected token".red().bold());
                eprintln!("  {}: {} = '{}'", "Token".yellow(), token_type, token_value.red());
                eprintln!("  {}: {}", "Position".yellow(), pos);
                if let Some(context) = get_error_context(code, *pos) {
                    eprintln!("  {}: {}", "Context".blue(), context);
                    let pointer_pos = context.find("...").map_or(20, |p| p + 3);
                    eprintln!("  {}: {}{}", "Where".blue(), " ".repeat(pointer_pos), "^".red().bold());
                }
            },
        }
        eprintln!(); 
    }
    
    eprintln!("{}", "=".repeat(50).yellow());
    eprintln!("{} error(s) found", errors.len().to_string().red().bold());
}

pub fn print_ast(ast: &[AstNode]) {
    println!("{}", "Abstract Syntax Tree:".green().bold());
    println!("{}", "=".repeat(40).blue());
    
    for (i, node) in ast.iter().enumerate() {
        println!("{} {}:", "Node".cyan(), i + 1);
        print_node(node, 0);
        println!();
    }
}

fn print_node(node: &AstNode, depth: usize) {
    let indent = "  ".repeat(depth);
    let next_depth = depth + 1;
    
    match node {
        AstNode::Number(value) => {
            println!("{}└── {}: {}", indent, "Number".yellow(), value);
        }
        AstNode::String(value) => {
            println!("{}└── {}: \"{}\"", indent, "String".yellow(), value);
        }
        AstNode::BinaryOp { op, left, right } => {
            println!("{}└── {}: {}", indent, "BinaryOp".cyan(), op);
            
            // Левая часть
            print!("{}  ├── {}", indent, "left:".magenta());
            match &**left {
                AstNode::BinaryOp { .. } => {
                    println!();
                    print_node(&left, next_depth + 1);
                }
                _ => {
                    print!(" ");
                    print_simple_node(&left, next_depth);
                }
            }
            
            // Правая часть  
            print!("{}  └── {}", indent, "right:".magenta());
            match &**right {
                AstNode::BinaryOp { .. } => {
                    println!();
                    print_node(&right, next_depth + 1);
                }
                _ => {
                    print!(" ");
                    print_simple_node(&right, next_depth);
                }
            }
        }
        AstNode::Identifier(name) => {
            println!("{}└── {}: {}", indent, "Identifier".green(), name);
        }
        AstNode::FunctionCall { name, args } => {
            println!("{}└── {}: {}", indent, "FunctionCall".blue(), name);
            println!("{}  └── {}:", indent, "arguments:".magenta());
            for (i, arg) in args.iter().enumerate() {
                print!("{}    {}", indent, if i == args.len() - 1 { "└── " } else { "├── " });
                match arg {
                    AstNode::BinaryOp { .. } | AstNode::UnaryOpTT { .. } | AstNode::FunctionCall { .. } => {
                        println!();
                        print_node(arg, next_depth + 2);
                    }
                    _ => {
                        print_simple_node(arg, next_depth + 1);
                    }
                }
            }
        }
        _ => {
            println!("{}└── {}: {:?}", indent, "UnknownNode".red(), node);
        }
    }
}

// Вспомогательная функция для простых узлов
fn print_simple_node(node: &AstNode, depth: usize) {
    let indent = "  ".repeat(depth);
    
    match node {
        AstNode::Number(value) => println!("{}", value),
        AstNode::String(value) => println!("\"{}\"", value),
        AstNode::Identifier(name) => println!("{}", name),
        AstNode::BinaryOp { op, left, right } => {
            println!("BinaryOp({})", op);
            print!("{}  left: ", indent);
            print_simple_node(&left, depth + 1);
            print!("{}  right: ", indent);
            print_simple_node(&right, depth + 1);
        }
        _ => println!("{:?}", node),
    }
}

pub fn print_runtime_value(v: RuntimeValue) {
    match v {
        RuntimeValue::Number(n) => println!("Number: {}", n),
        RuntimeValue::Array(a) => {
            print!("Array: [");
            for (i, item) in a.iter().enumerate() {
                if i > 0 {
                    print!(", ");
                }
                match item {
                    RuntimeValue::Number(n) => print!("{}", n),
                    RuntimeValue::String(s) => print!("\"{}\"", s),
                    RuntimeValue::Boolean(b) => print!("{}", b),
                    RuntimeValue::Float(f) => print!("{}", f),
                    _ => print!("{:?}", item), 
                }
            }
            println!("]");
        },
        RuntimeValue::Float(f) => println!("Float: {}", f),
        RuntimeValue::Boolean(b) => println!("Boolean: {}", b), 
        RuntimeValue::ConstValue(c) => {
            print!("Const value: ");
            print_runtime_value(*c);
        },
        RuntimeValue::String(s) => println!("String: \"{}\"", s),
        RuntimeValue::Void => println!("Void"), // Исправлено: убрали скобки ()
        RuntimeValue::Regex(r) => println!("Regex: /{}/", r),
        RuntimeValue::ReturnValue(r) => {
            print!("Return value: ");
            print_runtime_value(*r); 
        },
    }
}