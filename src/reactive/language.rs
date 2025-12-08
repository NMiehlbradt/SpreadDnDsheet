use std::collections::{HashMap, HashSet};

use super::sheet::{CellId, Sheet};

pub trait IntermediateRep: Sized {
    type Value;
    type Error;

    fn parse(text: &str) -> Result<Self, Self::Error>;

    fn evaluate(
        &self,
        ctx: &Sheet<Self>,
        pushed_values: &Vec<Self::Value>,
        reads: &mut HashSet<CellId>,
        pushes: &mut HashMap<CellId, Vec<Self::Value>>,
    ) -> Result<Self::Value, Self::Error>;

    fn make_error(message: impl Into<String>) -> Self::Error;
}
