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

    let source_file = if args.len() > 1 { Some(String::from(&args[1])) } else { None };
    let output_file = if args.len() > 2 { Some(String::from(&args[2])) } else { None };

    // Get expression from json
    let expr = Box::new(get_expression_from(source_file));

    // Optimization phase
    let optimizer = ChainOptimizer{optimizers: vec![
        Box::new(DetectLoadColumnsOptimizer{}),
        Box::new(PushDownSelectionsOptimizer{}),
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
