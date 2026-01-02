
macro_rules! def_builtins {
    ($($str:literal = $id:ident,)*) => {
        #[derive(Debug, Clone, Copy)]
        pub enum BuiltinFunction {
            $($id),*
        }

        pub fn lookup_builtin(name: &str) -> Option<BuiltinFunction> {
            match name {
                $($str => Some(BuiltinFunction::$id),)*
                _ => None
            }
        }

        pub fn stringify_builtin(builtin: BuiltinFunction) -> String {
            match builtin {
                $(BuiltinFunction::$id => $str.to_string(),)*
            }
        }
    };
}

def_builtins!{
    "+" = Add,
    "-" = Sub,
    "*" = Mul,
    "negate" = Negate,

    "push" = Push,
    "read" = Read,

    "index" = Index,

    "<" = LessThan,
    ">" = GreaterThan,
    "<=" = LessThanEqual,
    ">=" = GreaterThanEqual,
    "==" = Equals,

    "and" = And,
    "or" = Or,
    "not" = Not,

    "if" = If,

    "map" = Map,
    "fold" = Fold,
    "filter" = Filter,
}