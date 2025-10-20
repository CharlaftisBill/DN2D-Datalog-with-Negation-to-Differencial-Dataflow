use serde::Serialize;

#[derive(Debug, Serialize, Clone, Copy)]
pub enum BinaryOperator { Add, Sub, Mul, Div, Mod, Eq, NotEq, Lt, LtEq, Gt, GtEq }

#[derive(Debug, Serialize, Clone, Copy)]
pub enum UnaryOperator { Neg }