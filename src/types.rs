use serde_derive::Deserialize;
use std::collections::HashMap;

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

#[derive(Deserialize)]
pub enum Condition {
    True,
    False,
    Or(Box<Condition>, Box<Condition>),
    And(Box<Condition>, Box<Condition>),
    Less(Box<Value>, Box<Value>),
    Equal(Box<Value>, Box<Value>),
    More(Box<Value>, Box<Value>)
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase", tag = "operation", content = "args")] 
pub enum Expression {
    //#[serde(skip)]
    //Table(Table),
    #[serde(rename="selection")]
    Select(Box<Expression>, Box<Condition>),
    #[serde(rename="projection")]
    Project(Box<Expression>, Vec<String>), // expression, column names
    #[serde(rename="renaming")]
    Rename(Box<Expression>, Vec<String>, Vec<String>), // expression, old column names, new column names
    #[serde(rename="minus")]
    Except(Box<Expression>, Box<Expression>),
    Union(Box<Expression>, Box<Expression>),
    Product(Box<Expression>, Box<Expression>),
    ReadSelectProjectRename(String, Box<Condition>, Vec<String>, Vec<String>),
    Load(String)
}