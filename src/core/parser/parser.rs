use crate::core::parser::{ast_nodes::AstNode, parser_error::ParserError};

// Сделайте структуру публичной
pub struct Parser {
    tokens: Vec<(String, String)>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<(String, String)>) -> Self {
        Parser { tokens, pos: 0 }
    }
    
    pub fn parse(&mut self) -> Result<Vec<AstNode>, Vec<ParserError>> {
        let mut nodes = Vec::new();
        
        while self.pos < self.tokens.len() {
            let node = self.parse_expr().unwrap();
            nodes.push(node);
        }
        
        Ok(nodes)
    }

    fn peek(&self) -> Option<&(String, String)> {
        self.tokens.get(self.pos)
    }

    fn current_token(&self) -> Option<&(String, String)> {
        self.tokens.get(self.pos)
    }

    fn consume(&mut self, expected_type: &str) -> Result<String, ParserError> {
        if let Some((token_type, token_value)) = self.peek() {
            if token_type == expected_type {
                let value = token_value.clone();
                self.pos += 1; 
                return Ok(value);
            } else {
                return Err(ParserError::SyntaxError { 
                    pos: self.pos, 
                    expected: expected_type.to_string(), 
                    found: token_type.clone() 
                });
            }
        }
        Err(ParserError::UnexpectedEof { pos: self.pos })
    }

    fn parse_expr(&mut self) -> Result<AstNode, String> {
        let mut node = self.parse_term()?;
        
        while let Some((token_type, token_value)) = self.current_token() {
            if token_type == "OP" && (token_value == "+" || token_value == "-") {
                let op = self.consume("OP").unwrap();
                let right = self.parse_term()?;
                node = AstNode::BinaryOp {
                    op,
                    left: Box::new(node),
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        
        Ok(node)
    }

    fn parse_term(&mut self) -> Result<AstNode, String> {
        let mut node = self.parse_factor()?;
        
        while let Some((token_type, token_value)) = self.current_token() {
            if token_type == "OP" && (token_value == "*" || token_value == "/") {
                let op = self.consume("OP").unwrap();
                let right = self.parse_factor()?;
                node = AstNode::BinaryOp {
                    op,
                    left: Box::new(node),
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        
        Ok(node)
    }

    fn parse_factor(&mut self) -> Result<AstNode, String> {
        let token = self.current_token()
            .ok_or("Unexpected end of input".to_string())?;
            
        // Исправляем проблему с типами - сравниваем с String
        match token {
            (token_type, _) if token_type == "NUMBER" => {
                let num_str = self.consume("NUMBER").unwrap();
                let num = num_str.parse::<i64>()
                    .map_err(|e| format!("Failed to parse number: {}", e))?;
                Ok(AstNode::Number(num))
            }
            (token_type, _) if token_type == "STRING" => {
                let s = self.consume("STRING").unwrap();
                Ok(AstNode::String(s))
            }
            (token_type, token_value) => {
                Err(format!("Unexpected token type: {} value: {}", token_type, token_value))
            }
        }
    }
}