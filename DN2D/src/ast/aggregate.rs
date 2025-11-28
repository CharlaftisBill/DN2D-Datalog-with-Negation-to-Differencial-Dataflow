use serde::Serialize;

use crate::ast::Identifier;

#[derive(Debug, Serialize, Clone)]
pub enum AggregateFunction { Count, Sum, Min, Max, Avg }

#[derive(Debug, Serialize, Clone)]
pub struct Aggregate {
    pub func: AggregateFunction,
    pub arg: Identifier,
}