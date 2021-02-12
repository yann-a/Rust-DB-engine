#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum Value {
    Int(u64),
    Str(&'static str)
}

type Entry = Vec<Value>;

type Table = (Vec<&'static str>, Vec<Entry>);

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
    Table(Table),
    Selection(Box<Expression>, Box<Condition>),
    Project(Vec<&'static str>, Box<Expression>),
    Renaming(Vec<&'static str>, Vec<&'static str>, Box<Expression>),
    Minus(Box<Expression>, Box<Expression>),
    Union(Box<Expression>, Box<Expression>),
    Product(Box<Expression>, Box<Expression>),
    Load(&'static str)
}

fn eval(e: Expression) -> Table {
    match e {
        Expression::Table(table) => table,
        Expression::Selection(e_from, cond) => select(*e_from, *cond),
        _ => (Vec::new(), Vec::new())
    }
}

fn select(e: Expression, cond: Condition) -> Table {
    let t_from = eval(e);
    let mut t_res: Vec<Entry> = Vec::new();

    for entry in &(t_from.1) {
        if eval_cond_on_entry(cond, t_from.0, entry) {
            t_res.push(*entry);
        }
    }

    return (t_from.0, t_res);
}

fn eval_cond_on_entry(cond: Condition, fields: Vec<&'static str>, e: &Entry) -> bool {
    match cond {
        Condition::True => true,
        Condition::False => false,
        Condition::And(c1, c2) => eval_cond_on_entry(*c1, fields, e) && eval_cond_on_entry(*c2, fields, e),
        Condition::Or(c1, c2) => eval_cond_on_entry(*c1, fields, e) || eval_cond_on_entry(*c2, fields, e),
        Condition::Equal(f1, f2) => get_value(f1, fields, e) == get_value(f2, fields, e),
        Condition::Less(f1, f2) => get_value(f1, fields, e) < get_value(f2, fields, e),
        Condition::More(f1, f2) => get_value(f1, fields, e) > get_value(f2, fields, e)
    }
}

fn get_value(field: &'static str, fields: Vec<&'static str>, e: Entry) -> Value {
    for i in 0..fields.len()-1 {
        if field == fields[i] {
            return e[i];
        }
    }

    return Value::Int(0);
}

fn main() {
    println!("Hello, worldd!");
}