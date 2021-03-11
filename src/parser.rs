use crate::types::*;
use std::fs::File;

use serde_derive::Deserialize;
use std::io::BufReader;
use std::io::{self, Read};

#[derive(Deserialize)]
#[serde(untagged)]
pub enum ConditionParse {
    True,
    False,
    Log1 {logical: String, condition: Box<ConditionParse>},
    Log2 {logical: String, condition1: Box<ConditionParse>, condition2: Box<ConditionParse>},
    Comp {comparator: String, attribute1: String, attribute2: String}
}

#[derive(Deserialize)]
#[serde(tag = "operation", content = "args")] 
pub enum ExpressionParse {
    #[serde(rename = "selection")]
    Select {object: Box<ExpressionParse>, condition: Box<ConditionParse>},
    #[serde(rename = "projection")]
    Project {object: Box<ExpressionParse>, attributes: Vec<String>},
    #[serde(rename = "renaming")]
    Rename {object: Box<ExpressionParse>, #[serde(rename = "old attributes")] old_attributes: Vec<String>, #[serde(rename = "new attributes")] new_attributes: Vec<String>},
    #[serde(rename = "minus")]
    Except {object1: Box<ExpressionParse>, object2: Box<ExpressionParse>},
    #[serde(rename = "union")]
    Union {object1: Box<ExpressionParse>, object2: Box<ExpressionParse>},
    #[serde(rename = "product")]
    Product {object1: Box<ExpressionParse>, object2: Box<ExpressionParse>},
    #[serde(rename = "load")]
    Read {filename: String},
    #[serde(rename = "rspr")]
    ReadSelectProjectRename {filename: String, condition: Box<ConditionParse>, #[serde(rename = "old attributes")] old_attributes: Vec<String>, #[serde(rename = "new attributes")] new_attributes: Vec<String>},
    JoinProjectRename {object1: Box<ExpressionParse>, object2: Box<ExpressionParse>, condition: Box<ConditionParse>, #[serde(rename = "old attributes")] old_attributes: Vec<String>, #[serde(rename = "new attributes")] new_attributes: Vec<String>}
}

impl From<ConditionParse> for Condition {
    fn from(condition: ConditionParse) -> Condition {
        match condition {
            ConditionParse::True => Condition::True,
            ConditionParse::False => Condition::False,
            ConditionParse::Log1 {logical: op, condition: c} =>
                match &op[..] {
                   "not" => Condition::Not(Box::new(Condition::from(*c))),
                    _ => panic!(format!("unknown conditional operator of arity 1 {}", op))
                },
            ConditionParse::Log2 {logical: op, condition1: c1, condition2: c2} => 
                match &op[..] {
                    "or" => Condition::Or(Box::new(Condition::from(*c1)), Box::new(Condition::from(*c2))),
                    "and" => Condition::And(Box::new(Condition::from(*c1)), Box::new(Condition::from(*c2))),
                    _ => panic!(format!("unknown conditional operator of arity 2 {}", op))
                },
            ConditionParse::Comp {comparator: c, attribute1: a1, attribute2: a2} => {
                let v1 = match a1.parse::<i64>() {
                    Ok(i) => Value::Int(i),
                    Err(_) => Value::Column(String::from(a1))
                };
                let v2 = match a2.parse::<i64>() {
                    Ok(i) => Value::Int(i),
                    Err(_) => Value::Column(String::from(a2))
                };

                match &c[..] {
                    "=" => Condition::Equal(v1, v2),
                    "<" => Condition::Less(v1, v2),
                    ">" => Condition::More(v1, v2),
                    _ => panic!(format!("unknown comparator {}", c))
                }
            }
        }
    }
}

impl From<ExpressionParse> for Expression {
    fn from(expression: ExpressionParse) -> Expression {
        match expression {
            ExpressionParse::Select {object: o, condition: c} => Expression::Select(Box::new(Expression::from(*o)), Box::new(Condition::from(*c))),
            ExpressionParse::Project {object: o, attributes: a} => Expression::Project(Box::new(Expression::from(*o)), a),
            ExpressionParse::Rename {object: o, old_attributes: oa, new_attributes: na} => Expression::Rename(Box::new(Expression::from(*o)), oa, na),
            ExpressionParse::Except {object1: o1, object2: o2} => Expression::Except(Box::new(Expression::from(*o1)), Box::new(Expression::from(*o2))),
            ExpressionParse::Union {object1: o1, object2: o2} => Expression::Union(Box::new(Expression::from(*o1)), Box::new(Expression::from(*o2))),
            ExpressionParse::Product {object1: o1, object2: o2} => Expression::Product(Box::new(Expression::from(*o1)), Box::new(Expression::from(*o2))),
            ExpressionParse::Read {filename: f} => Expression::Load(f, None),
            ExpressionParse::ReadSelectProjectRename {filename: f, condition: c, old_attributes: oa, new_attributes: na} => Expression::ReadSelectProjectRename(f, Box::new(Condition::from(*c)), oa, na),
            ExpressionParse::JoinProjectRename {object1: o1, object2: o2, condition: c, old_attributes: oa, new_attributes: na} => Expression::JoinProjectRename(Box::new(Expression::from(*o1)), Box::new(Expression::from(*o2)), Box::new(Condition::from(*c)), oa, na)
        }
    }
}

#[allow(dead_code)]
pub fn get_expression_from_str(json: &'static str) -> Expression {
    serde_json::from_str(json).unwrap()
}

pub fn get_expression_from(path: Option<String>) -> Expression {
    match path {
        Some(filename) => {
            let file = File::open(filename).unwrap();
            
            serde_json::from_reader(BufReader::new(file)).unwrap()
        },
        None => {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer).unwrap();

            serde_json::from_str(&buffer).unwrap()
        }
    }
}
