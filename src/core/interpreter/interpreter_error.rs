use thiserror::Error;

use crate::core::parser::ast_nodes::AstNode;

#[derive(Error, Debug)]
pub enum InterpretErrors {
    #[error("Unexpected end of input at position {pos}")]
    UnexpectedEof {
        pos: usize
    },

    #[error("Not implemented {node}")]
    NotImplemented {
        node: AstNode
    },

    #[error("Interpreter error: {message}")]
    InterpretError {
        message: String
    },
}