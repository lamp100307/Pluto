#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Float,
    Bool,
    String,
    Array,
    Void,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub param_type: Option<Type>,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum AstNode {
    Number(i64),
    Float(f64),
    String(String),
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
    TypeFunc {
        expr: Box<AstNode>
    },
}

pub struct Parser {
    tokens: Vec<(String, String)>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<(String, String)>) -> Self {
        Parser { tokens, pos: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<AstNode>, String> {
        let mut nodes = Vec::new();

        while self.pos < self.tokens.len() {
            let node = self.parse_expr()?;
            nodes.push(node);
        }

        Ok(nodes)
    }

    fn current_token(&self) -> Option<&(String, String)> {
        self.tokens.get(self.pos)
    }

    fn consume(&mut self, expected_type: &str) -> Result<String, String> {
        let current = self.current_token().cloned();
        if let Some((token_type, token_value)) = current {
            if token_type == expected_type {
                self.pos += 1;
                return Ok(token_value);
            }
        }
        Err(format!("Expected {}, found {:?}", expected_type, self.current_token()))
    }

    fn parse_block(&mut self) -> Result<Vec<AstNode>, String> {
        self.consume("LBRACE")?;
        let mut nodes = Vec::new();

        while let Some((token_type, _)) = self.current_token() {
            if token_type == "RBRACE" {
                break;
            }

            let node = self.parse_expr()?;
            nodes.push(node);
        }

        self.consume("RBRACE")?;
        Ok(nodes)
    }

    fn parse_type(&mut self) -> Result<Type, String> {
        let type_str = self.consume("TYPES")?;
        match type_str.as_str() {
            "int" => Ok(Type::Int),
            "float" => Ok(Type::Float),
            "bool" => Ok(Type::Bool),
            "string" => Ok(Type::String),
            "array" => Ok(Type::Array),
            "void" => Ok(Type::Void),
            _ => Err(format!("Unknown type: {}", type_str)),
        }
    }

    fn parse_param(&mut self) -> Result<Param, String> {
        let name = self.consume("ID")?;
        let mut param_type = None;
        if let Some((token_type, _)) = self.current_token() {
            if token_type == "COLON" {
                self.consume("COLON")?;
                param_type = Some(self.parse_type()?);
            }
        }

        Ok(Param { name, param_type })
    }

    fn parse_optional_else(&mut self) -> Result<Option<Vec<AstNode>>, String> {
        if let Some((t_type, t_value)) = self.current_token() {
            if t_type == "KEYWORD" && t_value == "else" {
                self.consume("KEYWORD")?;
                Ok(Some(self.parse_block()?))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn peek_op(&self) -> Result<String, String> {
        self.current_token()
            .and_then(|(t, v)| if t == "OP" { Some(v.clone()) } else { None })
            .ok_or("Expected operator".to_string())
    }

    fn parse_function(&mut self) -> Result<AstNode, String> {
        let name = self.consume("ID")?;
        self.consume("LPAREN")?;

        let mut params = Vec::new();
        if let Some((token_type, _)) = self.current_token() {
            if token_type != "RPAREN" {
                params.push(self.parse_param()?);
                while let Some((token_type, _)) = self.current_token() {
                    if token_type == "RPAREN" {
                        break;
                    }
                    self.consume("COMMA")?;
                    params.push(self.parse_param()?);
                }
            }
        }

        self.consume("RPAREN")?;

        if let Some((token_type, _)) = self.current_token() {
            if token_type == "ARROW" {
                self.consume("ARROW")?;
                self.parse_type()?
            } else {
                Type::Void
            }
        } else {
            Type::Void
        };

        let body = self.parse_block()?;

        Ok(AstNode::Function {
            name,
            params,
            body,
        })
    }

    // Добавляем обработку return
    fn parse_return(&mut self) -> Result<AstNode, String> {
        let expr = self.parse_expr()?;
        Ok(AstNode::Return(Box::new(expr)))
    }

    fn parse_for(&mut self) -> Result<AstNode, String> {
        self.consume("LPAREN")?;

        if let Some((token_type, _)) = self.current_token() {
            if token_type == "ID" {
                let id = self.consume("ID")?;
                if let Some((next_type, _)) = self.current_token() {
                    if next_type == "KEYWORD" {
                        let keyword = self.consume("KEYWORD")?;
                        if keyword == "in" {
                            let iterable = self.parse_expr()?;
                            self.consume("RPAREN")?;
                            let body = self.parse_block()?;

                            return Ok(AstNode::ForIn {
                                var: id,
                                iterable: Box::new(iterable),
                                body,
                            });
                        }
                    }
                }
                self.pos -= 1;
            }
        }

        let init = self.parse_expr()?;
        self.consume("COMMA")?;
        let condition = self.parse_expr()?;
        self.consume("COMMA")?;
        let increment = self.parse_expr()?;
        self.consume("RPAREN")?;
        let body = self.parse_block()?;

        Ok(AstNode::For {
            init: Box::new(init),
            condition: Box::new(condition),
            increment: Box::new(increment),
            body,
        })
    }

    fn parse_delete(&mut self) -> Result<AstNode, String> {
        self.consume("LPAREN")?;
        let expr = self.parse_expr()?;
        self.consume("RPAREN")?;
        Ok(AstNode::Delete(Box::new(expr)))
    }

    fn parse_array(&mut self) -> Result<AstNode, String> {
        self.consume("LBRACKET")?;
        let mut exprs = Vec::new();
        if let Some((token_type, _)) = self.current_token() {
            if token_type != "RBRACKET" {
                exprs.push(self.parse_expr()?);
                while let Some((token_type, _)) = self.current_token() {
                    if token_type == "RBRACKET" {
                        break;
                    }
                    self.consume("COMMA")?;
                    exprs.push(self.parse_expr()?);
                }
            }
        }
        self.consume("RBRACKET")?;
        Ok(AstNode::Array(exprs))
    }

    fn parse_to(&mut self) -> Result<AstNode, String> {
        self.consume("LPAREN")?;
        let types = self.parse_type()?;
        self.consume("COMMA")?;
        let expr = self.parse_expr()?;
        self.consume("RPAREN")?;

        Ok(AstNode::ToType {
            types,
            expr: Box::new(expr),
        })
    }

    fn parse_type_func(&mut self) -> Result<AstNode, String> {
        self.consume("LPAREN")?;
        let expr = self.parse_expr()?;
        self.consume("RPAREN")?;
        Ok(AstNode::TypeFunc { expr: Box::new(expr) })
    }

    // Parse expressions with + and - (lowest precedence)
    fn parse_expr(&mut self) -> Result<AstNode, String> {
        let mut node = self.parse_term()?;
        loop {
            match self.current_token() {
                Some((token_type, token_value)) if token_type == "OP" && (token_value == "+" || token_value == "-")  => {
                    let op = self.consume("OP")?;
                    let right = self.parse_term()?;
                    node = AstNode::BinaryOp {
                        op,
                        left: Box::new(node),
                        right: Box::new(right),
                    }
                }
                Some((token_type, token_value)) if token_type == "OP" && (token_value == "++" || token_value == "--")  => {
                    let op = self.consume("OP")?;
                    node = AstNode::UnaryOpTT {
                        op,
                        var: Box::new(node)
                    }
                },
                Some((token_type, _)) if token_type == "ASSIGN" => {
                    self.consume("ASSIGN")?;
                    let right = self.parse_expr()?;
                    node = AstNode::Assign {
                        left: Box::new(node),
                        right: Box::new(right),
                    }
                }
                _ => break,
            }
        }
        Ok(node)
    }

    // Parse terms with * and / (higher precedence)
    fn parse_term(&mut self) -> Result<AstNode, String> {
        let mut node = self.parse_comparison()?;
        loop {
            match self.current_token() {
                Some((token_type, token_value)) if token_type == "OP" && ["*", "/"].contains(&token_value.as_str()) => {
                    let op = self.consume("OP")?;
                    let right = self.parse_comparison()?;
                    node = AstNode::BinaryOp {
                        op,
                        left: Box::new(node),
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }
        Ok(node)
    }

    fn parse_comparison(&mut self) -> Result<AstNode, String> {
        let mut node = self.parse_factor()?;
        loop {
            match self.current_token() {
                Some((token_type, token_value)) if token_type == "OP" && ["==", "!=", "<", ">", "<=", ">=", "&&", "||"].contains(&token_value.as_str()) => {
                    let op = self.consume("OP")?;
                    let right = self.parse_factor()?;
                    node = AstNode::BinaryOp {
                        op,
                        left: Box::new(node),
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }
        Ok(node)
    }

    // Parse basic elements (numbers, identifiers, keywords)
    fn parse_factor(&mut self) -> Result<AstNode, String> {
        // Сначала парсим базовый элемент (число, строку, идентификатор и т.д.)
        let mut node = match self.current_token().cloned() {
            Some((token_type, _token_value)) => match token_type.as_str() {
                "KEYWORD" => {
                    let keyword = self.consume("KEYWORD")?;
                    match keyword.as_str() {
                        "printf" => {
                            self.consume("LPAREN")?;
                            let expr = self.parse_expr()?;
                            self.consume("RPAREN")?;
                            Ok(AstNode::Print {
                                left: Box::new(expr),
                            })
                        }
                        "let" => {
                            let name = self.consume("ID")?;
                            let mut is_const = false;
                            let mut var_type = None;

                            if let Some((token_type, _)) = self.current_token() {
                                if token_type == "COLON" {
                                    self.consume("COLON")?;

                                    var_type = Some(self.parse_type()?);

                                    if let Some((next_type, token_value)) = self.current_token() {
                                        if next_type == "OP" && token_value == "&" {
                                            is_const = true;
                                            self.consume("OP")?;
                                        }
                                    }
                                }
                            }

                            self.consume("ASSIGN")?;
                            let var_value = self.parse_expr()?;

                            if var_type.is_none() {
                                match var_value {
                                    AstNode::Array(_) => var_type = Some(Type::Array),
                                    AstNode::String(_) => var_type = Some(Type::String),
                                    AstNode::Number(_) => var_type = Some(Type::Int),
                                    AstNode::Boolean(_) => var_type = Some(Type::Bool),
                                    AstNode::Float(_) => var_type = Some(Type::Float),
                                    _ => Err("Unknown variable type")?
                                }
                            }

                            Ok(AstNode::Let {
                                name,
                                is_const,
                                var_type,
                                var_value: Box::new(var_value),
                            })
                        }
                        "import" => {
                            let import_path = self.consume("STRING")?;
                            Ok(AstNode::Import { path: import_path })
                        }
                        "input" => {
                            self.consume("LPAREN")?;
                            let name = self.consume("STRING")?;
                            self.consume("RPAREN")?;
                            Ok(AstNode::Input{placeholder: name})
                        }
                        "if" => {
                            self.consume("LPAREN")?;
                            let condition = self.parse_expr()?;
                            self.consume("RPAREN")?;

                            let body = self.parse_block()?;

                            // Проверяем наличие else
                            let else_body = if let Some((t_type, t_value)) = self.current_token() {
                                if t_type == "KEYWORD" && t_value == "else" {
                                    self.consume("KEYWORD")?;
                                    Some(self.parse_block()?)
                                } else {
                                    None
                                }
                            } else {
                                None
                            };

                            Ok(AstNode::If {
                                condition: Box::new(condition),
                                body,
                                else_body,
                            })
                        }
                        "while" => {
                            self.consume("LPAREN")?;
                            let condition = self.parse_expr()?;
                            self.consume("RPAREN")?;

                            let body = self.parse_block()?;

                            Ok(AstNode::While {
                                condition: Box::new(condition),
                                body,
                            })
                        }
                        "for" => {
                            self.parse_for()
                        }
                        "random" => {
                            self.consume("LPAREN")?;
                            let left = self.parse_expr()?;
                            self.consume("COMMA")?;
                            let right = self.parse_expr()?;
                            self.consume("RPAREN")?;
                            Ok(AstNode::Random {
                                left: Box::new(left),
                                right: Box::new(right),
                            })
                        }
                        "return" => {
                            Ok(self.parse_return()?)
                        }
                        "del" => {
                            Ok(self.parse_delete()?)
                        }
                        "func" => {
                            Ok(self.parse_function()?)
                        }
                        "to" => {
                            Ok(self.parse_to()?)
                        }
                        "type" => {
                            Ok(self.parse_type_func()?)
                        }
                        _ => Err(format!("Unexpected keyword: {}", keyword)),
                    }
                }
                "BOOL" => {
                    let bool_str = self.consume("BOOL")?;
                    let bool_val = bool_str.parse::<bool>()
                        .map_err(|e| e.to_string())?;
                    Ok(AstNode::Boolean(bool_val))
                }
                "LPAREN" => {
                    self.consume("LPAREN")?;
                    let expr = self.parse_expr()?;
                    self.consume("RPAREN")?;
                    Ok(expr)
                }
                "LBRACKET" => {
                    Ok(self.parse_array()?)
                }
                "STRING" => {
                    let string = self.consume("STRING")?;
                    Ok(AstNode::String(string))
                }
                "NUM" => {
                    let num_str = self.consume("NUM")?;
                    let num = num_str.parse::<i64>()
                        .map_err(|e| e.to_string())?;
                    Ok(AstNode::Number(num))
                }

                "FLOAT" => {
                    let float_str = self.consume("FLOAT")?;
                    let float = float_str.parse::<f64>()
                        .map_err(|e| e.to_string())?;
                    Ok(AstNode::Float(float))
                }

                "ID" => {
                    let id = self.consume("ID")?;

                    // Сначала проверяем вызов функции (скобки)
                    if let Some((token_type, _)) = self.current_token() {
                        if token_type == "LPAREN" {
                            self.consume("LPAREN")?;
                            let mut args = Vec::new();
                            while self.current_token().is_some() && self.current_token().unwrap().0 != "RPAREN" {
                                let arg = self.parse_expr()?;
                                args.push(arg);
                                if self.current_token().is_some() && self.current_token().unwrap().0 == "COMMA" {
                                    self.consume("COMMA")?;
                                }
                            }
                            self.consume("RPAREN")?;

                            // После вызова функции может быть оператор ?
                            if let Some((next_type, _)) = self.current_token() {
                                if next_type == "OP" {
                                    let op = self.peek_op()?;
                                    if op == "?" {
                                        self.consume("OP")?;
                                        let body = self.parse_block()?;
                                        let else_body = self.parse_optional_else()?;

                                        return Ok(AstNode::If {
                                            condition: Box::new(AstNode::FunctionCall {
                                                name: id,
                                                args,
                                            }),
                                            body,
                                            else_body,
                                        });
                                    }
                                }
                            }

                            return Ok(AstNode::FunctionCall {
                                name: id,
                                args,
                            });
                        }
                    }

                    // Затем проверяем оператор ? для обычного идентификатора
                    if let Some((token_type, _)) = self.current_token() {
                        if token_type == "OP" {
                            let op = self.peek_op()?;
                            if op == "?" {
                                self.consume("OP")?;
                                let body = self.parse_block()?;
                                let else_body = self.parse_optional_else()?;

                                return Ok(AstNode::If {
                                    condition: Box::new(AstNode::Identifier(id)),
                                    body,
                                    else_body,
                                });
                            }
                        }
                    }

                    // Если ничего из вышеперечисленного - обычный идентификатор
                    Ok(AstNode::Identifier(id))
                }
                _ => Err(format!("Unexpected token: {:?}", self.current_token())),
            },
            None => Err("Unexpected end of input".to_string()),
        }?;

        loop {
            match self.current_token() {
                Some((token_type, _)) if token_type == "LBRACKET" => {
                    self.consume("LBRACKET")?;
                    let index = self.parse_expr()?;
                    self.consume("RBRACKET")?;
                    node = AstNode::Index {
                        array: Box::new(node),
                        index: Box::new(index),
                    };
                }
                Some((token_type, _)) if token_type == "DOT" => {
                    self.consume("DOT")?;
                    let method = self.consume("ID")?;
                    self.consume("LPAREN")?;

                    let mut args = Vec::new();
                    if self.current_token().unwrap().0 != "RPAREN" {
                        args.push(self.parse_expr()?);
                        while self.current_token().unwrap().0 == "COMMA" {
                            self.consume("COMMA")?;
                            args.push(self.parse_expr()?);
                        }
                    }

                    self.consume("RPAREN")?;
                    node = AstNode::MethodCall {
                        object: Box::new(node),
                        method,
                        args,
                    };
                }
                _ => break,
            }
        }

        Ok(node)
    }
}