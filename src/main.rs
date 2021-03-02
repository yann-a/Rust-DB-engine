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

use crate::eval::*;
use crate::output::*;
use crate::optimize::*;
use crate::parser::*;

use std::env;

fn main() {
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

    // Eval and print/write result
    let table = eval(expr);
    match output_file {
        Some(filename) => write_table(table, filename),
        None => print_table(table)
    }
}
