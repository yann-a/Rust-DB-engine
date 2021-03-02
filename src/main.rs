mod types;
mod eval;
mod output;
mod optimize;
mod parser;

#[cfg(test)]
mod tests {
    pub mod eval_test;
    pub mod optimize_test;
}

use crate::types::*;
use crate::eval::*;
use crate::output::*;
use crate::optimize::*;
use crate::parser::*;

fn main() {
    let expr = Box::new(Expression::Product(
        Box::new(Expression::Load(String::from("project_spec/samples/projets.csv"), None)),
        Box::new(Expression::Project(
            Box::new(Expression::Rename(
                Box::new(Expression::Load(String::from("project_spec/samples/employes.csv"), None)),
                vec![String::from("dpt")],
                vec![String::from("test")]
            )),
            vec![String::from("test"), String::from("email")]
        )),
    ));

    let expr = Box::new(read_json());

    // optimization phase
    let optimizer = ChainOptimizer{optimizers: vec![
        Box::new(DetectLoadColumnsOptimizer{}),
        Box::new(ApplyProjectionsEarlyOptimizer{}),
    ]};
    let expr = optimizer.optimize(expr);

    let table = eval(expr);
    print_table(table);
}