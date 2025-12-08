use std::collections::{BTreeMap, HashMap, HashSet};

use crate::reactive::{
    language::IntermediateRep,
    sheet::{CellId, Sheet},
};

use super::parser::parse;

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

#[derive(Debug, Clone)]
pub struct Error {
    pub message: String,
}

pub fn pretty_print_result(res: &Result<EvaluatedValue, Error>) -> String {
    use super::s_exprs::ToSExpr;
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
        AST::Function(Box::new(AST::Name(name.into())), args)
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
        pushed_values: &Vec<EvaluatedValue>,
        reads: &mut HashSet<CellId>,
        pushes: &mut HashMap<CellId, Vec<EvaluatedValue>>,
    ) -> Result<EvaluatedValue, Error> {
        match self {
            Value::Unit => Ok(EvaluatedValue(Value::Unit)),
            Value::Integer(i) => Ok(EvaluatedValue(Value::Integer(*i))),
            Value::String(s) => Ok(EvaluatedValue(Value::String(s.clone()))),
            Value::Record(m) => Ok(EvaluatedValue(Value::Record(
                m.iter()
                    .map(|(k, v)| v.evaluate(ctx, pushed_values, reads, pushes).map(|ev| (k.clone(), ev)))
                    .collect::<Result<BTreeMap<String, EvaluatedValue>, _>>()?,
            ))),
            Value::List(l) => Ok(EvaluatedValue(Value::List(
                l.iter()
                    .map(|ast| ast.evaluate(ctx, pushed_values, reads, pushes))
                    .collect::<Result<_, _>>()?,
            ))),
            Value::BuiltinFunction(name) => {
                Ok(EvaluatedValue(Value::BuiltinFunction(name.clone())))
            }
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
        pushed_values: &Vec<EvaluatedValue>,
        reads: &mut HashSet<CellId>,
        pushes: &mut HashMap<CellId, Vec<Self::Value>>,
    ) -> Result<Self::Value, Self::Error> {
        match self {
            AST::Literal(value) => Ok(value.evaluate(ctx, pushed_values, reads, pushes)?),

            AST::Name(name) => {
                let cell_id = CellId(name.clone());
                if let Some(value) = ctx.get_cell_value(&cell_id) {
                    reads.insert(cell_id.clone());
                    value.clone().map_err(|_| Error::propogated_error(&cell_id))
                } else {
                    Ok(Value::BuiltinFunction(name.clone()).into())
                }
            }

            AST::Seq(first, second) => {
                first.evaluate(ctx, pushed_values, reads, pushes)?;
                second.evaluate(ctx, pushed_values, reads, pushes)
            }

            AST::FieldAccess(record, field) => {
                let record = record.evaluate(ctx, pushed_values, reads, pushes)?;
                match record {
                    EvaluatedValue(Value::Record(m)) => {
                        Ok(m.get(field).cloned().ok_or(Error::with_message("Field does not exist"))?.into())
                    }
                    _ => Err(Error::with_message(
                        "Cannot access the field of a non-record type",
                    )),
                }
            }

            AST::Function(func_name, args) => {
                let function = func_name.evaluate(ctx, pushed_values, reads, pushes)?;
                let func_name = match function {
                    EvaluatedValue(Value::BuiltinFunction(name)) => name,
                    _ => return Err(Error::with_message("Uncallable type")),
                };

                let evaluated_args = args
                    .iter()
                    .map(|ast| ast.evaluate(ctx, pushed_values, reads, pushes))
                    .collect::<Result<Vec<Self::Value>, Self::Error>>()?;

                macro_rules! eval_function {
                    ( $([$( $pat:pat ),*] => $body:expr),+ $(,)?) => {
                        match evaluated_args.as_slice() {
                            $([ $( EvaluatedValue($pat) ),* ] => $body,)+
                            _ => Err(Error::with_message("Invalid arguments")),
                        }
                    };
                }

                match func_name.as_str() {
                    "+" => eval_function!(
                        [Value::Integer(a), Value::Integer(b)] => Ok(Value::Integer(a + b).into()),
                    ),
                    "-" => eval_function!(
                        [Value::Integer(a), Value::Integer(b)] => Ok(Value::Integer(a - b).into()),
                    ),
                    "*" => eval_function!(
                        [Value::Integer(a), Value::Integer(b)] => Ok(Value::Integer(a * b).into()),
                    ),
                    "negate" => eval_function!(
                        [Value::Integer(a)] => Ok(Value::Integer(-a).into()),
                    ),
                    "dot" => eval_function!(
                        [Value::Record(fields), Value::String(name)] => fields.get(name).cloned().ok_or_else(|| {
                            Error::with_message(format!("Unknown field \"{}\"", name))
                        }),
                    ),
                    "push" => eval_function!(
                        [Value::String(target), to_push] => {
                            let results = pushes.entry(CellId(target.clone())).or_insert_with(Vec::new);
                            results.push(to_push.clone().into());
                            Ok(Value::Unit.into())
                        },
                    ),
                    "read" => eval_function!([] => {
                        Ok(Value::List(pushed_values.clone()).into())
                    }),
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
