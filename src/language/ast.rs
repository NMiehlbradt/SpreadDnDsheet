use std::collections::{BTreeMap, HashMap, HashSet};

use crate::reactive::{
    language::IntermediateRep,
    sheet::{CellId, Sheet},
};

use super::parser::parse;

#[derive(Debug, Clone)]
pub enum Value<T> {
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
}

#[derive(Debug, Clone)]
pub struct Error {
    pub message: String,
}

pub fn pretty_print_result(res: &Result<EvaluatedValue, Error>) -> String {
    use self::s_exprs::ToSExpr;
    match res {
        Ok(v) => v.to_s_expr(),
        Err(e) => format!("Error: {}", e.message),
    }
}

impl Error {
    pub fn with_message<'a>(message: impl Into<String>) -> Self {
        Error {
            message: message.into(),
        }
    }

    fn not_found(cell_id: &CellId) -> Self {
        Error::with_message(format!("Cell {} not found", cell_id.0))
    }

    fn propogated_error(cell_id: &CellId) -> Self {
        Error::with_message(format!("Error in read cell {}", cell_id.0))
    }
}

impl AST {
    pub fn function(name: impl Into<String>, args: Vec<AST>) -> AST {
        AST::Function(Box::new(AST::Literal(Value::BuiltinFunction(name.into()))), args)
    }
}

impl Value<AST> {
    /// Evaluates a value in the context of a sheet.
    ///
    /// This function takes a mutable reference to a set of cells that were read during the evaluation,
    /// and a mutable reference to a map of cells to that were pushed during the evaluation.
    ///
    /// The function returns a Result containing the evaluated value, or an error message if the evaluation failed.
    fn evaluate(
        &self,
        ctx: &Sheet<AST>,
        reads: &mut HashSet<CellId>,
        pushes: &mut HashMap<CellId, Vec<EvaluatedValue>>,
    ) -> Result<EvaluatedValue, Error> {
        match self {
            Value::Integer(i) => Ok(EvaluatedValue(Value::Integer(*i))),
            Value::String(s) => Ok(EvaluatedValue(Value::String(s.clone()))),
            Value::Record(m) => Ok(EvaluatedValue(Value::Record(
                m.iter()
                    .map(|(k, v)| v.evaluate(ctx, reads, pushes).map(|ev| (k.clone(), ev)))
                    .collect::<Result<BTreeMap<String, EvaluatedValue>, _>>()?,
            ))),
            Value::List(l) => Ok(EvaluatedValue(Value::List(
                l.iter()
                    .map(|ast| ast.evaluate(ctx, reads, pushes))
                    .collect::<Result<_, _>>()?,
            ))),
            Value::BuiltinFunction(name) => Ok(EvaluatedValue(Value::BuiltinFunction(name.clone()))),
        }
    }
}

impl IntermediateRep for AST {
    type Value = EvaluatedValue;

    type Error = Error;

    fn parse(text: &str) -> Result<Self, Self::Error> {
        parse(text)
    }

    /// Evaluates an AST in the context of a sheet.
    ///
    /// This function takes a mutable reference to a set of cells that were read during the evaluation,
    /// and a mutable reference to a map of cells to that were pushed during the evaluation.
    ///
    /// The function returns a Result containing the evaluated value, or an error message if the evaluation failed.
    ///
    /// The function is used internally by the sheet to evaluate the contents of cells.
    fn evaluate(
        &self,
        ctx: &Sheet<Self>,
        reads: &mut HashSet<CellId>,
        pushes: &mut HashMap<CellId, Vec<Self::Value>>,
    ) -> Result<Self::Value, Self::Error> {
        match self {
            AST::Literal(value) => Ok(value.evaluate(ctx, reads, pushes)?),

            AST::Name(name) => {
                let cell_id = CellId(name.clone());
                reads.insert(cell_id.clone());
                match ctx.get_cell_value(&cell_id) {
                    Some(value) => value.clone().map_err(|_| Error::propogated_error(&cell_id)),
                    None => Err(Error::not_found(&cell_id)),
                }
            }

            AST::Function(func_name, args) => {
                let function = func_name.evaluate(ctx, reads, pushes)?;
                let func_name = match function {
                    EvaluatedValue(Value::BuiltinFunction(name)) => name,
                    _ => return Err(Error::with_message("Uncallable type")),
                };
                let mut args = args.as_slice();

                macro_rules! eval_function {
                    ($e:expr) => {
                        match args {
                            [] => $e,
                            _ => Err(Error::with_message("Invalid number of arguments")),
                        }
                    };

                    ($e:expr, $x:pat $(,$y:pat)*) => {
                        match args {
                            [_, ..] => match args[0].evaluate(ctx, reads, pushes)? {
                                EvaluatedValue($x) => {
                                    args = &args[1..];
                                    eval_function!($e $(,$y)*)
                                }
                                _ => Err(Error::with_message("Invalid argument type")),
                            }
                            [] => Err(Error::with_message("Invalid number of arguments")),
                        }
                    }
                }

                match func_name.as_str() {
                    "+" => eval_function!(
                        Ok(Value::Integer(a + b).into()),
                        Value::Integer(a),
                        Value::Integer(b)
                    ),
                    "-" => eval_function!(
                        Ok(Value::Integer(a - b).into()),
                        Value::Integer(a),
                        Value::Integer(b)
                    ),
                    "*" => eval_function!(
                        Ok(Value::Integer(a * b).into()),
                        Value::Integer(a),
                        Value::Integer(b)
                    ),
                    "negate" => eval_function!(Ok(Value::Integer(-a).into()), Value::Integer(a)),

                    "dot" => eval_function!(
                        fields.get(&name).cloned().ok_or_else(|| {
                            Error::with_message(format!("Unknown field \"{}\"", name))
                        }),
                        Value::Record(fields),
                        Value::String(name)
                    ),

                    _ => Err(Error::with_message(format!(
                        "Unknown function \"{}\"",
                        func_name
                    ))),
                }
            }
        }
    }

    fn make_error(message: impl Into<String>) -> Self::Error {
        Error::with_message(message)
    }
}

pub mod s_exprs {
    use super::*;

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
                Value::Integer(i) => i.to_string(),
                Value::String(s) => s.clone(),
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
                Value::BuiltinFunction(name) => name.clone(),
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
            }
        }
    }
}
