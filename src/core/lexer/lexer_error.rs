extern crate thiserror;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum LexerError {
    #[error("Unexpected character: {char}")]
    UnexpectedCharacter { char: char, pos: usize },
}