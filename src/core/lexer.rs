extern crate regex;

use regex::Regex;

pub fn lex(code: &str) -> Vec<(String, String)> {
    let keywords = vec!["printf", "let", "import", "if", "else",
                        "while", "for", "in", "random", "func", "return",
                        "del", "in", "input", "to", "type", "sleep", "compile_all", "compile"];

    //tokens specification
    let tokens_spec: Vec<(String, Regex)> = vec![
        ("WHITESPACE".to_string(), Regex::new(r"\s+").unwrap()),
        ("NEWLINE".to_string(), Regex::new(r"\n+").unwrap()),
        ("COMMENT".to_string(), Regex::new(r"//.*").unwrap()),
        ("ARROW".to_string(), Regex::new(r"->").unwrap()),
        ("OP".to_string(), Regex::new(r"\?|\+\+|--|\+|-|\*|/|!=|==|<=|>=|<|>|&&|&|\|\|").unwrap()),
        ("ASSIGN".to_string(), Regex::new(r"=").unwrap()),
        ("LPAREN".to_string(), Regex::new(r"\(").unwrap()),
        ("RPAREN".to_string(), Regex::new(r"\)").unwrap()),
        ("LBRACE".to_string(), Regex::new(r"\{").unwrap()),
        ("RBRACE".to_string(), Regex::new(r"}").unwrap()),
        ("LBRACKET".to_string(), Regex::new(r"\[").unwrap()),
        ("RBRACKET".to_string(), Regex::new(r"]").unwrap()),
        ("COLON".to_string(), Regex::new(r":").unwrap()),
        ("COMMA".to_string(), Regex::new(r",").unwrap()),
        ("DOT".to_string(), Regex::new(r"\.").unwrap()),
        ("TYPES".to_string(), Regex::new(r"int|float|bool|string|array|void").unwrap()),
        ("FLOAT".to_string(), Regex::new(r"\d+\.\d+").unwrap()),
        ("NUM".to_string(), Regex::new(r"\d+").unwrap()),
        ("STRING".to_string(), Regex::new(r#""([^"\\]|\\.|\\\$[^{]|\$\{[^}]*\})*""#).unwrap()),
        ("REGEX".to_string(), Regex::new(r"\\[^/]+\\").unwrap()), // Исправлено для regex
        ("BOOL".to_string(), Regex::new(r"true|false").unwrap()),
        ("ID".to_string(), Regex::new(r"[a-zA-Z_][a-zA-Z0-9_]*").unwrap()),
    ];

    let mut tokens = Vec::new();
    let mut remaining = code;

    //matching tokens
    while !remaining.is_empty() {
        let mut matched = false;

        for (name, regex) in &tokens_spec {
            if let Some(mat) = regex.find(remaining) {
                if mat.start() == 0 {
                    //skips
                    if name != "WHITESPACE" && name != "COMMENT" && name != "NEWLINE" {
                        let token_value = mat.as_str().to_string();
                        let token_type = if name == "ID" && keywords.contains(&token_value.as_str()) {
                            "KEYWORD".to_string()
                        } else {
                            name.clone()
                        };

                        // Для строк и regex убираем только внешние кавычки/слэши
                        let processed_value = if name == "REGEX" {
                            token_value[1..token_value.len()-1].to_string()
                        } else {
                            token_value
                        };

                        tokens.push((token_type, processed_value));
                    }
                    remaining = &remaining[mat.end()..];
                    matched = true;
                    break;
                }
            }
        }

        if !matched {
            panic!("Unexpected character at: {}", remaining);
        }
    }

    tokens
}