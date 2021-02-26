#[derive(Debug)]
pub enum Value {
    Int(i64),
    Str(String),
    Column(String)
}

pub type Entry = Vec<Value>;

pub type Table = (Vec<String>, Vec<Entry>);

pub enum Condition {
    True,
    False,
    Or(Box<Condition>, Box<Condition>),
    And(Box<Condition>, Box<Condition>),
    Less(Box<Value>, Box<Value>),
    Equal(Box<Value>, Box<Value>),
    More(Box<Value>, Box<Value>)
}

pub enum Expression {
    Table(Table),
    Select(Box<Expression>, Box<Condition>),
    Project(Box<Expression>, Vec<String>), // expression, column names
    Rename(Box<Expression>, Vec<String>, Vec<String>), // expression, old column names, new column names
    Except(Box<Expression>, Box<Expression>),
    Union(Box<Expression>, Box<Expression>),
    Product(Box<Expression>, Box<Expression>),
    Load(String)
}