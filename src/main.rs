#[derive(PartialOrd, Ord, Debug)]
enum Value {
    Int(u64),
    Str(&'static str),
    Column(&'static str)
}
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int(i), Value::Int(j)) => i==j,
            (Value::Str(s), Value::Str(t)) => s==t,
            (_, _) => false
        }
    }
}
impl Eq for Value {}

type Entry = Vec<Value>;

type Table = (Vec<&'static str>, Vec<Entry>);

enum Condition {
    True,
    False,
    Or(Box<Condition>, Box<Condition>),
    And(Box<Condition>, Box<Condition>),
    Less(Box<Value>, Box<Value>),
    Equal(Box<Value>, Box<Value>),
    More(Box<Value>, Box<Value>)
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
    let (fields, values) = eval(e);
    let t_res: Vec<Entry> = values.into_iter().filter(|entry| { eval_cond_on_entry(&cond, &fields, &entry) }).collect();

    return (fields, t_res);
}

fn eval_cond_on_entry(cond: &Condition, fields: &Vec<&'static str>, e: &Entry) -> bool {
    match cond {
        Condition::True => true,
        Condition::False => false,
        Condition::And(c1, c2) => eval_cond_on_entry(c1, &fields, e) && eval_cond_on_entry(c2, &fields, e),
        Condition::Or(c1, c2) => eval_cond_on_entry(c1, &fields, e) || eval_cond_on_entry(c2, &fields, e),
        Condition::Equal(v1, v2) => *get_value(v1, &fields, e) == *get_value(v2, &fields, e),
        Condition::Less(v1, v2) => *get_value(v1, &fields, e) < *get_value(v2, &fields, e),
        Condition::More(v1, v2) => *get_value(v1, &fields, e) > *get_value(v2, &fields, e)
    }
}

fn get_value<'a, 'b>(value: &'a Value, fields: &'b Vec<&'static str>, e: &'a Entry) -> &'a Value {
    match value {
        Value::Int(_) | Value::Str(_) => value,
        Value::Column(field) => {
            for i in 0..fields.len() {
                if *field == fields[i] {
                    return &e[i];
                }
            }
            return &Value::Int(2306); // Shouldn't happen
        }
    }
}

fn print_table(t: Table) {
    let (fields, values) = t;

    for i in 0..fields.len() {
        print!("{} ", fields[i]);
    }
    println!();
    for j in 0..values.len() {
        for i in 0..fields.len() {
            print!("{:?} ", values[j][i]);
        }
        println!();
    }
}

fn main() {
    let req = Expression::Selection(
        Box::new(
            Expression::Table((
                vec![&"Id", &"Nom", &"Nb"],
                vec![
                    vec![Value::Int(5), Value::Str(&"Guilhem"), Value::Int(5)],
                    vec![Value::Int(12), Value::Str(&"Yann"), Value::Int(23)],
                    vec![Value::Int(23), Value::Str(&"JFP"), Value::Int(2)]
                ]
            ))
        ),
        Box::new(
            Condition::Equal(Box::new(Value::Column(&"Id")), Box::new(Value::Int(12)))
        )
    );

    let result = eval(req);

    print_table(result);
}