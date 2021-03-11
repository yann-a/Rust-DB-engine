# DBDM Project

This file serves as both a documentation and a report of our project

**Authors**:
* Guilhem Niot
* Yann Aguettaz

## Specifications

### Usage

Use the following command to run (and automatically build) the program :

`cargo run [input file] [output file]`

If no input file is specified, it will read from the standard input instead.  
If no output file is specified, it will output in the standard output instead.

*NB :* `cargo build` can be used to compile, but not run the program

### Input format

The program takes JSON-formatted inputs, according to the following grammar
```
R ::= { "operation" : O, "args" : ARGS }
O ::= "selection" | "projection" | product | "renaming" | "minus" | "union" | "load" | "rspr | "jpr" 
ARGS ::= { "attributes" : ATTS, "object" : R } // for projection
      |  { "condition" : COND, "object" : R } // for selection
      |  { "object1" : R, "object2" : R } // for product, minus and union
      |  { "old attributes" : ATTS, "new attributes" : ATTS, "object" : R } // for renaming
      |  { "filename" : FILENAME } // for load
      |  { "filename" : FILENAME, "condition": COND, "old attributes" : ATTS, "new attributes" : ATTS } // for rspr
      |  { "object1" : R, "object2" : R, "condition" : COND, "old attributes" : ATTS, "new attributes" : ATTS } // for jpr
COND ::= "True" | "False" 
      | { "logical" : "not", "condition" : COND }
      | { "logical" : "and", "condition1" : COND, "condition2" : COND }
      | { "logical" : "or", "condition1" : COND, "condition2" : COND }
      | { "comparator" : COMP, "attribute1" : STRING, "attribute2" : STRING }
COMP ::= "<" | ">" | "="
ATTS ::= STRING list
FILENAME ::= ... // matches \"[A-Za-z\-_0-9]+\.csv\"
```

Examples can be found in the `expr_samples` folder

### Tables

Tables are represented by CSV files, the first one holding the column names, and each subsequent line containing as many values as there are columns.

### Tests

Some tests to demonstrate the correctness of the optimizations are implemented.

They can be run through `cargo tests`.  
They compare the outputs of the execution of some expression and of its optimized version, to check whether we obtain the same table (which should be the case).

### Benchmarks

If tests are there to demonstrate the correctness of the optimizations, benchmarks are there to assess the performance gain.

Benchmarks can be run through `cargo run -- -b`, or `cargo run -- --benchmark`.  
This command processes each benchmark in the `expr_samples/benchmarks` folder, and displays the average execution time over 50 tries.

This means one can easily add requests to the benchmarking by adding a JSON file to the `expr_samples/benchmarks` folder, which should follow the following syntax :
```
{
      "input": { input, formatted according to the main syntax (see above) },
      "tests": [
            {
                  "name": // a string to name this variant,
                  "optims": [
                        // A list of optimizations to apply
                  ]
            },
            ...
      ]
}
```

The idea is that, for each input, we attach a series of tests, to see whether one test performs better than the others.  
A test is composed of a name, used to distinguish it from others; and of a list of optomizations to use.

Each optimization should be one of the following :
* `DLC` : *Detect Load Columns*. Detects the columns that are actually used. Should always be used before *PDS*, as the latter relies on this column detection.
* `PDS` : *Push Down Selection*. Try to push down selections as long as possible.
* `APE` : *Apply Projections Early*. Tries to project as early as possible.
* `FCE` : *Fold Complex Expressions*. Tries to replace parts of the expression by `rspr` or `jpr` constructions.

## On our implementation

### Parsing JSON inputs
This is done using the `serde` crate, along with its JSON parser `serde_json`. Basically, we anotate our type definitions using serde directives in order to specify the bindings between our Rust types and the JSON grammar. Serde then automatically reads the json we feed him and converts it in order to fit the type into which we wish to transform the data.