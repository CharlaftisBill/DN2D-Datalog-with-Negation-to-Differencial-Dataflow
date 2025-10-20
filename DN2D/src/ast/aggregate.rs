use serde::Serialize;

use crate::ast::Identifier;

#[derive(Debug, Serialize)]
pub enum AggregateFunction { Count, Sum, Min, Max, Avg }

#[derive(Debug, Serialize)]
pub struct Aggregate {
    pub func: AggregateFunction,
    pub arg: Identifier,
}