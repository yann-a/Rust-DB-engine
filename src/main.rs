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

    let source_file = match args.len() {
        nbarg if nbarg >1 => Some(String::from(&args[1])),
        _ => None
    };
    let output_file = match args.len() {
        nbarg if nbarg>2 => Some(String::from(&args[2])),
        _ => None
    };

    // Get expression from json
    let expr = Box::new(get_expression_as_input(source_file));

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
