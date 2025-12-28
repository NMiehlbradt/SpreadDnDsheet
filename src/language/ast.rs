use std::collections::BTreeMap;

use crate::language::{bultins::BuiltinFunction, errors::Error};

#[derive(Debug, Clone)]
pub enum Value<T> {
    Unit,
    Integer(i64),
    String(String),
    Boolean(bool),

    Record(BTreeMap<String, T>),
    List(Vec<T>),

    BuiltinFunction(BuiltinFunction),
    Lambda(Vec<String>, Box<AST>),
}

#[derive(Debug, Clone)]
pub struct EvaluatedValue(pub Value<EvaluatedValue>);

impl From<Value<EvaluatedValue>> for EvaluatedValue {
    fn from(value: Value<EvaluatedValue>) -> Self {
        EvaluatedValue(value)
    }
}

impl From<EvaluatedValue> for AST {
    fn from(value: EvaluatedValue) -> Self {
        AST::Literal(value.0.into())
    }
}

impl From<EvaluatedValue> for Value<AST> {
    fn from(value: EvaluatedValue) -> Self {
        value.0.into()
    }
}

impl From<Value<EvaluatedValue>> for Value<AST> {
    fn from(value: Value<EvaluatedValue>) -> Self {
        match value {
            Value::Unit => Value::Unit,
            Value::Integer(i) => Value::Integer(i),
            Value::String(s) => Value::String(s),
            Value::Boolean(b) => Value::Boolean(b),
            Value::Record(fields) => {
                Value::Record(fields.into_iter().map(|(k, v)| (k, v.into())).collect())
            }
            Value::List(items) => Value::List(items.into_iter().map(Into::into).collect()),
            Value::BuiltinFunction(function) => Value::BuiltinFunction(function),
            Value::Lambda(args, body) => Value::Lambda(args, body),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Binding(pub String, pub AST);

#[derive(Debug, Clone)]
pub enum AST {
    Literal(Value<AST>),
    Name(String),
    Function(Box<AST>, Vec<AST>),
    FieldAccess(Box<AST>, String),
    Let(Vec<Binding>, Box<AST>),
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
