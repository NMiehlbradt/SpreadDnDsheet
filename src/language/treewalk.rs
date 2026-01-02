use std::collections::{BTreeMap, HashMap};

use crate::{
    language::{
        ast::{AST, Binding, EvaluatedValue, Function, Value},
        bultins::{BuiltinFunction, lookup_builtin},
        errors::Error,
        parser::parse,
    },
    reactive::language::{IntermediateRep, ReactiveContext},
};

struct Scope<'a, T> {
    vars: HashMap<String, T>,
    parent: Option<&'a Scope<'a, T>>,
}

impl<'a, T: Clone> Scope<'a, T> {
    fn new() -> Self {
        Scope {
            vars: HashMap::new(),
            parent: None,
        }
    }

    fn new_with_parent(parent: &'a Self) -> Self {
        Scope {
            vars: HashMap::new(),
            parent: Some(parent),
        }
    }

    fn lookup(&self, name: &String) -> Option<T> {
        if let Some(value) = self.vars.get(name) {
            Some(value.clone())
        } else {
            self.parent.as_ref().and_then(|s| s.lookup(name))
        }
    }

    fn insert(&mut self, name: String, value: T) {
        self.vars.insert(name, value);
    }
}

struct InterpreterCtx<'inner, 'outer> {
    ctx: &'outer mut ReactiveContext<'inner, AST>,
    local_vars: Scope<'outer, EvaluatedValue>,
}

impl<'inner, 'outer> InterpreterCtx<'inner, 'outer> {
    fn new(ctx: &'outer mut ReactiveContext<'inner, AST>) -> Self {
        InterpreterCtx {
            ctx: ctx,
            local_vars: Scope::new(),
        }
    }

