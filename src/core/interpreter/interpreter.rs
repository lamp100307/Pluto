use crate::core::parser::ast_nodes::{AstNode, Type};
use crate::core::interpreter::interpreter_func::{eval_binop};
use crate::core::interpreter::interpreter_error::InterpretErrors;
use std::collections::HashMap;

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
    ReturnValue(Box<RuntimeValue>),
    ConstValue(Box<RuntimeValue>)
}

pub struct Interpreter {
    variables: HashMap<String, RuntimeValue>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            variables: HashMap::new(),
        }
    }
    pub fn interpret(&mut self, ast: Vec<AstNode>) -> Result<RuntimeValue, InterpretErrors> {
        let mut last_value:RuntimeValue = RuntimeValue::Void;

        for node in ast {
            last_value = self.interpret_single(node)?;
            if let RuntimeValue::ReturnValue(_) = last_value {
                break;
            }
        }

        Ok(last_value)
    }

    pub fn interpret_single(&mut self, node: AstNode) -> Result<RuntimeValue, InterpretErrors> {
        match node {
            AstNode::Number(n) => Ok(RuntimeValue::Number(n)),
            AstNode::BinaryOp { op, left, right } => {
                let left_val = self.interpret_single(*left)?;
                let right_val = self.interpret_single(*right)?;

                eval_binop(&op, left_val, right_val)
                    .map_err(|e| InterpretErrors::InterpretError { message: "cannot eval binary operation".to_string() })
            },
            _ => Err(InterpretErrors::NotImplemented { node: node })
        }
    }
}