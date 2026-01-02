use crate::language::bultins::stringify_builtin;

use super::ast::*;

pub trait ToSExpr {
    fn to_s_expr(&self) -> String;
}

impl ToSExpr for EvaluatedValue {
    fn to_s_expr(&self) -> String {
        self.0.to_s_expr()
    }
}

// Conversions to s expressions for testing
impl<T: ToSExpr> ToSExpr for Value<T> {
    fn to_s_expr(&self) -> String {
        match self {
            Value::Unit => "()".to_string(),
            Value::Integer(i) => i.to_string(),
            Value::String(s) => format!("\"{s}\""),
            Value::Boolean(b) => b.to_string(),
            Value::Record(fields) => format!(
                "{{{}}}",
                fields
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v.to_s_expr()))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Value::List(values) => format!(
                "[{}]",
                values
                    .iter()
                    .map(|v| v.to_s_expr())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Value::Function(Function::Builtin(function)) => format!("(builtin {})", stringify_builtin(*function)),
            Value::Function(Function::Lambda(params, body)) => {
                format!("(lambda ({}) {})", params.join(", "), body.to_s_expr())
            }
        }
    }
}

impl ToSExpr for AST {
    fn to_s_expr(&self) -> String {
        match self {
            AST::Literal(value) => value.to_s_expr(),
            AST::Name(name) => name.clone(),
            AST::Function(name, args) => format!(
                "({} {})",
                name.to_s_expr(),
                args.iter()
                    .map(|a| a.to_s_expr())
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
            AST::FieldAccess(record, field) => format!("(.{field} {})", record.to_s_expr()),
            AST::Let(bindings, expr) => {
                let binding_s_exprs = bindings
                    .iter()
                    .map(|Binding(name, value)| format!("({} {})", name, value.to_s_expr()))
                    .collect::<Vec<_>>()
                    .join(" ");
                format!("(let ({}) {})", binding_s_exprs, expr.to_s_expr())
            }
        }
    }
}
