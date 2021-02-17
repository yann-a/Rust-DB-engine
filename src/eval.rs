use crate::types::*;

pub fn eval(expression: Box<Expression>) -> Table {
    match *expression {
        Expression::Table(table) => table,
        Expression::Select(expression_from, condition) => select(expression_from, condition),
        _ => (Vec::new(), Vec::new())
    }
}

fn select(expression: Box<Expression>, condition: Box<Condition>) -> Table {
    let (column_names, entries) = eval(expression);
    let new_entries: Vec<Entry> = entries.iter().filter(
        |entry| { eval_condition(entry, &column_names, condition) }
    ).collect();

    (column_names, new_entries)
}

fn eval_condition(entry: Entry, column_names: &Vec<String>, condition: Box<Condition>) -> bool {
    match *condition {
        Condition::True => true,
        Condition::False => false,
        Condition::And(c1, c2) => eval_condition(entry, column_names, c1) && eval_condition(entry, column_names, c2),
        _ => false
    }
}