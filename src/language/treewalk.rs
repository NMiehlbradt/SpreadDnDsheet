use std::collections::{BTreeMap, HashMap, HashSet};

use crate::{
    language::{
        ast::{AST, EvaluatedValue, Value},
        errors::Error,
        parser::parse,
    },
    reactive::{
        language::IntermediateRep,
        sheet::{CellId, Sheet},
    },
};

struct InterpreterCtx<'a> {
    ctx: &'a Sheet<AST>,
    pushed_values: &'a Vec<EvaluatedValue>,
    reads: &'a mut HashSet<CellId>,
    pushes: &'a mut HashMap<CellId, Vec<EvaluatedValue>>,
}

impl InterpreterCtx<'_> {
    fn new<'a>(
        ctx: &'a Sheet<AST>,
        pushed_values: &'a Vec<EvaluatedValue>,
        reads: &'a mut HashSet<CellId>,
        pushes: &'a mut HashMap<CellId, Vec<EvaluatedValue>>,
    ) -> InterpreterCtx<'a> {
        InterpreterCtx {
            ctx,
            pushed_values,
            reads,
            pushes,
        }
    }

    fn evaluate(&mut self, ast: &AST) -> Result<EvaluatedValue, Error> {
        match ast {
            AST::Literal(value) => Ok(self.evaluate_value(value)?),

            AST::Name(name) => {
                let cell_id = CellId(name.clone());
                if let Some(value) = self.ctx.get_cell_value(&cell_id) {
                    self.reads.insert(cell_id.clone());
                    value.clone().map_err(|_| Error::propogated_error(&cell_id))
                } else {
                    Ok(Value::BuiltinFunction(name.clone()).into())
                }
            }

            AST::Seq(first, second) => {
                self.evaluate(first)?;
                self.evaluate(second)
            }

            AST::FieldAccess(record, field) => {
                let record = self.evaluate(record)?;
                match record {
                    EvaluatedValue(Value::Record(m)) => Ok(m
                        .get(field)
                        .cloned()
                        .ok_or(Error::with_message("Field does not exist"))?
                        .into()),
                    _ => Err(Error::with_message(
                        "Cannot access the field of a non-record type",
                    )),
                }
            }

            AST::Function(func_name, args) => {
                let function = self.evaluate(func_name)?;
                let func_name = match function {
                    EvaluatedValue(Value::BuiltinFunction(name)) => name,
                    _ => return Err(Error::with_message("Uncallable type")),
                };

                let evaluated_args =
                    args.iter()
                        .map(|ast| self.evaluate(ast))
                        .collect::<Result<Vec<EvaluatedValue>, Error>>()?;

                macro_rules! eval_function {
                    ( $([$( $pat:pat ),*] => $body:expr),+ $(,)?) => {
                        match evaluated_args.as_slice() {
                            $([ $( EvaluatedValue($pat) ),* ] => $body,)+
                            _ => Err(Error::with_message("Invalid arguments")),
                        }
                    };
                }

                match func_name.as_str() {
                    // Math Operations
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
                    // Push value operations
                    "push" => eval_function!(
                        [Value::String(target), to_push] => {
                            let results = self.pushes.entry(CellId(target.clone())).or_insert_with(Vec::new);
                            results.push(to_push.clone().into());
                            Ok(Value::Unit.into())
                        },
                    ),
                    "read" => eval_function!([] => {
                        Ok(Value::List(self.pushed_values.clone()).into())
                    }),
                    // Misc
                    "index" => eval_function!(
                        [Value::List(l), Value::Integer(i)] => {
                            let len = l.len() as i64;
                            if *i < 0 || *i >= len {
                                Err(Error::with_message("Index out of range"))
                            } else {
                                Ok(l[*i as usize].clone().into())
                            }
                        }
                    ),
                    // Error case
                    _ => Err(Error::with_message(format!(
                        "Unknown function \"{}\"",
                        func_name
                    ))),
                }
            }
        }
    }

    fn evaluate_value(&mut self, ast: &Value<AST>) -> Result<EvaluatedValue, Error> {
        match ast {
            Value::Unit => Ok(EvaluatedValue(Value::Unit)),
            Value::Integer(i) => Ok(EvaluatedValue(Value::Integer(*i))),
            Value::String(s) => Ok(EvaluatedValue(Value::String(s.clone()))),
            Value::Record(m) => Ok(EvaluatedValue(Value::Record(
                m.iter()
                    .map(|(k, v)| self.evaluate(v).map(|ev| (k.clone(), ev)))
                    .collect::<Result<BTreeMap<String, EvaluatedValue>, _>>()?,
            ))),
            Value::List(l) => Ok(EvaluatedValue(Value::List(
                l.iter()
                    .map(|ast| self.evaluate(ast))
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
        InterpreterCtx::new(ctx, pushed_values, reads, pushes).evaluate(self)
    }

    fn make_error(message: impl Into<String>) -> Self::Error {
        Error::with_message(message)
    }
}
