use crate::types::*;
use crate::output::*;

pub fn eval(expression: Box<Expression>) -> Table {
    match *expression {
        Expression::Table(table) => table,
        Expression::Select(expression_from, condition) => select(expression_from, condition),
        Expression::Project(expression_from, columns) => project(expression_from, columns),
        _ => (Vec::new(), Vec::new())
    }
}

fn select(expression: Box<Expression>, condition: Box<Condition>) -> Table {
    let (column_names, entries) = eval(expression);
    let new_entries: Vec<Entry> = entries.into_iter().filter(
        |entry| eval_condition(entry, &column_names, &condition) 
    ).collect();

    (column_names, new_entries)
}

fn project(expression: Box<Expression>, columns: Vec<String>) -> Table {
    let (mut column_names, mut entries) = eval(expression);
    
    for (i, column) in columns.iter().enumerate() {
        let actual_pos = column_names.iter().position(|column_name| *column_name == *column).unwrap();

        if i == actual_pos {
            continue; // on est déjà content
        }

        for entry in &mut entries {
            entry.swap(i, actual_pos);
        }
        column_names.swap(i, actual_pos);
    }

    // on enlève les champs en trop
    for entry in &mut entries {
        entry.truncate(columns.len());
    }

    (columns, entries)
}

fn eval_condition(entry: &Entry, column_names: &Vec<String>, condition: &Box<Condition>) -> bool {
    match &**condition {
        Condition::True => true,
        Condition::False => false,
        Condition::And(c1, c2) => eval_condition(entry, column_names, &c1) && eval_condition(entry, column_names, &c2),
        _ => false
    }
}