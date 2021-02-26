mod types;
mod eval;
mod output;
use crate::types::*;
use crate::eval::*;
use crate::output::*;

fn main() {
    let v = Box::new(Expression::Table((vec![String::from("Id")], vec![vec![Value::Int(5)]])));
    let _x = eval(v);
    print_table(_x);
}