    // Creates a new InterpreterCtx from the current one, but with an empty scope
    fn empty_context<'a>(&'a mut self) -> InterpreterCtx<'inner, 'a> {
        InterpreterCtx {
            ctx: self.ctx,
            local_vars: Scope::new(),
        }
    }

    // Creates a new InterpreterCtx from the current one, but creating an inner scope
    fn push_scope<'a>(&'a mut self) -> InterpreterCtx<'inner, 'a> {
        InterpreterCtx {
            ctx: self.ctx,
            local_vars: Scope::new_with_parent(&self.local_vars),
        }
    }

    fn add_local_var(&mut self, name: String, value: EvaluatedValue) {
        self.local_vars.insert(name, value);
    }

    fn evaluate(&mut self, ast: &AST) -> Result<EvaluatedValue, Error> {
        match ast {
            AST::Literal(value) => Ok(self.evaluate_value(value)?),

            AST::Name(name) => {
                // Names starting $ force a cell reference using the rest of the name
                if let Some(c) = name.chars().next()
                    && c == '$'
                {
                    if let Some((_, value)) = self.ctx.read_cell_by_name(&name[1..]) {
                        value.clone().map_err(|_| Error::propogated_error(&name))
                    } else {
                        Err(Error::with_message(format!(
                            "Unknown cell name \"{}\"",
                            name[1..].to_string()
                        )))
                    }
                } else if let Some(value) = self.local_vars.lookup(name) {
                    Ok(value.clone())
                } else if let Some(builtin) = lookup_builtin(name) {
                    Ok(Value::Function(Function::Builtin(builtin)).into())
                } else if let Some((_, value)) = self.ctx.read_cell_by_name(name) {
                    value.clone().map_err(|_| Error::propogated_error(&name))
                } else {
                    Err(Error::with_message(format!("Unknown name \"{}\"", name)))
                }
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

            AST::Let(bindings, expr) => {
                let mut inner_scope = self.push_scope();
                for Binding(name, expr) in bindings {
                    let value = inner_scope.evaluate(expr)?;
                    inner_scope.add_local_var(name.clone(), value);
                }
                inner_scope.evaluate(expr)
            }

            AST::Function(func_name, args) => {
                let function = self.evaluate(func_name)?;
                match function {
                    EvaluatedValue(Value::Function(function)) => {
                        self.evaluate_function(&function, args)
                    }
                    _ => Err(Error::with_message("Uncallable type")),
                }
            }
        }
    }

    fn evaluate_function(
        &mut self,
        function: &Function,
        args: &[AST],
    ) -> Result<EvaluatedValue, Error> {
        match function {
            Function::Lambda(arg_names, body) => {
                let evaluated_args =
                    args.iter()
                        .map(|ast| self.evaluate(ast))
                        .collect::<Result<Vec<EvaluatedValue>, Error>>()?;
                if evaluated_args.len() != arg_names.len() {
                    return Err(Error::with_message("Incorrect number of arguments"));
                }
                let mut ctx = self.empty_context();
                for (name, arg) in arg_names.iter().zip(evaluated_args.iter()) {
                    ctx.add_local_var(name.clone(), arg.clone());
                }
                ctx.evaluate(&body)
            }
            Function::Builtin(builtin) => {
                // Strict evaluation, match on the number and types of arguments
                macro_rules! eval_function {
                            ($([$( $pat:pat ),*] => $body:expr),+ $(,)?) => {{
                                let evaluated_args = args
                                    .iter()
                                    .map(|ast| self.evaluate(ast))
                                    .collect::<Result<Vec<EvaluatedValue>, Error>>()?;

                                match evaluated_args.as_slice() {
                                    $([ $( EvaluatedValue($pat) ),* ] => $body,)+
                                    _ => Err(Error::with_message("Invalid arguments")),
                                }
                            }};
                        }

                // Lazy evaluation, match on the number of arguments but leave them as AST nodes
                macro_rules! lazy_eval {
                            ($([$( $name:ident ),*] => $body:expr),+ $(,)?) => {
                                match args {
                                    $([ $( $name ),* ] => $body,)+
                                    _ => Err(Error::with_message("Incorrect number of arguments")),
                                }
                            };
                        }

                use BuiltinFunction::*;

                match builtin {
                    Add => eval_function!(
                        [Value::Integer(a), Value::Integer(b)] => Ok(Value::Integer(a + b).into()),
                    ),
                    Sub => eval_function!(
                        [Value::Integer(a), Value::Integer(b)] => Ok(Value::Integer(a - b).into()),
                    ),
                    Mul => eval_function!(
                        [Value::Integer(a), Value::Integer(b)] => Ok(Value::Integer(a * b).into()),
                    ),
                    Negate => eval_function!(
                        [Value::Integer(a)] => Ok(Value::Integer(-a).into()),
                    ),
                    Index => eval_function!(
                        [Value::List(l), Value::Integer(i)] => {
                            let len = l.len() as i64;
                            if *i < 0 || *i >= len {
                                Err(Error::with_message("Index out of range"))
                            } else {
                                Ok(l[*i as usize].clone().into())
                            }
                        },
                        [Value::Record(r), Value::String(s)] => {
                            let value = r.get(s).cloned().ok_or(Error::with_message("Field does not exist"))?;
                            Ok(value.into())
                        }
                    ),
                    Read => eval_function!([] => {
                        Ok(Value::List(self.ctx.get_pushes().clone()).into())
                    }),
                    Push => eval_function!(
                        [Value::String(target), to_push] => {
                            let to_push = to_push.clone().into();
                            self.ctx.add_push_by_name(target, &to_push);
                            Ok(Value::Unit.into())
                        },
                    ),
                    LessThan => eval_function!(
                        [Value::Integer(a), Value::Integer(b)] => Ok(Value::Boolean(a < b).into()),
                    ),
                    GreaterThan => eval_function!(
                        [Value::Integer(a), Value::Integer(b)] => Ok(Value::Boolean(a > b).into()),
                    ),
                    LessThanEqual => eval_function!(
                                [Value::Integer(a), Value::Integer(b)] => Ok(Value::Boolean(a <= b).into()),),
                    GreaterThanEqual => eval_function!(
                        [Value::Integer(a), Value::Integer(b)] => Ok(Value::Boolean(a >= b).into()),
                    ),
                    Equals => eval_function!(
                        [Value::Integer(a), Value::Integer(b)] => Ok(Value::Boolean(a == b).into()),
                        [Value::String(a), Value::String(b)] => Ok(Value::Boolean(a == b).into()),
                        [Value::Boolean(a), Value::Boolean(b)] => Ok(Value::Boolean(a == b).into()),
                    ),

                    Not => eval_function!(
                                [Value::Boolean(b)] => Ok(Value::Boolean(!b).into()),),
                    And => lazy_eval!([lhs, rhs] => {
                        if self.evaluate(lhs)?.try_into().map_err(|_| {Error::with_message("Incorrect type")})? {
                            self.evaluate(rhs)
                        } else {
                            Ok(Value::Boolean(false).into())
                        }
                    }),
                    Or => lazy_eval!([lhs, rhs] => {
                        if self.evaluate(lhs)?.try_into().map_err(|_| {Error::with_message("Incorrect type")})? {
                            Ok(Value::Boolean(true).into())
                        } else {
                            self.evaluate(rhs)
                        }
                    }),
                    If => lazy_eval!([cond, then, else_] => {
                        if self.evaluate(cond)?.try_into().map_err(|_| {Error::with_message("Incorrect type")})? {
                            self.evaluate(then)
                        } else {
                            self.evaluate(else_)
                        }
                    }),

                    Map => eval_function!(
                        [Value::Function(f), Value::List(l)] => {
                            let mut new_list = Vec::with_capacity(l.len());
                            for elem in l {
                                new_list.push(self.evaluate_function(f, &[elem.clone().into()])?);
                            }
                            Ok(Value::List(new_list).into())
                        },
                        [Value::Function(f), Value::Record(r)] => {
                            let mut new_record = BTreeMap::new();
                            for (k, v) in r {
                                new_record.insert(k.clone(), self.evaluate_function(f, &[k.clone().into(),v.clone().into()])?);
                            }
                            Ok(Value::Record(new_record).into())
                        }
                    ),
                    Fold => eval_function!(
                        [Value::Function(f), acc_base, Value::List(l)] => {
                            let mut acc = EvaluatedValue(acc_base.clone());
                            for elem in l {
                                acc = self.evaluate_function(f, &[acc.clone().into(), elem.clone().into()])?;
                            }
                            Ok(acc)
                        },
                        [Value::Function(f), acc_base, Value::Record(r)] => {
                            let mut acc = EvaluatedValue(acc_base.clone());
                            for (k, v) in r {
                                acc = self.evaluate_function(f, &[acc.clone().into(), k.clone().into(), v.clone().into()])?;
                            }
                            Ok(acc)
                        }
                    ),
                    Filter => eval_function!(
                        [Value::Function(f), Value::List(l)] => {
                            let mut new_list = Vec::new();
                            for elem in l {
                                if self.evaluate_function(f, &[elem.clone().into()])?.try_into().map_err(|_| {Error::with_message("Incorrect type")})? {
                                    new_list.push(elem.clone());
                                }
                            }
                            Ok(Value::List(new_list).into())
                        },
                        [Value::Function(f), Value::Record(r)] => {
                            let mut new_record = BTreeMap::new();
                            for (k, v) in r {
                                if self.evaluate_function(f, &[k.clone().into(),v.clone().into()])?.try_into().map_err(|_| {Error::with_message("Incorrect type")})? {
                                    new_record.insert(k.clone(), v.clone());
                                }
                            }
                            todo!()
                        }
                    ),
                }
            }
        }
    }

    fn evaluate_value(&mut self, ast: &Value<AST>) -> Result<EvaluatedValue, Error> {
        match ast {
            Value::Unit => Ok(EvaluatedValue(Value::Unit)),
            Value::Integer(i) => Ok(EvaluatedValue(Value::Integer(*i))),
            Value::String(s) => Ok(EvaluatedValue(Value::String(s.clone()))),
            Value::Boolean(b) => Ok(EvaluatedValue(Value::Boolean(*b))),
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
            Value::Function(Function::Builtin(name)) => Ok(EvaluatedValue(Value::Function(
                Function::Builtin(name.clone()),
            ))),
            Value::Function(Function::Lambda(params, body)) => {
                Ok(EvaluatedValue(Value::Function(Function::Lambda(
                    params.clone(),
                    Box::new(self.capture_values(
                        &mut {
                            let mut locals = Scope::new();
                            for param in params.iter() {
                                locals.insert(param.clone(), ());
                            }
                            locals
                        },
                        body,
                    )),
                ))))
            }
        }
    }

    fn capture_values(&self, local_scope: &mut Scope<()>, ast: &AST) -> AST {
        match ast {
            AST::Literal(value) => AST::Literal(match value {
                Value::Record(fields) => Value::Record(
                    fields
                        .iter()
                        .map(|(k, v)| (k.clone(), self.capture_values(local_scope, v)))
                        .collect(),
                ),
                Value::List(items) => Value::List(
                    items
                        .iter()
                        .map(|i| self.capture_values(local_scope, i))
                        .collect(),
                ),
                Value::Function(Function::Lambda(args, ast)) => {
                    Value::Function(Function::Lambda(args.clone(), {
                        let mut inner_scope = Scope::new_with_parent(local_scope);
                        for arg in args {
                            inner_scope.insert(arg.clone(), ());
                        }
                        Box::new(self.capture_values(&mut inner_scope, ast))
                    }))
                }
                value => value.clone(),
            }),
            AST::Name(name) => {
                if let Some(value) = self.local_vars.lookup(name) {
                    value.into()
                } else {
                    ast.clone()
                }
            }
            AST::Function(function, args) => AST::Function(
                Box::new(self.capture_values(local_scope, function)),
                args.iter()
                    .map(|a| self.capture_values(local_scope, a))
                    .collect(),
            ),
            AST::FieldAccess(ast, field) => AST::FieldAccess(
                Box::new(self.capture_values(local_scope, ast)),
                field.clone(),
            ),
            AST::Let(bindings, ast) => {
                let mut inner_scope = Scope::new_with_parent(local_scope);
                let new_bindings = bindings
                    .iter()
                    .map(|Binding(name, expr)| {
                        let new_expr = self.capture_values(&mut inner_scope, expr);
                        inner_scope.insert(name.clone(), ());
                        Binding(name.clone(), new_expr)
                    })
                    .collect();
                AST::Let(
                    new_bindings,
                    Box::new(self.capture_values(&mut inner_scope, ast)),
                )
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
    fn evaluate<'a>(&self, mut ctx: ReactiveContext<'a, Self>) -> Result<Self::Value, Self::Error> {
        InterpreterCtx::new(&mut ctx).evaluate(self)
    }

    fn make_error(message: impl Into<String>) -> Self::Error {
        Error::with_message(message)
    }
}
