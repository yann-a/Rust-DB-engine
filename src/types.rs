use crate::parser::*;

use serde_derive::Deserialize;
use std::collections::HashMap;
use std::collections::HashSet;


#[derive(Debug, Clone, Deserialize)]
pub enum Value {
    Int(i64),
    Str(String),
    Column(String)
}
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int(i), Value::Int(j)) => i==j,
            (Value::Str(s), Value::Str(t)) => s==t,
            (Value::Column(s), Value::Column(t)) => s==t,
            (_, _) => false
        }
    }
}
impl Eq for Value {}

pub type Entry = Vec<Value>;

pub type Table = (HashMap<String, usize>, Vec<Entry>);

#[derive(Debug, Clone)]
pub enum Condition {
    Not(Box<Condition>),
    Or(Box<Condition>, Box<Condition>),
    And(Box<Condition>, Box<Condition>),
    Less(Value, Value),
    Equal(Value, Value),
    More(Value, Value)
}

impl PartialEq for Condition {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Condition::Not(c1), Condition::Not(c2)) => *c1==*c2,
            (Condition::Or(c11, c12), Condition::Or(c21, c22)) |
                (Condition::And(c11, c12), Condition::And(c21, c22)) => *c11==*c21 && *c12 == *c22,
            (Condition::Less(v11, v12), Condition::Less(v21, v22)) |
                (Condition::Equal(v11, v12), Condition::Equal(v21, v22)) |
                (Condition::More(v11, v12), Condition::More(v21, v22)) => v11==v21 && v12 == v22,
            (_, _) => false
        }
    }
}
impl Eq for Condition {}

#[derive(Deserialize, Debug, Clone)]
#[serde(from = "ExpressionParse", into = "ExpressionParse")] 
pub enum Expression {
    Select(Box<Expression>, Box<Condition>),
    Project(Box<Expression>, Vec<String>), // expression, column names
    Rename(Box<Expression>, Vec<String>, Vec<String>), // expression, old column names, new column names
    Except(Box<Expression>, Box<Expression>),
    Union(Box<Expression>, Box<Expression>),
    Product(Box<Expression>, Box<Expression>),
    ReadSelectProjectRename(String, Box<Condition>, Vec<String>, Vec<String>),
    Load(String, Option<HashSet<String>>) // Optionally contains the columns to be loaded for future optimizations
}

impl PartialEq for Expression {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Expression::Select(e1, c1), Expression::Select(e2, c2)) => *e1==*e2 && *c1==*c2,
            (Expression::Project(e1, on1), Expression::Project(e2, on2)) => {
                let mut o1 = on1.clone(); let mut o2 = on2.clone();
                o1.sort(); o2.sort(); // We don't care about the order
                *e1==*e2 && o1==o2
            },
            (Expression::Rename(e1, old1, new1), Expression::Rename(e2, old2, new2)) => *e1==*e2 && old1==old2 && new1 == new2,
            (Expression::Except(e11, e12), Expression::Except(e21, e22)) |
                (Expression::Union(e11, e12), Expression::Union(e21, e22)) |
                (Expression::Product(e11, e12), Expression::Product(e21, e22)) => *e11==*e21 && *e12 == *e22,
            (Expression::ReadSelectProjectRename(f1, c1, old1, new1), Expression::ReadSelectProjectRename(f2, c2, old2, new2)) => f1==f2 && *c1 == *c2 && old1==old2 && new1==new2,
            (Expression::Load(f1, _), Expression::Load(f2, _)) => f1 == f2,
            (_, _) => false
        }
    }
}
impl Eq for Expression {}