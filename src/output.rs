use crate::types::*;
use csv::Writer;

pub fn print_table(t: Table) {
    let (fields, values) = t;

    let mut keys = vec![""; fields.len()];
    for (key, value) in &fields {
        keys[*value] = &key;
    }

    for key in keys {
        print!("{} ", key);
    }
    println!();
    for j in 0..values.len() {
        for i in 0..fields.len() {
            print!("{:?} ", values[j][i]);
        }
        println!();
    }
}

pub fn write_table(t: Table, filename: String) {
    let mut wtr = Writer::from_path(filename).unwrap();

    let (headers, entries) = t;

    wtr.write_record(headers.keys()).unwrap();
    for entry in entries {
        wtr.write_record(entry.into_iter().map(|v| get_string_for_value(v))).unwrap();
    }

    wtr.flush().unwrap();
}

fn get_string_for_value(v: Value) -> String {
    match v {
        Value::Int(i) => i.to_string(),
        Value::Str(s) => s,
        _ => panic!("A column shouldn't appear at this point")
    }
}