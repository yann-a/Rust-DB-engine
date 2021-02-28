mod types;
mod eval;
mod output;
mod optimize;

use crate::types::*;
use crate::eval::*;
use crate::output::*;
use crate::optimize::*;

fn main() {
    let expr = Box::new(Expression::Product(
        Box::new(Expression::Project(
            Box::new(Expression::Load(String::from("project_spec/samples/employes.csv"), None)), 
            vec![String::from("dpt"), String::from("email")]
        )),
        Box::new(Expression::Load(String::from("project_spec/samples/membres.csv"), None))
    ));

    // optimization phase
    let optimizer = ChainOptimizer{optimizers: vec![ Box::new(DetectLoadColumnsOptimizer{}) ]};
    let expr = optimizer.optimize(expr);

    let table = eval(expr);
    print_table(table);
}