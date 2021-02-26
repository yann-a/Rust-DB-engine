mod types;
mod eval;
mod output;
use crate::types::*;
use crate::eval::*;
use crate::output::*;

fn main() {
    let v = Box::new(Expression::Product(
        Box::new(Expression::Project(
            Box::new(Expression::Load(String::from("project_spec/samples/employes.csv"))), 
            vec![String::from("dpt"), String::from("email")]
        )),
        Box::new(Expression::Load(String::from("project_spec/samples/membres.csv")))
    ));

    
    let _x = eval(v);
    print_table(_x);
}