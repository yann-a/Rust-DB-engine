use crate::types::*;
use crate::optimize::*;
use crate::eval::*;

use std::fs::File;
use serde_derive::Deserialize;
use std::io::BufReader;

use std::time::Instant;


#[derive(Deserialize, Debug)]
struct Test {
    nom: String,
    optis: Vec<String>
}

#[derive(Deserialize, Debug)]
struct Benchmark {
    input: Expression,
    tests: Vec<Test>
}

fn get_benchmark_from(path: String) -> Benchmark {
    let file = File::open(path).unwrap();

    serde_json::from_reader(BufReader::new(file)).unwrap()
}

fn opti_from_string(opti: String) -> Box<dyn Optimizer> {
    match &opti[..] {
        "DLC" => Box::new(DetectLoadColumnsOptimizer{}),
        "PDS" => Box::new(PushDownSelectionsOptimizer{}),
        "APE" => Box::new(ApplyProjectionsEarlyOptimizer{}),
        "FCE" => Box::new(FoldComplexExpressionsOptimizer{}),
        _ => panic!(format!("unknown optimization: {}", opti))
    }
}

pub fn run_benchmark(path: String) {
    let benchmark = get_benchmark_from(path.clone());

    let expression = Box::new(benchmark.input);
    let tests = benchmark.tests;

    println!("### Running benchmark {} ###\n", path);

    for test in tests {
        let optims = test.optis.into_iter().map(|opti| opti_from_string(opti)).collect();
        let optimizer = ChainOptimizer{optimizers: optims};

        let expr = optimizer.optimize(expression.clone());

        let time_before = Instant::now();
        eval(expr);
        let time_elapsed = time_before.elapsed();
        
        println!("{} took {:.2?}", test.nom, time_elapsed);
    }
}