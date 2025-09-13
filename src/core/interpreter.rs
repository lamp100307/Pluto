use super::parser::{AstNode, Parser};
use super::parser::Type;
extern crate rand;

use rand::Rng;
use std::fs::File;
use std::io;
use std::io::{Read, Write};
use crate::core::lexer::lex;

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeValue {
    Number(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Array(Vec<RuntimeValue>),
    Void,
    ReturnValue(Box<RuntimeValue>),
    ConstValue(Box<RuntimeValue>),
}

#[derive(Debug)]
pub struct Interpreter {
    variables: std::collections::HashMap<String, RuntimeValue>,
    functions: std::collections::HashMap<String, (Vec<String>, Vec<AstNode>)>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            variables: std::collections::HashMap::new(),
            functions: std::collections::HashMap::new(),
        }
    }

    pub fn interpret(&mut self, nodes: &[AstNode]) -> Result<RuntimeValue, String> {
        let mut last_value = RuntimeValue::Void;

        for node in nodes {
            last_value = self.interpret_single(node)?;
            if let RuntimeValue::ReturnValue(_) = last_value {
                break;
            }
        }

        Ok(last_value)
    }

    pub fn interpret_single(&mut self, node: &AstNode) -> Result<RuntimeValue, String> {
        match node {
            AstNode::Number(n) => Ok(RuntimeValue::Number(*n)),
            AstNode::Float(f) => Ok(RuntimeValue::Float(*f)),
            AstNode::String(s) => Ok(RuntimeValue::String((*s.to_string()).parse().unwrap())),
            AstNode::Boolean(b) => Ok(RuntimeValue::Boolean(*b)),
            AstNode::Array(elements) => Ok(RuntimeValue::Array(elements.iter().map(|e| self.interpret_single(e)).collect::<Result<Vec<RuntimeValue>, String>>()?)),
            AstNode::Index { array, index } => {
                let array_val = self.interpret_single(array)?;
                let index_val = self.interpret_single(index)?;

                match (array_val, index_val) {
                    (RuntimeValue::Array(arr), RuntimeValue::Number(idx)) => {
                        if idx < 0 || idx >= arr.len() as i64 {
                            Err(format!("Index {} out of bounds for array of length {}", idx, arr.len()))
                        } else {
                            Ok(arr[idx as usize].clone())
                        }
                    }
                    _ => Err("Indexing only supported for arrays with numeric indices".to_string()),
                }
            }

            AstNode::MethodCall { object, method, args } => {
                let obj_val = self.interpret_single(object)?;
                let evaluated_args = args.iter()
                    .map(|arg| self.interpret_single(arg))
                    .collect::<Result<Vec<_>, _>>()?;

                match (obj_val, method.as_str()) {
                    (RuntimeValue::Array(mut arr), "push") => {
                        if evaluated_args.len() != 1 {
                            return Err("Array.add() expects exactly 1 argument".to_string());
                        }
                        arr.push(evaluated_args[0].clone());

                        if let AstNode::Identifier(id) = object.as_ref() {
                            self.variables.insert(id.clone(), RuntimeValue::Array(arr.clone()));
                        }

                        Ok(RuntimeValue::Array(arr))
                    }
                    (RuntimeValue::String(s), "chars") => {
                        if !evaluated_args.is_empty() {
                            return Err("String.chars() expects no arguments".to_string());
                        }

                        let chars: Vec<RuntimeValue> = s.chars()
                            .map(|c| RuntimeValue::String(c.to_string()))
                            .collect();

                        Ok(RuntimeValue::Array(chars))
                    }
                    _ => Err(format!("Method {} not supported for this type", method)),
                }
            }

            AstNode::Assign { left, right } => {
                let right_val = self.interpret_single(right)?;

                match left.as_ref() {
                    AstNode::Identifier(id) => {
                        if let Some(existing_val) = self.variables.get(id) {
                            if let RuntimeValue::ConstValue(_) = existing_val {
                                return Err(format!("Cannot reassign constant {}", id));
                            }

                            match (existing_val, &right_val) {
                                (RuntimeValue::Number(_), RuntimeValue::Number(_)) => {},
                                (RuntimeValue::Float(_), RuntimeValue::Float(_)) => {},
                                (RuntimeValue::Boolean(_), RuntimeValue::Boolean(_)) => {},
                                (RuntimeValue::String(_), RuntimeValue::String(_)) => {},
                                (RuntimeValue::Array(_), RuntimeValue::Array(_)) => {},
                                _ => return Err(format!("Type mismatch for variable {}", id)),
                            }
                        }

                        self.variables.insert(id.clone(), right_val);
                        Ok(RuntimeValue::Void)
                    }
                    AstNode::Index { array, index } => {
                        let array_val = self.interpret_single(array)?;
                        let index_val = self.interpret_single(index)?;

                        match (array_val, index_val) {
                            (RuntimeValue::Array(mut arr), RuntimeValue::Number(idx)) => {
                                if idx < 0 || idx >= arr.len() as i64 {
                                    return Err(format!("Index {} out of bounds for array of length {}", idx, arr.len()));
                                }

                                if let AstNode::Identifier(id) = array.as_ref() {
                                    if let Some(RuntimeValue::ReturnValue(_)) = self.variables.get(id) {
                                        return Err(format!("Cannot modify constant array {}", id));
                                    }
                                }

                                arr[idx as usize] = right_val;

                                if let AstNode::Identifier(id) = array.as_ref() {
                                    self.variables.insert(id.clone(), RuntimeValue::Array(arr.clone()));
                                }

                                Ok(RuntimeValue::Array(arr))
                            }
                            _ => Err("Index assignment only supported for arrays with numeric indices".to_string()),
                        }
                    }
                    _ => Err("Assignment to non-identifier or non-index".to_string()),
                }
            }

            AstNode::ToType { types, expr } => {
                let value = self.interpret_single(expr)?;

                match (value, types) {
                    // Преобразование в int
                    (RuntimeValue::Number(n), Type::Int) => Ok(RuntimeValue::Number(n)),
                    (RuntimeValue::Float(f), Type::Int) => Ok(RuntimeValue::Number(f as i64)),
                    (RuntimeValue::Boolean(b), Type::Int) => Ok(RuntimeValue::Number(if b { 1 } else { 0 })),
                    (RuntimeValue::String(s), Type::Int) => {
                        s.parse::<i64>()
                            .map(RuntimeValue::Number)
                            .map_err(|_| format!("Cannot convert string '{}' to int", s))
                    }

                    // Преобразование в float
                    (RuntimeValue::Number(n), Type::Float) => Ok(RuntimeValue::Float(n as f64)),
                    (RuntimeValue::Float(f), Type::Float) => Ok(RuntimeValue::Float(f)),
                    (RuntimeValue::Boolean(b), Type::Float) => Ok(RuntimeValue::Float(if b { 1.0 } else { 0.0 })),
                    (RuntimeValue::String(s), Type::Float) => {
                        s.parse::<f64>()
                            .map(RuntimeValue::Float)
                            .map_err(|_| format!("Cannot convert string '{}' to float", s))
                    }

                    // Преобразование в bool
                    (RuntimeValue::Number(n), Type::Bool) => Ok(RuntimeValue::Boolean(n != 0)),
                    (RuntimeValue::Float(f), Type::Bool) => Ok(RuntimeValue::Boolean(f != 0.0)),
                    (RuntimeValue::Boolean(b), Type::Bool) => Ok(RuntimeValue::Boolean(b)),
                    (RuntimeValue::String(s), Type::Bool) => {
                        Ok(RuntimeValue::Boolean(!s.is_empty() && s != "false" && s != "0"))
                    }

                    // Преобразование в string
                    (RuntimeValue::Number(n), Type::String) => Ok(RuntimeValue::String(n.to_string())),
                    (RuntimeValue::Float(f), Type::String) => Ok(RuntimeValue::String(f.to_string())),
                    (RuntimeValue::Boolean(b), Type::String) => Ok(RuntimeValue::String(b.to_string())),
                    (RuntimeValue::String(s), Type::String) => Ok(RuntimeValue::String(s)),

                    // Преобразование в array
                    (RuntimeValue::Array(arr), Type::Array) => Ok(RuntimeValue::Array(arr)),
                    (value, Type::Array) => {
                        // Создаем массив из одного элемента
                        Ok(RuntimeValue::Array(vec![value]))
                    }

                    // Неподдерживаемые преобразования
                    (value, target_type) => {
                        Err(format!("Cannot convert {:?} to {:?}", value, target_type))
                    }
                }
            }

            AstNode::Identifier(id) => {
                self.variables.get(id)
                    .cloned()
                    .ok_or_else(|| format!("Undefined variable: {}", id))
            }
            AstNode::Delete(expr) => {
                let value = self.interpret_single(expr)?;

                if let AstNode::Identifier(id) = expr.as_ref() {
                    self.variables.remove(id);
                    Ok(RuntimeValue::Void)
                } else {
                    Ok(value)
                }
            }
            AstNode::Return(expr) => {
                let value = self.interpret_single(expr)?;
                Ok(RuntimeValue::ReturnValue(Box::new(value)))
            }
            AstNode::Function { name, params, body } => {
                let param_names = params.iter().map(|p| p.name.clone()).collect();
                self.functions.insert(
                    name.clone(),
                    (param_names, body.clone())
                );
                Ok(RuntimeValue::Void)
            }
            AstNode::FunctionCall { name, args } => {
                let (params, body) = self.functions.get(name)
                    .ok_or_else(|| format!("Undefined function: {}", name))?
                    .clone();

                if args.len() != params.len() {
                    return Err(format!(
                        "Function {} expects {} arguments, got {}",
                        name,
                        params.len(),
                        args.len()
                    ));
                }

                let mut evaluated_args = Vec::new();
                for arg in args {
                    evaluated_args.push(self.interpret_single(arg)?);
                }

                let mut new_vars = std::collections::HashMap::new();
                for (param, arg) in params.iter().zip(evaluated_args) {
                    new_vars.insert(param.clone(), arg);
                }

                let mut new_interpreter = Interpreter {
                    variables: new_vars,
                    functions: self.functions.clone(),
                };

                match new_interpreter.interpret(&body)? {
                    RuntimeValue::ReturnValue(v) => Ok(*v),
                    other => Ok(other),                  }
            }
            AstNode::Print { left } => {
                let value = self.interpret_single(left)?;
                match value {
                    RuntimeValue::Number(v) => println!("{}", v),
                    RuntimeValue::Float(v) => println!("{}", v),
                    RuntimeValue::String(v) => println!("{}", v),
                    RuntimeValue::Boolean(v) => println!("{}", v),
                    RuntimeValue::Array(v) => {
                        self.print_array(&v);
                    }
                    RuntimeValue::Void => println!("()"),
                    _ => println!("()")
                }
                Ok(RuntimeValue::Void)
            }
            AstNode::Let { name, is_const, var_type, var_value } => {
                let value = self.interpret_single(var_value)?;

                if let Some(t) = var_type {
                    match (&value, t) {
                        (RuntimeValue::Number(_), Type::Int) => {},
                        (RuntimeValue::Float(_), Type::Float) => {},
                        (RuntimeValue::Boolean(_), Type::Bool) => {},
                        (RuntimeValue::String(_), Type::String) => {},
                        (RuntimeValue::Array(_), Type::Array) => {},
                        _ => return Err(format!("Type mismatch for variable {}", name)),
                    }
                }

                if self.variables.contains_key(name) {
                    return Err(format!("Variable {} already defined", name));
                }

                if *is_const {
                    // Для констант используем специальный RuntimeValue
                    self.variables.insert(name.clone(), RuntimeValue::ConstValue(Box::new(value)));
                } else {
                    self.variables.insert(name.clone(), value);
                }
                Ok(RuntimeValue::Void)
            }
            AstNode::Import {path} => {
                let mut file = File::open("./src/".to_string() + path).unwrap();
                let mut contents = String::new();
                file.read_to_string(&mut contents).unwrap();

                let tokens = lex(&*contents);
                let ast = Parser::new(tokens.clone()).parse()?;

                self.interpret(&ast)
            }
            AstNode::Input {placeholder} => {
                let mut input = String::new();
                print!("{}", placeholder);
                io::stdout().flush().unwrap();
                io::stdin().read_line(&mut input).unwrap();
                Ok(RuntimeValue::String(input.trim().to_string()))
            }
            AstNode::Random { left, right } => {
                let left_val = self.interpret_single(left)?;
                let right_val = self.interpret_single(right)?;

                match (left_val, right_val) {
                    (RuntimeValue::Number(l), RuntimeValue::Number(r)) => {
                        if l < r {
                            Ok(RuntimeValue::Number(rand::thread_rng().gen_range(l..=r)))
                        } else {
                            Ok(RuntimeValue::Number(rand::thread_rng().gen_range(r..=l)))
                        }
                    }
                    _ => Err("Random range requires numbers".to_string()),
                }
            }
            AstNode::If { condition, body, else_body } => {
                let condition_val = self.interpret_single(condition)?;

                match condition_val {
                    RuntimeValue::Boolean(true) => self.interpret(body),
                    RuntimeValue::Boolean(false) => match else_body {
                        Some(body) => self.interpret(body),
                        None => Ok(RuntimeValue::Void),
                    },
                    RuntimeValue::Number(n) => if n > 0 { self.interpret(body) } else { match else_body { Some(body) => self.interpret(body), None => Ok(RuntimeValue::Void) } },
                    _ => Err("Condition must be boolean".to_string()),
                }
            }
            AstNode::While { condition, body } => {
                while self.interpret_single(condition)? == RuntimeValue::Boolean(true) {
                    self.interpret(body)?;
                }
                Ok(RuntimeValue::Void)
            }
            AstNode::For {init, condition, increment, body} => {
                let mut _value = self.interpret_single(&*init.clone())?;

                while self.interpret_single(&*condition.clone())? == RuntimeValue::Boolean(true) {
                    self.interpret(&*body.clone())?;
                    _value = self.interpret_single(&*increment.clone())?;
                }
                Ok(RuntimeValue::Void)
            }
            AstNode::ForIn { var, iterable, body } => {
                let iterable_val = self.interpret_single(iterable)?;

                match iterable_val {
                    RuntimeValue::Array(items) => {
                        for item in items {
                            self.variables.insert(var.clone(), item);

                            self.interpret(body)?;
                        }
                        Ok(RuntimeValue::Void)
                    }
                    RuntimeValue::Number(num) => {
                        for i in 1..=num {
                            self.variables.insert(var.clone(), RuntimeValue::Number(i));
                            self.interpret(body)?;
                        }
                        Ok(RuntimeValue::Void)
                    }
                    RuntimeValue::String(s) => {
                        for c in s.chars() {
                            self.variables.insert(var.clone(), RuntimeValue::String(c.to_string()));
                            self.interpret(body)?;
                        }
                        Ok(RuntimeValue::Void)
                    }
                    _ => Err("for..in can only iterate over arrays".to_string()),
                }
            }
            AstNode::BinaryOp { op, left, right } => {
                let left_val = self.interpret_single(left)?;
                let right_val = self.interpret_single(right)?;

                self.eval_binop(op, left_val, right_val)
            }
            AstNode::UnaryOpTT { op, var } => {
                let var_name = match var.as_ref() {
                    AstNode::Identifier(id) => id.clone(),
                    _ => return Err("++/-- can only be applied to variables".to_string()),
                };

                let current_value = self.variables.get(&var_name)
                    .ok_or_else(|| format!("Undefined variable: {}", var_name))?;

                let new_value = match (op.as_str(), current_value) {
                    ("++", RuntimeValue::Number(n)) => RuntimeValue::Number(n + 1),
                    ("--", RuntimeValue::Number(n)) => RuntimeValue::Number(n - 1),
                    _ => return Err(format!("Invalid operation {} for type", op)),
                };
                self.variables.insert(var_name, new_value.clone());

                Ok(new_value)
            },
            AstNode::TypeFunc { expr } => {
                let value = self.interpret_single(expr.as_ref())?;
                match value {
                    RuntimeValue::Number(_n) => Ok(RuntimeValue::String("int".to_string())),
                    RuntimeValue::Float(_f) => Ok(RuntimeValue::String("float".to_string())),
                    RuntimeValue::Boolean(_b) => Ok(RuntimeValue::String("bool".to_string())),
                    RuntimeValue::String(_s) => Ok(RuntimeValue::String("string".to_string())),
                    RuntimeValue::Array(_a) => Ok(RuntimeValue::String("array".to_string())),
                    RuntimeValue::ReturnValue(_v) => Ok(RuntimeValue::String("ReturnValue".to_string())),
                    RuntimeValue::Void => Ok(RuntimeValue::String("void".to_string())),
                    RuntimeValue::ConstValue(_c) => Ok(RuntimeValue::String("const".to_string())),
                }
            },
            &AstNode::Void => todo!(),
        }
    }

    fn print_array(&self, array: &Vec<RuntimeValue>) {
        print!("[");
        for (i, item) in array.iter().enumerate() {
            if i > 0 { print!(", "); }
            match item {
                RuntimeValue::Number(n) => print!("{}", n),
                RuntimeValue::String(s) => print!("\"{}\"", s),
                RuntimeValue::Boolean(b) => print!("{}", b),
                RuntimeValue::Array(a) => self.print_array(a),
                RuntimeValue::Float(f) => print!("{}", f),
                RuntimeValue::ReturnValue(v) => print!("{:?}", v),
                RuntimeValue::ConstValue(v) => print!("{:?}", v),
                RuntimeValue::Void => print!("()"),
            }
        }
        println!("]");
    }

    fn eval_binop(&self, op: &str, left: RuntimeValue, right: RuntimeValue) -> Result<RuntimeValue, String> {
        match (left, right) {
            (RuntimeValue::Number(l), RuntimeValue::Number(r)) => match op {
                "+" => Ok(RuntimeValue::Number(l + r)),
                "-" => Ok(RuntimeValue::Number(l - r)),
                "*" => Ok(RuntimeValue::Number(l * r)),
                "/" => {
                    if r == 0 {
                        Err("Division by zero".to_string())
                    } else {
                        Ok(RuntimeValue::Number(l / r))
                    }
                }
                "==" => Ok(RuntimeValue::Boolean(l == r)),
                "!=" => Ok(RuntimeValue::Boolean(l != r)),
                "<" => Ok(RuntimeValue::Boolean(l < r)),
                ">" => Ok(RuntimeValue::Boolean(l > r)),
                "<=" => Ok(RuntimeValue::Boolean(l <= r)),
                ">=" => Ok(RuntimeValue::Boolean(l >= r)),
                _ => Err(format!("Unknown operator for numbers: {}", op)),
            },
            (RuntimeValue::Boolean(l), RuntimeValue::Boolean(r)) => match op {
                "==" => Ok(RuntimeValue::Boolean(l == r)),
                "!=" => Ok(RuntimeValue::Boolean(l != r)),
                "&&" => Ok(RuntimeValue::Boolean(l && r)),
                "||" => Ok(RuntimeValue::Boolean(l || r)),
                _ => Err(format!("Unknown operator for booleans: {}", op)),
            },
            (RuntimeValue::Float(l), RuntimeValue::Float(r)) => {  // Добавляем обработку float
                match op {
                    "+" => Ok(RuntimeValue::Float(l + r)),
                    "-" => Ok(RuntimeValue::Float(l - r)),
                    "*" => Ok(RuntimeValue::Float(l * r)),
                    "/" => Ok(RuntimeValue::Float(l / r)),
                    "==" => Ok(RuntimeValue::Boolean(l == r)),
                    "!=" => Ok(RuntimeValue::Boolean(l != r)),
                    "<" => Ok(RuntimeValue::Boolean(l < r)),
                    ">" => Ok(RuntimeValue::Boolean(l > r)),
                    "<=" => Ok(RuntimeValue::Boolean(l <= r)),
                    ">=" => Ok(RuntimeValue::Boolean(l >= r)),
                    _ => Err(format!("Unknown operator for floats: {}", op)),
                }
            },
            (RuntimeValue::String(l), RuntimeValue::String(r)) => match op {
                "==" => Ok(RuntimeValue::Boolean(l == r)),
                "!=" => Ok(RuntimeValue::Boolean(l != r)),
                "+" => Ok(RuntimeValue::String(format!("{}{}", l, r))),
                _ => Err(format!("Unknown operator for strings: {}", op)),
            }
            _ => Err("Type mismatch in binary operation".to_string()),
        }
    }
}