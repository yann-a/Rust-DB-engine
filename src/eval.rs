use crate::types::*;
use csv::Reader;
use std::collections::HashMap;


pub fn eval(expression: Box<Expression>) -> Table {
    match *expression {
        Expression::Table(table) => table,
        Expression::Select(expression_from, condition) => select(expression_from, condition),
        Expression::Project(expression_from, columns) => project(expression_from, columns),
        Expression::Product(expr1, expr2) => product(expr1, expr2),
        Expression::Except(expr1, expr2) => minus(expr1, expr2),
        Expression::Union(expr1, expr2) => union(expr1, expr2),
        Expression::ReadSelectProjectRename(filename, condition, old_attrs, new_attrs) => read_select_project_rename(filename, condition, old_attrs, new_attrs),
        Expression::Load(filename) => read(filename),
        _ => (HashMap::new(), Vec::new())
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
    let (column_names, mut entries) = eval(expression);
    let final_columns = HashMap::new();

    // On regarde dans les premières positions celles qui peuvent être utilisées
    let mut canBeUsed = vec![true; columns.len()];
    for column in columns.iter() {
        let index = *column_names.get(column).unwrap();
        if index < columns.len() {
            canBeUsed[index] = false;
            final_columns[column] = index;
        }
    }

    let swaps = Vec::new();

    // on va associer tout ça comme il faut
    let mut i = 0;
    for column in columns.iter() {
        let index = *column_names.get(column).unwrap();
        if index < columns.len() {
            continue; // on est déjà bon, rien à faire
        }

        // sinon, on cherche une nouvelle position
        while !canBeUsed[i] {
            i += 1;
        }

        swaps.push((index, i));
        final_columns[column] = i;
        i += 1;
    }

    for entry in entries {
        for (i, j) in swaps {
            entry.swap(i, j);
        }
        entry.truncate(columns.len())
    }

    (final_columns, entries)
}

fn product(expression1: Box<Expression>, expression2: Box<Expression>) -> Table {
    let (column_names1, entries1) = eval(expression1);
    let (column_names2, entries2) = eval(expression2);

    let mut final_entries: Vec<Entry> = Vec::new();

    for entry1 in entries1 {
        for entry2 in &entries2 {
            let mut entry = entry1.clone();
            entry.append(&mut entry2.clone());

            final_entries.push(entry);
        }
    }

    let final_columns = HashMap::new();
    for (key, value) in column_names1 {
        final_columns[&key] = value;
    }
    for (key, value) in column_names2 {
        final_columns[&key] = value;
    }

    (final_columns, final_entries)
}


fn minus(expression1: Box<Expression>, expression2: Box<Expression>) -> Table {
    let (column_names1, entries1) = eval(expression1);
    let (column_names2, entries2) = eval(expression2);

    let new_entries = entries1.into_iter().filter(
        |entry1| entries2.iter().all(|entry2| !(*entry1 == *entry2))
    ).collect();

    (column_names1, new_entries)
}

fn union(expression1: Box<Expression>, expression2: Box<Expression>) -> Table {
    let (column_names1, mut entries1) = eval(expression1);
    let (column_names2, mut entries2) = eval(expression2);

    entries1.append(&mut entries2);

    (column_names1, entries1)
}

fn read(filename: String) -> Table {
    let mut rdr = Reader::from_path(filename).unwrap();
    let mut column_names = HashMap::new();

    for (i, header) in rdr.headers().unwrap().into_iter().enumerate() {
        column_names.insert(String::from(header), i);
    }

    let entries: Vec<Entry> = rdr.records().map(
        |record| record.unwrap().into_iter().map(
            |value| {
                match value.parse::<i64>() {
                    Ok(i) => Value::Int(i),
                    Err(_) => Value::Str(String::from(value))
                }
            }
        ).collect()
    ).collect();

    (column_names, entries)
}

fn read_select_project_rename(filename: String, condition: Box<Condition>, old_attrs: Vec<String>, new_attrs: Vec<String>) -> Table {
    let mut rdr = Reader::from_path(filename).unwrap();
    let mut column_names = HashMap::new();

    for (i, header) in rdr.headers().unwrap().into_iter().enumerate() {
        column_names.insert(String::from(header), i);
    }
    let entries: Vec<Entry> = rdr.records().map(
        |record| record.unwrap().into_iter()
        .map(
            |value| {
                match value.parse::<i64>() {
                    Ok(i) => Value::Int(i),
                    Err(_) => Value::Str(String::from(value))
                }
            }
        )
        .collect()
    )
    .filter(|entry| eval_condition(entry, &column_names, &condition))
    // .map(|entry| )
    .collect();

    (column_names, entries)
}

fn eval_condition(entry: &Entry, column_names: &HashMap<String, usize>, condition: &Box<Condition>) -> bool {
    match &**condition {
        Condition::True => true,
        Condition::False => false,
        Condition::And(c1, c2) => eval_condition(entry, column_names, &c1) && eval_condition(entry, column_names, &c2),
        _ => false
    }
}