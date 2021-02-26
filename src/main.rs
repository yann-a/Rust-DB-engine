mod types;
mod eval;
mod output;
use crate::types::*;
use crate::eval::*;
use crate::output::*;

fn main() {
    let v = Box::new(Expression::Load(String::from("project_spec/samples/employes.csv")));
    let _x = eval(v);
    print_table(_x);
}