use crate::types::*;

pub fn print_table(t: Table) {
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