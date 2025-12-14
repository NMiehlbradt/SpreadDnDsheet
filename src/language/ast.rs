use std::collections::BTreeMap;

use crate::language::errors::Error;

#[derive(Debug, Clone)]
pub enum Value<T> {
    Unit,
    Integer(i64),
    String(String),

    Record(BTreeMap<String, T>),
    List(Vec<T>),

    BuiltinFunction(String),
}

#[derive(Debug, Clone)]
pub struct EvaluatedValue(pub Value<EvaluatedValue>);

impl From<Value<EvaluatedValue>> for EvaluatedValue {
    fn from(value: Value<EvaluatedValue>) -> Self {
        EvaluatedValue(value)
    }
}

#[derive(Debug, Clone)]
pub enum AST {
    Literal(Value<AST>),
    Name(String),
    Function(Box<AST>, Vec<AST>),
    Seq(Box<AST>, Box<AST>),
    FieldAccess(Box<AST>, String),
}

pub fn pretty_print_result(res: &Result<EvaluatedValue, Error>) -> String {
    use super::s_exprs::ToSExpr;
    match res {
        Ok(v) => v.to_s_expr(),
        Err(e) => format!("Error: {}", e.message),
    }
}

impl AST {
    pub fn function(name: impl Into<String>, args: Vec<AST>) -> AST {
        AST::Function(Box::new(AST::Name(name.into())), args)
    }
}
