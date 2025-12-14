use crate::ast::dataflow::operator::{MapKind, NodeId, Operators};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct Program {
    statements: Vec<Operators>,
}

impl Program {
    pub fn new() -> Self {
        Self { statements: vec![] }
    }

    fn add(&mut self, op: Operators) -> NodeId {
        let id = NodeId(self.statements.len());
        self.statements.push(op);
        id
    }

    pub fn add_input(&mut self, name: String) -> NodeId {
        self.add(Operators::Source { name })
    }

    pub fn add_map(&mut self, input: NodeId, logic: MapKind) -> NodeId {
        self.add(Operators::Map { input, logic })
    }

    pub fn add_join(&mut self, left: NodeId, right: NodeId) -> NodeId {
        self.add(Operators::Join { left, right })
    }

    pub fn add_concat(&mut self, left: NodeId, right: NodeId) -> NodeId {
        self.add(Operators::Concat { left, right })
    }

    pub fn add_iterate(&mut self, input: NodeId, step_root: NodeId) -> NodeId {
        self.add(Operators::Iterate { input, step_root })
    }

    pub fn get(&self, id: NodeId) -> &Operators {
        &self.statements[id.0]
    }
}
