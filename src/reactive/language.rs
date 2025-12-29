use std::collections::{HashMap, HashSet};
use std::fmt::Debug;

use super::sheet::{CellId, Sheet};

pub trait IntermediateRep: Sized {
    type Value;
    type Error;

    fn parse(text: &str) -> Result<Self, Self::Error>;

    fn evaluate<'a>(
        &self,
        ctx: ReactiveContext<'a, Self>
    ) -> Result<Self::Value, Self::Error>;

    fn make_error(message: impl Into<String>) -> Self::Error;
}

pub struct ReactiveContext<'a, IR: IntermediateRep> {
    pub(super) ctx: &'a Sheet<IR>,
    pub(super) pushed_values: &'a Vec<IR::Value>,
    pub(super) reads: &'a mut HashSet<CellId>,
    pub(super) pushes: &'a mut HashMap<CellId, Vec<IR::Value>>,
}

impl<'a, IR: IntermediateRep> ReactiveContext<'a, IR> 
where 
    IR::Value: Clone + Debug,
{
    pub fn read_cell_by_name(&mut self, name: &str) -> Option<(CellId, &Result<IR::Value, IR::Error>)> {
        let id = CellId(name.to_string());
        self.reads.insert(id.clone());
        self.ctx.get_cell_value(&id).map(|v| (id, v))
    }

    pub fn get_pushes(&self) -> &Vec<IR::Value> {
        self.pushed_values
    }

    pub fn add_push_by_name(&mut self, target: &str, value: &IR::Value) {
        let results = self.pushes.entry(CellId(target.to_string())).or_insert_with(Vec::new);
        results.push(value.clone());
    }
}
