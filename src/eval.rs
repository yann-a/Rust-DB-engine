use crate::types::*;
use csv::Reader;
use std::collections::{HashMap,HashSet};


pub fn eval(expression: Box<Expression>) -> Table {
    match *expression {
        Expression::Select(expression_from, condition) => select(expression_from, condition),
        Expression::Project(expression_from, columns) => project(expression_from, columns),
        Expression::Product(expr1, expr2) => product(expr1, expr2),
        Expression::Except(expr1, expr2) => minus(expr1, expr2),
        Expression::Union(expr1, expr2) => union(expr1, expr2),
        Expression::Rename(expression, old_columns, new_columns) => renaming(expression, old_columns, new_columns),
        Expression::ReadSelectProjectRename(filename, condition, old_attrs, new_attrs) => read_select_project_rename(filename, condition, old_attrs, new_attrs),
        Expression::JoinProjectRename(expr1, expr2, condition, old_attrs, new_attrs) => join_project_rename(expr1, expr2, condition, old_attrs, new_attrs),
        Expression::Load(filename, _) => read(filename),
    }
}

fn select(expression: Box<Expression>, condition: Box<Condition>) -> Table {
    let (column_names, entries) = eval(expression);
    let new_entries: Vec<Entry> = entries.into_iter().filter(
        |entry| eval_condition(entry, &column_names, &condition)
    ).collect();

    (column_names, new_entries)
}

fn swaps_for_projection(column_names: &HashMap<String, usize>, columns: &Vec<String>) -> (Vec<(usize, usize)>, HashMap<String, usize>) {

    let mut final_columns: HashMap<String, usize> = HashMap::new();

    // On regarde dans les premières positions celles qui peuvent être utilisées
    let mut can_be_used = vec![true; columns.len()];
    for column in columns.iter() {
        let index = *column_names.get(column).unwrap();
        if index < columns.len() {
            can_be_used[index] = false;
            final_columns.insert(column.clone(), index);
        }
    }

    let mut swaps = Vec::new();

    // on va associer tout ça comme il faut
    let mut i = 0;
    for column in columns.iter() {
        let index = *column_names.get(column).unwrap();
        if index < columns.len() {
            continue; // on est déjà bon, rien à faire
        }

        // sinon, on cherche une nouvelle position
        while !can_be_used[i] {
            i += 1;
        }

        swaps.push((index, i));
        final_columns.insert(column.clone(), i);
        i += 1;
    }

    (swaps, final_columns)
}

