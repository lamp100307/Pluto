use super::parser::{AstNode, Parser};
use super::parser::Type;
extern crate rand;

use regex::Regex;
use rand::Rng;
use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::thread::sleep;
use crate::core::lexer::lex;

#[derive(Debug, Clone, PartialEq)]
pub struct MatchResult {
    pub text: String,    // Что было найдено
    pub start: usize,    // Начальная позиция
    pub end: usize,      // Конечная позиция
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum RuntimeValue {
    Number(i64),
    Float(f64),
    String(String),
    Regex(String),
    Boolean(bool),
    Array(Vec<RuntimeValue>),
    Void,
    MatchValue(Box<MatchResult>),
    ReturnValue(Box<RuntimeValue>),
    ConstValue(Box<RuntimeValue>),
    MatchArray(Vec<MatchResult>),
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
            AstNode::Regex(r) => Ok(RuntimeValue::Regex((*r.to_string()).parse().unwrap())),
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

                            if let RuntimeValue::Void = existing_val {}
                            else {
                                match (existing_val, &right_val) {
                                    (RuntimeValue::Number(_), RuntimeValue::Number(_)) => {},
                                    (RuntimeValue::Float(_), RuntimeValue::Float(_)) => {},
                                    (RuntimeValue::Boolean(_), RuntimeValue::Boolean(_)) => {},
                                    (RuntimeValue::String(_), RuntimeValue::String(_)) => {},
                                    (RuntimeValue::Array(_), RuntimeValue::Array(_)) => {},
                                    (RuntimeValue::Regex(_), RuntimeValue::Regex(_)) => {},
                                    (RuntimeValue::MatchValue(_), RuntimeValue::MatchValue(_)) => {},
                                    (RuntimeValue::ReturnValue(_), RuntimeValue::ReturnValue(_)) => {},
                                    (RuntimeValue::MatchArray(_), RuntimeValue::MatchArray(_)) => {},
                                    _ => return Err(format!("Type mismatch for variable {}", id)),
                                }
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
                                    if let Some(RuntimeValue::ConstValue(_)) = self.variables.get(id) {
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
                    RuntimeValue::Regex(v) => println!("{}", v),
                    RuntimeValue::Boolean(v) => println!("{}", v),
                    RuntimeValue::Array(v) => {
                        self.print_array(&v);
                    }
                    RuntimeValue::Void => println!("()"),
                    RuntimeValue::MatchValue(m) => {
                        println!("Match(text: \"{}\", start: {}, end: {})", m.text, m.start, m.end)
                    }
                    RuntimeValue::ReturnValue(v) => {
                        print!("Return(");
                        self.print_value(&v);
                        println!(")");
                    }
                    RuntimeValue::ConstValue(v) => {
                        print!("Const(");
                        self.print_value(&v);
                        println!(")");
                    }
                    RuntimeValue::MatchArray(arr) => {
                        print!("MatchArray[");
                        for (i, m) in arr.iter().enumerate() {
                            if i > 0 { print!(", "); }
                            print!("Match(text: \"{}\", start: {}, end: {})", m.text, m.start, m.end);
                        }
                        println!("]");
                    }
                }
                Ok(RuntimeValue::Void)
            }
            AstNode::Let { name, is_const, var_type, var_value } => {
                let value = self.interpret_single(var_value)?;

                // Функция для проверки совместимости типов
                fn is_type_compatible(existing: &RuntimeValue, new: &RuntimeValue, target_type: Option<&Type>) -> bool {
                    // Если переменная была Void, разрешаем любое присваивание
                    if let RuntimeValue::Void = existing {
                        return true;
                    }

                    // Если указан конкретный тип, проверяем соответствие
                    if let Some(t) = target_type {
                        match (existing, t) {
                            (RuntimeValue::Number(_), Type::Int) => true,
                            (RuntimeValue::Float(_), Type::Float) => true,
                            (RuntimeValue::Boolean(_), Type::Bool) => true,
                            (RuntimeValue::String(_), Type::String) => true,
                            (RuntimeValue::Regex(_), Type::String) => true, // Regex можно присвоить строке
                            (RuntimeValue::Array(_), Type::Array) => true,
                            (RuntimeValue::MatchValue(_), Type::Array) => true, // MatchValue можно присвоить массиву
                            (RuntimeValue::MatchArray(_), Type::Array) => true, // MatchArray можно присвоить массиву
                            _ => false,
                        }
                    } else {
                        // Если тип не указан, проверяем что типы идентичны
                        match (existing, new) {
                            (RuntimeValue::Number(_), RuntimeValue::Number(_)) => true,
                            (RuntimeValue::Float(_), RuntimeValue::Float(_)) => true,
                            (RuntimeValue::Boolean(_), RuntimeValue::Boolean(_)) => true,
                            (RuntimeValue::String(_), RuntimeValue::String(_)) => true,
                            (RuntimeValue::Regex(_), RuntimeValue::Regex(_)) => true,
                            (RuntimeValue::Array(_), RuntimeValue::Array(_)) => true,
                            (RuntimeValue::MatchValue(_), RuntimeValue::MatchValue(_)) => true,
                            (RuntimeValue::MatchArray(_), RuntimeValue::MatchArray(_)) => true,
                            (RuntimeValue::Void, _) => true, // Void можно заменить любым типом
                            _ => false,
                        }
                    }
                }

                // Проверяем, существует ли переменная и можно ли ее переопределить
                if let Some(existing_val) = self.variables.get(name) {
                    if let RuntimeValue::ConstValue(_) = existing_val {
                        return Err(format!("Cannot reassign constant {}", name));
                    }

                    if !is_type_compatible(existing_val, &value, var_type.as_ref()) {
                        return Err(format!("Type mismatch for variable {}", name));
                    }
                }

                // Проверяем соответствие указанному типу (если есть)
                if let Some(t) = var_type {
                    if !is_type_compatible(&RuntimeValue::Void, &value, Some(t)) {
                        return Err(format!("Value does not match declared type for variable {}", name));
                    }
                }

                if *is_const {
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
                let iterable_result = self.interpret_single(iterable)?;
                let iterable_val = self.interpret_constant(iterable_result)?;

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
                    RuntimeValue::Regex(_r) => Ok(RuntimeValue::String("regex".to_string())),
                    RuntimeValue::Array(_a) => Ok(RuntimeValue::String("array".to_string())),
                    RuntimeValue::ReturnValue(_v) => Ok(RuntimeValue::String("ReturnValue".to_string())),
                    RuntimeValue::MatchValue(_m) => Ok(RuntimeValue::String("MatchValue".to_string())),
                    RuntimeValue::Void => Ok(RuntimeValue::String("void".to_string())),
                    RuntimeValue::ConstValue(_c) => Ok(RuntimeValue::String("const".to_string())),
                    RuntimeValue::MatchArray(_a) => Ok(RuntimeValue::String("MatchArray".to_string())),
                }
            },
            AstNode::Sleep { expr } => {
                let value = self.interpret_single(expr.as_ref())?;

                match value {
                    RuntimeValue::Number(milliseconds) => {
                        let duration = std::time::Duration::from_millis(milliseconds as u64);
                        sleep(duration);
                        Ok(RuntimeValue::Void)
                    }
                    RuntimeValue::Float(milliseconds) => {
                        let duration = std::time::Duration::from_millis(milliseconds as u64);
                        sleep(duration);
                        Ok(RuntimeValue::Void)
                    }
                    _ => Err("Sleep function expects a number (milliseconds)".to_string()),
                }
            }
            &AstNode::Void => Ok(RuntimeValue::Void),
            AstNode::Compile { expr, regex } => {
                let value = self.interpret_single(expr.as_ref())?;
                let regex_str = self.interpret_single(regex.as_ref())?;

                let text = match value {
                    RuntimeValue::String(s) => s,
                    _ => return Err("Expected string for compile expression".to_string()),
                };

                let regex_pattern = match regex_str {
                    RuntimeValue::Regex(s) => s,
                    _ => return Err("Expected string for regex pattern".to_string()),
                };

                let regex = match Regex::new(&regex_pattern) {
                    Ok(re) => re,
                    Err(e) => return Err(format!("Invalid regex pattern: {}", e)),
                };

                // Ищем первое совпадение
                if let Some(mat) = regex.find(&text) {
                    // Возвращаем первый MatchResult
                    Ok(RuntimeValue::MatchValue(Box::new(MatchResult {
                        text: mat.as_str().to_string(),
                        start: mat.start(),
                        end: mat.end(),
                    })))
                } else {
                    // Если совпадение не найдено, возвращаем Void или null
                    Ok(RuntimeValue::Void)
                }
            },
            AstNode::CompileAll { expr, regex } => {
                let value = self.interpret_single(expr.as_ref())?;
                let regex_str = self.interpret_single(regex.as_ref())?;

                let text = match value {
                    RuntimeValue::String(s) => s,
                    _ => return Err("Expected string for CompileAll expression".to_string()),
                };

                let regex_pattern = match regex_str {
                    RuntimeValue::Regex(s) => s,
                    _ => return Err("Expected string for regex pattern".to_string()),
                };

                let regex = match Regex::new(&regex_pattern) {
                    Ok(re) => re,
                    Err(e) => return Err(format!("Invalid regex pattern: {}", e)),
                };

                // Создаем массив MatchResult
                let matches: Vec<MatchResult> = regex
                    .find_iter(&text)
                    .map(|mat| MatchResult {
                        text: mat.as_str().to_string(),
                        start: mat.start(),
                        end: mat.end(),
                    })
                    .collect();

                // Возвращаем как MatchArray или обычный Array с MatchValue
                Ok(RuntimeValue::Array(
                    matches.into_iter()
                        .map(|m| RuntimeValue::MatchValue(Box::new(m)))
                        .collect()
                ))
            },
        }
    }

    fn print_array(&self, array: &Vec<RuntimeValue>) {
        print!("[");
        for (i, item) in array.iter().enumerate() {
            if i > 0 { print!(", "); }
            self.print_value(item);
        }
        println!("]");
    }

    fn eval_binop(&self, op: &str, left: RuntimeValue, right: RuntimeValue) -> Result<RuntimeValue, String> {
        match (left, right) {
            // Числовые операции (целые и дробные)
            (RuntimeValue::Number(l), RuntimeValue::Number(r)) => self.num_op(op, l as f64, r as f64),
            (RuntimeValue::Float(l), RuntimeValue::Float(r)) => self.num_op(op, l, r),
            (RuntimeValue::Number(l), RuntimeValue::Float(r)) => self.num_op(op, l as f64, r),
            (RuntimeValue::Float(l), RuntimeValue::Number(r)) => self.num_op(op, l, r as f64),

            // Логические операции
            (RuntimeValue::Boolean(l), RuntimeValue::Boolean(r)) => self.bool_op(op, l, r),
            (RuntimeValue::Number(l), RuntimeValue::Boolean(r)) => self.bool_mixed_op(op, l != 0, r),
            (RuntimeValue::Boolean(l), RuntimeValue::Number(r)) => self.bool_mixed_op(op, l, r != 0),

            // Строковые операции
            (RuntimeValue::String(l), RuntimeValue::Array(r)) => self.str_op(op, l, RuntimeValue::Array(r)),
            (RuntimeValue::String(l), right) => self.str_op(op, l, right),
            (left, RuntimeValue::String(r)) => self.str_op_rev(op, left, r),

            // Операции с массивами
            (RuntimeValue::Array(mut a), RuntimeValue::Array(b)) => self.array_op(op, &mut a, b),

            // Все остальные случаи
            _ => Err("Type mismatch in binary operation".to_string()),
        }
    }

    // Вспомогательные методы
    fn num_op(&self, op: &str, l: f64, r: f64) -> Result<RuntimeValue, String> {
        match op {
            "+" => Ok(RuntimeValue::Float(l + r)),
            "-" => Ok(RuntimeValue::Float(l - r)),
            "*" => Ok(RuntimeValue::Float(l * r)),
            "/" => {
                if r == 0.0 {
                    Err("Division by zero".to_string())
                } else {
                    Ok(RuntimeValue::Float(l / r))
                }
            }
            "==" => Ok(RuntimeValue::Boolean(l == r)),
            "!=" => Ok(RuntimeValue::Boolean(l != r)),
            "<" => Ok(RuntimeValue::Boolean(l < r)),
            ">" => Ok(RuntimeValue::Boolean(l > r)),
            "<=" => Ok(RuntimeValue::Boolean(l <= r)),
            ">=" => Ok(RuntimeValue::Boolean(l >= r)),
            "&&" => Ok(RuntimeValue::Boolean(l != 0.0 && r != 0.0)),
            "||" => Ok(RuntimeValue::Boolean(l != 0.0 || r != 0.0)),
            "%" => Ok(RuntimeValue::Float(l % r)),
            _ => Err(format!("Unknown operator for numbers: {}", op)),
        }
    }

    fn bool_op(&self, op: &str, l: bool, r: bool) -> Result<RuntimeValue, String> {
        match op {
            "==" => Ok(RuntimeValue::Boolean(l == r)),
            "!=" => Ok(RuntimeValue::Boolean(l != r)),
            "&&" => Ok(RuntimeValue::Boolean(l && r)),
            "||" => Ok(RuntimeValue::Boolean(l || r)),
            _ => Err(format!("Unknown operator for booleans: {}", op)),
        }
    }

    fn bool_mixed_op(&self, op: &str, l: bool, r: bool) -> Result<RuntimeValue, String> {
        self.bool_op(op, l, r)
    }

    fn str_op(&self, op: &str, l: String, right: RuntimeValue) -> Result<RuntimeValue, String> {
        match op {
            "+" => Ok(RuntimeValue::String(format!("{}{}", l, self.to_str(right)?))),
            "==" => Ok(RuntimeValue::Boolean(l == self.to_str(right)?)),
            "!=" => Ok(RuntimeValue::Boolean(l != self.to_str(right)?)),
            _ => Err(format!("Unknown operator for strings: {}", op)),
        }
    }

    fn str_op_rev(&self, op: &str, left: RuntimeValue, r: String) -> Result<RuntimeValue, String> {
        match op {
            "+" => Ok(RuntimeValue::String(format!("{}{}", self.to_str(left)?, r))),
            _ => self.str_op(op, r, left),
        }
    }

    fn array_op(&self, op: &str, a: &mut Vec<RuntimeValue>, b: Vec<RuntimeValue>) -> Result<RuntimeValue, String> {
        match op {
            "==" => Ok(RuntimeValue::Boolean(a == &b)),
            "!=" => Ok(RuntimeValue::Boolean(a != &b)),
            "+" => {
                a.extend(b);
                Ok(RuntimeValue::Array(a.clone()))
            }
            "-" => {
                a.retain(|x| !b.contains(x));
                Ok(RuntimeValue::Array(a.clone()))
            }
            _ => Err(format!("Unknown operator for arrays: {}", op)),
        }
    }

    fn to_str(&self, value: RuntimeValue) -> Result<String, String> {
        match value {
            RuntimeValue::Number(n) => Ok(n.to_string()),
            RuntimeValue::Float(f) => Ok(f.to_string()),
            RuntimeValue::Boolean(b) => Ok(b.to_string()),
            RuntimeValue::String(s) => Ok(s),
            _ => Err("Cannot convert to string".to_string()),
        }
    }


    fn print_value(&self, value: &RuntimeValue) {
        match value {
            RuntimeValue::Number(v) => print!("{}", v),
            RuntimeValue::Float(v) => print!("{}", v),
            RuntimeValue::String(v) => print!("\"{}\"", v),
            RuntimeValue::Regex(v) => print!("{}", v),
            RuntimeValue::Boolean(v) => print!("{}", v),
            RuntimeValue::Array(v) => self.print_array(v),
            RuntimeValue::Void => print!("()"),
            RuntimeValue::MatchValue(m) => print!("Match(text: \"{}\", start: {}, end: {})", m.text, m.start, m.end),
            RuntimeValue::ReturnValue(v) => {
                print!("Return(");
                self.print_value(v);
                print!(")");
            }
            RuntimeValue::ConstValue(v) => {
                print!("Const(");
                self.print_value(v);
                print!(")");
            }
            RuntimeValue::MatchArray(arr) => {
                print!("MatchArray[");
                for (i, m) in arr.iter().enumerate() {
                    if i > 0 { print!(", "); }
                    print!("Match(text: \"{}\", start: {}, end: {})", m.text, m.start, m.end);
                }
                print!("]");
            }
        }
    }

    fn interpret_constant(&self, value: RuntimeValue) -> Result<RuntimeValue, String> {
        // Если это ConstValue, извлекаем внутреннее значение
        if let RuntimeValue::ConstValue(boxed_value) = value {
            Ok(*boxed_value)
        } else {
            Ok(value)
        }
    }
}