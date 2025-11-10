pub mod parser_error;
pub mod ast_nodes;
pub mod parse_func;
pub mod parser;

// Переэкспортируйте Parser для удобного импорта
pub use parser::Parser;