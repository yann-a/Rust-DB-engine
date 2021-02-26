mod types;
mod eval;
mod output;
mod optimize;

use crate::types::*;
use crate::eval::*;
use crate::output::*;
use crate::optimize::*;

fn main() {
    let v = Box::new(Expression::Product(
        Box::new(Expression::Project(
            Box::new(Expression::Load(String::from("project_spec/samples/employes.csv"))), 
            vec![String::from("dpt"), String::from("email")]
        )),
        Box::new(Expression::Load(String::from("project_spec/samples/membres.csv")))
    ));

    // optimization phase
    let optimizer = ChainOptimizer{optimizers: Vec::new()};
    let expr = optimizer.optimize(v);

    let _x = eval(expr);
    print_table(_x);
}