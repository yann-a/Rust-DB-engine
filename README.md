# DBDM Project

A relational algebra engine that operates on CSV files, written in Rust.

**Authors**:
* Guilhem Niot
* Yann Aguettaz

## Requirements

The project should run on all proposed environements. It is only tested for Ubuntu and Arch Linux though.

In order to manage dependencies easily, we use `cargo`, which is Rust's package manager.

To use the project, you will thus need to install Rust and Cargo.
On Unix-like systems, this can be done via `rustup`, by running `curl https://sh.rustup.rs -sSf | sh`.

### Dependencies

The project relies on the following crates (the equivalent of libraries in Rust) :
* `csv` to manage csv files
* `serde` to parse inputs
* `serde_derive` a addon of serde to automatically derive a grammar from a type
* `serde_json` a addon of serde to support JSON
* `clap` to handle command-line arguments

Singe Cargo manages them automatically, they do not require any kind of manual installation.

## Usage

Use the following command to run (and automatically build) the program :

`cargo run [input file] [output file]`

If no input file is specified, it will read from the standard input instead.  
If no output file is specified, it will output in the standard output instead.

*NB :* `cargo build` can be used to compile, but not run the program

### Other commands

* `cargo tests` runs some tests on the optimizations. These compare the outputs of the source and optimized version, to ensure that they are equal.
* `cargo run -- -b` or `cargo run -- --benchmark` runs the benchmarks