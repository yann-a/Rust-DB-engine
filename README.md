# DBDM Project

A relational algebra engine that operates on CSV files, written in Rust.

## General Informations
**Authors**:
* Guilhem Niot
* Yann Aguettaz

### Requirements

The project should run on all proposed environements. It is only tested for Ubuntu and Arch Linux though.

In order to manage dependencies easily, we use `cargo`, which is Rust's package manager.

To use the project, you will thus need to install Rust and Cargo.
On Unix-like systems, this can be done via `rustup`, by running `curl https://sh.rustup.rs -sSf | sh`.

#### Dependencies

The project relies on the following crates (the equivalent of libraries in Rust) :
* `csv` to manage csv files
* `serde` to parse inputs
* `serde_derive` a addon of serde to automatically derive a grammar from a type
* `serde_json` a addon of serde to support JSON
* `clap` to handle command-line arguments

Singe Cargo manages them automatically, they do not require any kind of manual installation.

### Usage

`cargo run [input file] [output file]`

Use `cargo run` to (build and) run the project. 
* If you don't specify an input file, it will try to read from the standard input instead.
* If you don't specify an output file, it will display the output on the standard output instead.

Note that you can only specify an output file if you provided an input one.

#### Benchmarking

Alternatively, use `cargo build -- --benchmark` to run the benchmarking script

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

## On our implementation

### Parsing JSON inputs
This is done using the `serde` crate, along with its JSON parser `serde_json`. Basically, we anotate our type definitions using serde directives in order to specify the bindings between our Rust types and the JSON grammar. Serde then automatically reads the json we feed him and converts it in order to fit the type into which we wish to transform the data.

### Benchmarking
The benchmark can be run using `cargo run -- -b` or `cargo run -- --benchmark`. It runs all benchmarks in the `tests/benchmarks` folder.

A benchmark is specified in JSON format, with the following format :
```
{
      "input": { input, formatted as per a regular execution },
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

Optimizations are to be chosen in the following list :
* **DLC** : 
* **PDS** : 
* **APE** : 
* **FCE** :