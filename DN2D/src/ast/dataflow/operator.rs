use serde::Serialize;

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

#[derive(Debug, Serialize, Clone)]
pub enum MapKind {
    AddOne,
    FilterPositive,
}

#[derive(Debug, Serialize, Clone)]
pub enum Operators {
    Source { 
        name: String 
    },
    Map { 
         input: NodeId, 
        logic: MapKind 
    },
    Join { 
        left: NodeId,  
        right: NodeId 
    },
    Concat { 
        left: NodeId, 
        right: NodeId 
    },   
    Iterate {
        input: NodeId,    
        step_root: NodeId, 
    },
    
    LoopVariable, 
}