fn project(expression: Box<Expression>, columns: Vec<String>) -> Table {
    let (column_names, mut entries) = eval(expression);

    let (swaps, final_columns) = swaps_for_projection(&column_names, &columns);

    for entry in &mut entries {
        for (i, j) in &swaps {
            entry.swap(*i, *j);
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

    let mut final_columns = HashMap::new();
    for (key, value) in column_names2 {
        final_columns.insert(key, column_names1.len()+value);
    }
    for (key, value) in column_names1 {
        final_columns.insert(key, value);
    }

    (final_columns, final_entries)
}

fn renaming(expression: Box<Expression>, old_columns: Vec<String>, new_columns: Vec<String>) -> Table {
    let (mut column_names, entries) = eval(expression);
    rename_columns(&mut column_names, old_columns, new_columns);

    (column_names, entries)
}

fn rename_columns(column_names: &mut HashMap<String, usize>, old_columns: Vec<String>, new_columns: Vec<String>) {
    for (i, new_column) in new_columns.into_iter().enumerate() {
        let index = column_names.remove(&old_columns[i]).unwrap();
        column_names.insert(new_column, index);
    }
}

fn minus(expression1: Box<Expression>, expression2: Box<Expression>) -> Table {
    let (column_names1, entries1) = eval(expression1);
    let (_column_names2, entries2) = eval(expression2);

    let new_entries = entries1.into_iter().filter(
        |entry1| entries2.iter().all(|entry2| !(*entry1 == *entry2))
    ).collect();

    (column_names1, new_entries)
}

fn union(expression1: Box<Expression>, expression2: Box<Expression>) -> Table {
    let (column_names1, mut entries1) = eval(expression1);
    let (mut column_names2, entries2) = eval(expression2);

    let mut columns = vec!["".to_string(); column_names2.len()];
    for (column, index) in &column_names2 {
        columns[*index] = column.clone();
    }

    let mut swaps = Vec::new();
    for (column, index) in &column_names1 {
        let actual_index_mut = column_names2.get_mut(column).unwrap();
        let actual_index = actual_index_mut.clone();
        if *index == actual_index {
            continue;
        }

        swaps.push((*index, actual_index));

        // On update les positions
        *actual_index_mut = *index;
        column_names2.insert(columns[*index].clone(), actual_index);
    }

    entries1.append(&mut entries2.into_iter().map(|mut entry| {
        for (i, j) in &swaps {
            entry.swap(*i, *j);
        }
        entry
    }).collect());

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

    let (swaps, mut final_columns) = swaps_for_projection(&column_names, &old_attrs);
    let entries: Vec<Entry> = rdr.records().map(
        |record|
            record.unwrap().into_iter()
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
    // chaining map, then filter is optimized by rust
    .filter(|entry| eval_condition(entry, &column_names, &condition))
    .map(|mut record| {
        for (i, j) in &swaps {
            record.swap(*i, *j);
        }
        record.truncate(old_attrs.len());

        record
    })
    .collect();

    rename_columns(&mut final_columns, old_attrs, new_attrs);

    (final_columns, entries)
}

fn join_project_rename(expr1: Box<Expression>, expr2: Box<Expression>, condition: Box<Condition>, old_attrs: Vec<String>, new_attrs: Vec<String>) -> Table {
    let (column_names1, entries1) = eval(expr1);
    let (column_names2, entries2) = eval(expr2);

    // On se repose sur un hash join pour accélérer les cross product
    let mut unsupported_conditions = Box::new(Condition::True);
    let mut conditions_to_treat = vec![condition];
    let mut bucket1 = HashSet::new();

    while let Some(condition) = conditions_to_treat.pop() {
        match *condition {
            Condition::Equal(Value::Column(f1), Value::Column(f2))
                if (column_names1.contains_key(&f1) && column_names2.contains_key(&f2)) || (column_names2.contains_key(&f1) && column_names1.contains_key(&f2)) => 
            {
                if column_names1.contains_key(&f1) {
                    bucket1.insert((f1, f2));
                } else {
                    bucket1.insert((f2, f1));
                }
            },
            Condition::And(c1, c2) => {
                conditions_to_treat.push(c1);
                conditions_to_treat.push(c2);
            },
            _ => unsupported_conditions = Box::new(Condition::And(unsupported_conditions, condition))
        }
    }

    let indexes = bucket1.into_iter().map(|(field1, field2)|
        (column_names1.get(&field1).unwrap(), column_names2.get(&field2).unwrap())
    ).collect::<Vec<_>>();

    let mut buckets = HashMap::new();

    for entry1 in entries1 {
        let mut repr = Vec::new();
        for (id, _) in &indexes {
            repr.push(entry1[**id].clone());
        }

        let mut bucket = buckets.get_mut(&repr);
        if bucket.is_none() {
            buckets.insert(repr.clone(), Vec::new());
            bucket = buckets.get_mut(&repr);
        }
        let bucket = bucket.unwrap();
        
        bucket.push(entry1);
    }

    let mut final_entries: Vec<Entry> = Vec::new();

    for entry2 in &entries2 {
        let mut repr = Vec::new();
        for (_, id) in &indexes {
            repr.push(entry2[**id].clone());
        }

        // On fait le produit avec les éléments du bucket qui correspond
        let bucket = buckets.get(&repr);
        if let Some(entries) = bucket {
            for entry in entries {
                let mut entry = entry.clone();
                entry.append(&mut entry2.clone());

                final_entries.push(entry);
            }
        }
    }

    let mut final_columns = HashMap::new();
    for (key, value) in column_names2 {
        final_columns.insert(key, column_names1.len()+value);
    }
    for (key, value) in column_names1 {
        final_columns.insert(key, value);
    }

    let (swaps, mut swapped_columns) = swaps_for_projection(&final_columns, &old_attrs);
    let final_entries = final_entries.into_iter()
        .filter(|entry| eval_condition(entry, &final_columns, &unsupported_conditions))
        .map(|mut record| {
            for (i, j) in &swaps {
                record.swap(*i, *j);
            }
            record.truncate(old_attrs.len());

            record
        })
        .collect();

    rename_columns(&mut swapped_columns, old_attrs, new_attrs);

    (swapped_columns, final_entries)
}

fn eval_condition(entry: &Entry, column_names: &HashMap<String, usize>, condition: &Box<Condition>) -> bool {
    match &**condition {
        Condition::True => true,
        Condition::False => false,
        Condition::Not(c) => !eval_condition(entry, column_names, &c),
        Condition::And(c1, c2) => eval_condition(entry, column_names, &c1) && eval_condition(entry, column_names, &c2),
        Condition::Or(c1, c2) => eval_condition(entry, column_names, &c1) || eval_condition(entry, column_names, &c2),
        Condition::Equal(v1, v2) => 
            match (get_value(entry, column_names, v1), get_value(entry, column_names, v2)) {
                (Value::Int(i), Value::Int(j)) => i == j,
                (Value::Str(i), Value::Str(j)) => i == j,
                _ => false
            },
        Condition::Less(v1, v2) => 
            match (get_value(entry, column_names, v1), get_value(entry, column_names, v2)) {
                (Value::Int(i), Value::Int(j)) => i < j,
                _ => false
            },
        Condition::More(v1, v2) => 
            match (get_value(entry, column_names, v1), get_value(entry, column_names, v2)) {
                (Value::Int(i), Value::Int(j)) => i > j,
                _ => false
            }
    }
}

fn get_value(entry: &Entry, column_names: &HashMap<String, usize>, value: &Value) -> Value {
    match value {
        Value::Column(s) => entry[*column_names.get(s).unwrap()].clone(),
        v => v.clone()
    }
}