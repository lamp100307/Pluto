use std::fmt;

/* Abstract Syntax Tree */
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum Type {
    Int,
    Float,
    Bool,
    String,
    Array,
    Void,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct Param {
    pub name: String,
    pub param_type: Option<Type>,
}

//nodes
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum AstNode {
    Number(i64),
    Float(f64),
    String(String),
    Regex(String),
    Boolean(bool),
    Array(Vec<AstNode>),
    Void,
    Index {
        array: Box<AstNode>,
        index: Box<AstNode>,
    },
    MethodCall {
        object: Box<AstNode>,
        method: String,
        args: Vec<AstNode>,
    },
    Identifier(String),
    If {
        condition: Box<AstNode>,
        body: Vec<AstNode>,
        else_body: Option<Vec<AstNode>>,
    },
    Print {
        left: Box<AstNode>
    },
    Let {
        name: String,
        is_const: bool,
        var_type: Option<Type>,
        var_value: Box<AstNode>,
    },
    Import {
        path: String
    },
    Input {
        placeholder: String
    },
    Random {
        left: Box<AstNode>,
        right: Box<AstNode>
    },
    Delete(Box<AstNode>),
    BinaryOp {
        op: String,
        left: Box<AstNode>,
        right: Box<AstNode>,
    },
    UnaryOpTT {
        op: String,
        var: Box<AstNode>
    },
    Assign {
        left: Box<AstNode>,
        right: Box<AstNode>
    },
    While { condition: Box<AstNode>, body: Vec<AstNode> },
    For {
        init: Box<AstNode>,
        condition: Box<AstNode>,
        increment: Box<AstNode>,
        body: Vec<AstNode>
    },
    ForIn {
        var: String,
        iterable: Box<AstNode>,
        body: Vec<AstNode>
    },
    Return(Box<AstNode>),
    Function {
        name: String,
        params: Vec<Param>,
        body: Vec<AstNode>,
    },
    FunctionCall { name: String, args: Vec<AstNode> },
    ToType {
        types: Type,
        expr: Box<AstNode>
    },
    Sleep {
        expr: Box<AstNode>
    },
    TypeFunc {
        expr: Box<AstNode>
    },
    CompileAll {
        expr: Box<AstNode>,
        regex: Box<AstNode>
    },
    Compile {
        expr: Box<AstNode>,
        regex: Box<AstNode>
    },
}


impl fmt::Display for AstNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AstNode::Number(n) => write!(f, "Number: {}", n),
            AstNode::String(s) => write!(f, "String: {}", s),
            AstNode::BinaryOp { op, left, right } => 
            write!(f, "binop: {}, left: {}, right: {}", op, left, right),

            _ => write!(f, "Node")
        }
    }
}