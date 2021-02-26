use crate::types::*;
use csv::Reader;

pub fn eval(expression: Box<Expression>) -> Table {
    match *expression {
        Expression::Table(table) => table,
        Expression::Select(expression_from, condition) => select(expression_from, condition),
        Expression::Project(expression_from, columns) => project(expression_from, columns),
        Expression::Load(filename) => read(filename),
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

fn read(filename: String) -> Table {
    let mut rdr = Reader::from_path(filename).unwrap();

    let headers: Vec<String> = rdr.headers().unwrap().into_iter().map(|s| String::from(s)).collect();
    let entries: Vec<Entry> = rdr.records().into_iter().map(
        |record| record.unwrap().into_iter().map(
            |value| {
                match value.parse::<i64>() {
                    Ok(i) => Value::Int(i),
                    Err(_) => Value::Str(String::from(value))
                }
            }
        ).collect()
    ).collect();

    (headers, entries)
}

fn eval_condition(entry: &Entry, column_names: &Vec<String>, condition: &Box<Condition>) -> bool {
    match &**condition {
        Condition::True => true,
        Condition::False => false,
        Condition::And(c1, c2) => eval_condition(entry, column_names, &c1) && eval_condition(entry, column_names, &c2),
        _ => false
    }
}