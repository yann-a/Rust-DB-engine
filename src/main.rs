enum Condition {
    True,
    False,
    Or(Box<Condition>, Box<Condition>),
    And(Box<Condition>, Box<Condition>),
    Less(&'static str, &'static str),
    Equal(&'static str, &'static str),
    More(&'static str, &'static str)
}

enum Expression {
    Table,
    Selection(Box<Expression>, Box<Condition>),
    Project(Vec<&'static str>, Box<Expression>),
    Renaming(Vec<&'static str>, Vec<&'static str>, Box<Expression>),
    Minus(Box<Expression>, Box<Expression>),
    Union(Box<Expression>, Box<Expression>),
    Load(&'static str)
}

fn main() {
    println!("Hello, worldd!");
}