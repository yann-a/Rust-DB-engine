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