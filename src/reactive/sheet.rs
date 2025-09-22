use crate::language::ast::{s_exprs::ToSExpr, AST};
use crate::maps::fastqueue::FastQueue;
use crate::maps::pairmap::PairMap;
use std::collections::{HashMap, HashSet};

use super::language::IntermediateRep;

pub struct Sheet<IR: IntermediateRep> {
    cells: HashMap<CellId, Cell<IR>>,
    read_relations: PairMap<CellId, CellId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CellId(pub String);

struct Cell<IR: IntermediateRep> {
    raw_contents: String,
    value: Result<IR::Value, IR::Error>,
    parsed: Option<IR>,
}

impl<IR: IntermediateRep> Sheet<IR> {
    /// Creates a new, empty sheet.
    pub fn new() -> Sheet<IR> {
        Sheet {
            cells: HashMap::new(),
            read_relations: PairMap::new(),
        }
    }

    /// Adds a cell to the sheet.
    ///
    /// If a cell with the given name already exists, returns None.
    /// Otherwise, returns the CellId of the newly created cell.
    ///
    /// The contents of the cell are parsed into an intermediate representation
    /// and evaluated in the context of the sheet. The read relations are
    /// also updated.
    pub fn add_cell(&mut self, name: String, contents: impl Into<String>) -> Option<CellId> {
        let id = CellId(name);
        if self.cells.contains_key(&id) {
            None
        } else {
            let mut reads = HashSet::new();
            let mut pushes = HashMap::new();
            let contents = contents.into();
            let (value, ast) = match IR::parse(&contents) {
                Ok(ast) => (ast.evaluate(&self, &mut reads, &mut pushes), Some(ast)),
                Err(err) => (Err(err), None),
            };

            let new_cell = Cell {
                raw_contents: contents,
                value: value,
                parsed: ast,
            };

            self.cells.insert(id.clone(), new_cell);

            for read in reads {
                self.read_relations.insert(read, id.clone());
            }

            Some(id)
        }
    }

    /// Updates the cell with the given name with the given contents.
    ///
    /// The contents are parsed into an intermediate representation and evaluated
    /// in the context of the sheet.
    ///
    /// All cells that depend on the updated cell are re-evaluated.
    pub fn update_cell(&mut self, id: &CellId, contents: impl Into<String>) -> HashSet<CellId> {
        // Update cell
        let cell = self.cells.get_mut(id).unwrap();
        let contents = contents.into();
        match IR::parse(&contents) {
            Ok(ast) => cell.parsed = Some(ast),
            Err(err) => {
                cell.parsed = None;
                cell.value = Err(err);
            }
        }
        cell.raw_contents = contents;

        let mut to_evaluate = FastQueue::new();
        to_evaluate.push(id.clone());
        let mut visited = HashSet::new();

        while let Some(id) = to_evaluate.pop() {
            if visited.insert(id.clone()) || !self.has_cyclic_dependency(&id) {
                self.recompute_cell(&id);
                for dependant in self.read_relations.get_with_left(&id) {
                    to_evaluate.push(dependant.clone());
                }
            } else {
                self.cells.get_mut(&id).unwrap().value = Err(IR::make_error("Circular dependency"));
            }
        }

        visited
    }

    /// Recomputes the cell with the given id and updates the read relations accordingly.
    ///
    /// This function is used by `update_cell` to re-evaluate a cell and all of its dependants.
    fn recompute_cell(&mut self, id: &CellId) {
        self.read_relations.delete_with_right(id);

        if let Some(ast) = &self.cells.get(id).unwrap().parsed {
            let mut new_reads = HashSet::new();
            let mut new_pushes = HashMap::new();
            let new_value = ast.evaluate(&self, &mut new_reads, &mut new_pushes);
            let cell = self.cells.get_mut(id).unwrap();
            cell.value = new_value;

            for read in new_reads {
                self.read_relations.insert(read, id.clone());
            }
        }
    }

    /// Checks if cell id is dependant on itself
    fn has_cyclic_dependency(&self, id: &CellId) -> bool {
        let mut to_evaluate = FastQueue::new();
        let mut visited = HashSet::new();

        to_evaluate.push(id.clone());
        while let Some(next_id) = to_evaluate.pop() {
            if visited.insert(next_id.clone()) {
                for dependant in self.read_relations.get_with_right(&next_id) {
                    to_evaluate.push(dependant.clone());
                }
            } else if *id == next_id {
                return true;
            }
        }

        false
    }

    /// Returns the current value of the cell with the given id.
    ///
    /// This is None if the cell does not exist.
    pub fn get_cell_value(&self, id: &CellId) -> Option<&Result<IR::Value, IR::Error>> {
        self.cells.get(id).map(|c| &c.value)
    }

    pub fn get_cell_text(&self, id: &CellId) -> Option<&str> {
        self.cells.get(id).map(|c| c.raw_contents.as_str())
    }
}

impl Sheet<AST> {
    pub fn get_ast_s_expr(&self, id: &CellId) -> String {
        self.cells
            .get(id)
            .map(|c| (&c.parsed).as_ref())
            .flatten()
            .map(|ast| ast.to_s_expr())
            .unwrap_or("No ast".to_string())
    }
}
