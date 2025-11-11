use crate::core::interpreter::interpreter::RuntimeValue;

pub fn eval_binop(op: &str, left: RuntimeValue, right: RuntimeValue) -> Result<RuntimeValue, String> {
    match op {
        // Арифметические операции
        "+" => {
            match (&left, &right) {
                (RuntimeValue::Number(l), RuntimeValue::Number(r)) => Ok(RuntimeValue::Number(l + r)),
                (RuntimeValue::String(l), RuntimeValue::String(r)) => Ok(RuntimeValue::String(l.clone() + r)),
                (RuntimeValue::String(l), RuntimeValue::Number(r)) => Ok(RuntimeValue::String(format!("{}{}", l, r))),
                (RuntimeValue::Number(l), RuntimeValue::String(r)) => Ok(RuntimeValue::String(format!("{}{}", l, r))),
                _ => Err(format!("Cannot add {:?} and {:?}", left, right)),
            }
        }
        "-" => {
            match (&left, &right) {
                (RuntimeValue::Number(l), RuntimeValue::Number(r)) => Ok(RuntimeValue::Number(l - r)),
                _ => Err(format!("Cannot subtract {:?} from {:?}", right, left)),
            }
        }
        "*" => {
            match (&left, &right) {
                (RuntimeValue::Number(l), RuntimeValue::Number(r)) => Ok(RuntimeValue::Number(l * r)),
                (RuntimeValue::String(l), RuntimeValue::Number(r)) => {
                    if r >= &0 {
                        Ok(RuntimeValue::String(l.repeat(*r as usize)))
                    } else {
                        Err("Cannot multiply string by negative number".to_string())
                    }
                }
                (RuntimeValue::Number(l), RuntimeValue::String(r)) => {
                    if l >= &0 {
                        Ok(RuntimeValue::String(r.repeat(*l as usize)))
                    } else {
                        Err("Cannot multiply string by negative number".to_string())
                    }
                }
                _ => Err(format!("Cannot multiply {:?} and {:?}", left, right)),
            }
        }
        "/" => {
            match (&left, &right) {
                (RuntimeValue::Number(l), RuntimeValue::Number(r)) => {
                    if r == &0 {
                        Err("Division by zero".to_string())
                    } else {
                        Ok(RuntimeValue::Number(l / r))
                    }
                }
                _ => Err(format!("Cannot divide {:?} by {:?}", left, right)),
            }
        }
        "%" => {
            match (&left, &right) {
                (RuntimeValue::Number(l), RuntimeValue::Number(r)) => {
                    if r == &0 {
                        Err("Modulo by zero".to_string())
                    } else {
                        Ok(RuntimeValue::Number(l % r))
                    }
                }
                _ => Err(format!("Cannot modulo {:?} by {:?}", left, right)),
            }
        }
        
        // Операции сравнения
        "==" => {
            match (&left, &right) {
                (RuntimeValue::Number(l), RuntimeValue::Number(r)) => Ok(RuntimeValue::Boolean(l == r)),
                (RuntimeValue::String(l), RuntimeValue::String(r)) => Ok(RuntimeValue::Boolean(l == r)),
                (RuntimeValue::Boolean(l), RuntimeValue::Boolean(r)) => Ok(RuntimeValue::Boolean(l == r)),
                _ => Ok(RuntimeValue::Boolean(false)),
            }
        }
        "!=" => {
            match (&left, &right) {
                (RuntimeValue::Number(l), RuntimeValue::Number(r)) => Ok(RuntimeValue::Boolean(l != r)),
                (RuntimeValue::String(l), RuntimeValue::String(r)) => Ok(RuntimeValue::Boolean(l != r)),
                (RuntimeValue::Boolean(l), RuntimeValue::Boolean(r)) => Ok(RuntimeValue::Boolean(l != r)),
                _ => Ok(RuntimeValue::Boolean(true)),
            }
        }
        ">" => {
            match (&left, &right) {
                (RuntimeValue::Number(l), RuntimeValue::Number(r)) => Ok(RuntimeValue::Boolean(l > r)),
                (RuntimeValue::String(l), RuntimeValue::String(r)) => Ok(RuntimeValue::Boolean(l > r)),
                _ => Err(format!("Cannot compare {:?} > {:?}", left, right)),
            }
        }
        "<" => {
            match (&left, &right) {
                (RuntimeValue::Number(l), RuntimeValue::Number(r)) => Ok(RuntimeValue::Boolean(l < r)),
                (RuntimeValue::String(l), RuntimeValue::String(r)) => Ok(RuntimeValue::Boolean(l < r)),
                _ => Err(format!("Cannot compare {:?} < {:?}", left, right)),
            }
        }
        ">=" => {
            match (&left, &right) {
                (RuntimeValue::Number(l), RuntimeValue::Number(r)) => Ok(RuntimeValue::Boolean(l >= r)),
                (RuntimeValue::String(l), RuntimeValue::String(r)) => Ok(RuntimeValue::Boolean(l >= r)),
                _ => Err(format!("Cannot compare {:?} >= {:?}", left, right)),
            }
        }
        "<=" => {
            match (&left, &right) {
                (RuntimeValue::Number(l), RuntimeValue::Number(r)) => Ok(RuntimeValue::Boolean(l <= r)),
                (RuntimeValue::String(l), RuntimeValue::String(r)) => Ok(RuntimeValue::Boolean(l <= r)),
                _ => Err(format!("Cannot compare {:?} <= {:?}", left, right)),
            }
        }
        
        // Логические операции
        "&&" => {
            match (&left, &right) {
                (RuntimeValue::Boolean(l), RuntimeValue::Boolean(r)) => Ok(RuntimeValue::Boolean(*l && *r)),
                _ => Err(format!("Cannot perform AND on {:?} and {:?}", left, right)),
            }
        }
        "||" => {
            match (&left, &right) {
                (RuntimeValue::Boolean(l), RuntimeValue::Boolean(r)) => Ok(RuntimeValue::Boolean(*l || *r)),
                _ => Err(format!("Cannot perform OR on {:?} and {:?}", left, right)),
            }
        }
        
        // Неподдерживаемая операция
        _ => Err(format!("Unknown operator: {}", op)),
    }
}