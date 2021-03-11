mod types;
mod eval;
mod output;
mod optimize;
mod parser;
mod benchmark;

#[cfg(test)]
mod tests {
    pub mod eval_test;
    pub mod optimize_test;
}

use crate::eval::*;
use crate::output::*;
use crate::optimize::*;
use crate::parser::*;
use crate::benchmark::*;

use clap::{Arg, App};

fn main() {
    // Parse command-line arguments and options
    let args = App::new("Linear Algebra Engine on CSV files")
        .version("1.0")
        .author("Guilhem Niot <guilhem.niot@ens-lyon.fr>; Yann Aguettaz <yann.aguettaz@ens-lyon.fr>")
        .about("Takes JSON-formatted querries and runs them on CSV tables.\n Read the docs in the mardown files.")
        .arg(Arg::new("source_file")
            .index(1))
        .arg(Arg::new("output_file")
            .index(2))
        .arg(Arg::new("benchmark")
            .short('b')
            .long("benchmark"))
        .get_matches();

    // If benchmarkn, run it. Else parse input and evaluate
    if args.is_present("benchmark") {
        run_benchmark();
    } else {
        let source_file = args.value_of("source_file").map(|str| String::from(str));
        let output_file = args.value_of("output_file").map(|str| String::from(str));

        // Get expression from json
        let expr = Box::new(get_expression_from(source_file));

        // Optimization phase
        let optimizer = ChainOptimizer{optimizers: vec![
            Box::new(UnfoldComplexExpressionsOptimizer{}),
            Box::new(DetectLoadColumnsOptimizer{}),
            Box::new(PushDownSelectionsOptimizer{}),
            Box::new(ApplyProjectionsEarlyOptimizer{}),
            Box::new(FoldComplexExpressionsOptimizer{})
        ]};
        let expr = optimizer.optimize(expr);

        // Eval and print/write result
        let table = eval(expr);
        match output_file {
            Some(filename) => write_table(table, filename),
            None => print_table(table)
        }
    }
}
