use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("Syntax error at position {pos}: expected {expected}, found {found}")]
    SyntaxError {
        pos: usize,
        expected: String,
        found: String,
    },

    #[error("Unexpected end of input at position {pos}")]
    UnexpectedEof {
        pos: usize
    },

    // Ошибки типов
    #[error("Type error: expected {expected}, got {actual} - {context}")]
    TypeError {
        expected: String,
        actual: String,
        context: String,
    },

    #[error("Not implemented: {function} - {node_info}")]
    NotImplemented {
        function: String,
        node_info: String,
        pos: usize
    },

    #[error("Unexpected token: {token_type} = '{token_value}' at position {pos}")]
    UnexpectedToken {
        token_type: String,
        token_value: String,
        pos: usize,
    },
}