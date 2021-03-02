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

use std::env;

fn main() {
    let _expr = Box::new(Expression::Product(
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
    // Read command-line arguments
    let args: Vec<String> = env::args().collect();
    let source_file = &args[1];
    let mut output_file: Option<String> = None;
    if args.len() > 2 {
        output_file = Some(String::from(&args[2]));
    }

    // Get expression from json
    let expr = Box::new(get_expression_from_file(String::from(source_file)));

    // Optimization phase
    let optimizer = ChainOptimizer{optimizers: vec![
        Box::new(DetectLoadColumnsOptimizer{}),
        Box::new(ApplyProjectionsEarlyOptimizer{}),
    ]};
    let expr = optimizer.optimize(expr);

    // Eval and print result
    let table = eval(expr);
    match output_file {
        Some(filename) => write_table(table, filename),
        None => print_table(table)
    }
}