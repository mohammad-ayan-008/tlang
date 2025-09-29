use std::{
    cell::RefCell,
    fmt::{Debug, format, write},
    rc::Rc,
    result, usize,
};

use crate::{
    token::{Literal, Token},
    tokentype::TokenType,
};
#[derive(Clone)]
pub enum LiteralValue {
    Number(f64),
    StringValue(String),
    True,
    False,
    Nil,
}
impl Debug for LiteralValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl PartialEq for LiteralValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Number(a), Self::Number(b)) => a == b,
            (Self::StringValue(a), Self::StringValue(b)) => a == b,
            (Self::True, Self::True) => true,
            (Self::False, Self::False) => true,
            _ => false,
        }
    }
}
impl LiteralValue {
    pub fn is_truthy(&self) -> bool {
        match self {
            LiteralValue::False | LiteralValue::Nil => false,
            _ => true,
        }
    }

    pub fn is_falsy(&self) -> LiteralValue {
        match self {
            Self::Number(x) => {
                if *x == 0.0 {
                    Self::True
                } else {
                    Self::False
                }
            }
            Self::StringValue(s) => {
                if s.len() == 0 {
                    Self::True
                } else {
                    Self::False
                }
            }
            Self::True => Self::False,
            Self::False => Self::True,
            Self::Nil => Self::True,
        }
    }
    pub fn from_bool(b: bool) -> Self {
        if b { Self::True } else { Self::False }
    }

    pub fn to_type(&self) -> String {
        match self {
            LiteralValue::Number(_) => "Number".to_string(),
            LiteralValue::StringValue(_) => "String".to_string(),
            LiteralValue::Nil => "nil".to_string(),
            LiteralValue::True => "true".to_string(),
            LiteralValue::False => "false".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    Assign {
        name: Token,
        value: Box<Expr>,
    },
    Call {
        callie: Box<Expr>,
        paren: Token,
        args: Vec<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Literal {
        value: LiteralValue,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Variable {
        name: Token,
    },
    Logical {
        expression: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
}

#[allow(warnings)]
impl ToString for LiteralValue {
    fn to_string(&self) -> String {
        match self {
            LiteralValue::Number(x) => x.to_string(),
            LiteralValue::StringValue(x) => x.clone(),
            LiteralValue::True => "true".to_string(),
            LiteralValue::False => "false".to_string(),
            LiteralValue::Nil => "nil".to_string(),
        }
    }
}
fn unwrap_as_string(literal: Option<Literal>) -> String {
    match literal {
        Some(Literal::StringLiteral(x)) => x.clone(),
        Some(Literal::IdentifierLiteral(s)) => s.clone(),
        _ => panic!("could not unwrap"),
    }
}
fn unwrap_as_f64(literal: Option<Literal>) -> f64 {
    match literal {
        Some(Literal::FLiteral(x)) => x as f64,
        _ => panic!("could not unwrap"),
    }
}

impl LiteralValue {
    pub fn from_token(token: Token) -> Self {
        match token.token_type {
            crate::tokentype::TokenType::NUMBER => Self::Number(unwrap_as_f64(token.literal)),
            crate::tokentype::TokenType::STRINGLIT => {
                Self::StringValue(unwrap_as_string(token.literal))
            }
            crate::tokentype::TokenType::FALSE => Self::False,
            crate::tokentype::TokenType::TRUE => Self::True,
            crate::tokentype::TokenType::NIL => Self::Nil,
            _ => panic!("could not create literal value from {:?}", token),
        }
    }
}
#[allow(warnings)]
impl ToString for Expr {
    fn to_string(&self) -> String {
        match self {
            Expr::Call {
                callie,
                paren,
                args,
            } => {
                format!("{:?}", callie)
            }
            Expr::Logical {
                expression,
                operator,
                right,
            } => "".to_string(),
            Expr::Assign { name, value } => {
                format!("{name:?} = {}", value.to_string())
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                format!(
                    "({} {} {})",
                    operator.lexeme,
                    left.to_string(),
                    right.to_string()
                )
            }
            Expr::Unary { operator, right } => {
                let operator_str = operator.lexeme.clone();
                let right_str = (*right).to_string();
                format!("({} {})", operator_str, right_str)
            }
            Expr::Literal { value } => {
                format!("{}", value.to_string())
            }
            Expr::Grouping { expression } => {
                format!("(group {})", (*expression).to_string())
            }
            Expr::Variable { name } => format!("(var {})", name.lexeme),
        }
    }
}
