use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Program {
    pub clauses: Vec<Clause>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Clause {
    Fact(Predicate),
    Rule(Rule),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Rule {
    pub head: Predicate,
    pub body: Vec<Literal>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Literal {
    pub predicate: Predicate,
    pub is_negated: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Predicate {
    pub name: String,
    pub terms: Vec<Term>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Term {
    Variable(Variable),
    Constant(Constant),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Variable {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Constant {
    Atom(String),
    Number(i64),
    String(String),
}