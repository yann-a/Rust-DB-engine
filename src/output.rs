use crate::types::*;

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