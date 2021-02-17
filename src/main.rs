mod types;
mod eval;
use crate::types::*;
use crate::eval::*;

fn main() {
    let v = Box::new(Expression::Table((vec![String::from("Id")], vec![vec![Value::Int(5)]])));
    let _x = eval(v);
}