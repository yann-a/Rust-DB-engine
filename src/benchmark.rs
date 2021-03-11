use crate::types::*;
use crate::optimize::*;
use crate::eval::*;

use std::fs::File;
use std::fs;
use serde_derive::Deserialize;
use std::io::BufReader;
use std::time::Instant;


#[derive(Deserialize, Debug)]
struct Test {
    name: String,
    optims: Vec<String>
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
        "UCE" => Box::new(UnfoldComplexExpressionsOptimizer{}),
        "DLC" => Box::new(DetectLoadColumnsOptimizer{}),
        "PDS" => Box::new(PushDownSelectionsOptimizer{}),
        "APE" => Box::new(ApplyProjectionsEarlyOptimizer{}),
        "FCE" => Box::new(FoldComplexExpressionsOptimizer{}),
        _ => panic!(format!("unknown optimization: {}", opti))
    }
}

fn run_benchmark_on(path: String, n: Option<u128>) {
    let benchmark = get_benchmark_from(path.clone());

    let expression = Box::new(benchmark.input);
    let tests = benchmark.tests;

    let nb_it = n.unwrap_or(100);

    println!("### Running benchmark {} ###\n", path);

    for test in tests {
        let optims = test.optims.into_iter().map(|opti| opti_from_string(opti)).collect();
        let optimizer = ChainOptimizer{optimizers: optims};

        let expr = optimizer.optimize(expression.clone());

        let mut total_time = std::time::Duration::new(0, 0);

        for _ in 0..nb_it {
            let time_before = Instant::now();
            eval(expr.clone());
            let time_elapsed = time_before.elapsed();

            total_time += time_elapsed;
        }
        
        println!("{} took {:.2?} on average", test.name, total_time/(nb_it as u32));
    }
}

pub fn run_benchmark() {
    // Get a vector of all filenaames inside "expr_samples/benchmarks"
    let mut entries : Vec<_> = fs::read_dir("expr_samples/benchmarks").
        unwrap()
        .map(|res| {
            res.map(|e| e.path())
            .unwrap()
            .to_str()
            .map(|p| String::from(p))
            .unwrap()
        })
        .collect();

    // Sort them
    entries.sort();

    // Run benchmarking on each of them
    for entry in entries {
        run_benchmark_on(entry, Some(100));
    }
}