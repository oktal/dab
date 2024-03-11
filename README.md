<div align="center">
  <a href="https://github.com/oktal/dab">
    <img src="assets/logo.png" alt="Logo" width="80" height="80">
  </a>

<h3 align="center">dab</h3>

</div>

# About the project

dab stands for `Distributeur Automatique de Billet` which is the french translation of `Automated Teller Machine` ATM

This is a toy project that simulates a payment engine that reads transactions from a CSV
file and outputs a resulting CSV

# Getting started

To run the project, first make sure to have [Rust](https://rustup.sh) installed on your environment.

Use `cargo run` to run the project with a valid CSV transaction file as an input to the program

```
cargo run --release -- transactions.csv
```

# Design principles

## Input dataset

While being a toy project, the underlying goal is to make sure that `dab` can handle small
and very large datasets

The `input` module provides the main abstraction to read the data from a file that represents
transactions. In this example, transactions come from a CSV file. However, as a serious data engineer, we might decide in a near future to handle very large datasets through more compact and efficient formats like Apache [arrow](https://arrow.apache.org/) and [parquet](https://parquet.apache.org/)

The `input::Reader` trait abstraction should make it easy to extend and experiment with multiple
input formats

For very large datasets, loading the whole dataset in memory is not an option as the dataset size
might exceed the total memory size or we might be constrained by the total available memory size
when running in cloud environnments such as AWS (either by the size of the allocated instances or by budget constraint)

Thus, the `input::Reader` exposes an `Iterator` abstraction to be as lazy as possible when processing data and be gentle on memory usage

## Correctness

To validate the correctness of the data, we leverage the Rust type-system by making it impossible for a transaction that hold invalid state.

For example, `Dispute` and `Resolve` transactions do not have an associated amount. While
we could represent the type of a transaction by a simple `enum` and have an associated `Option<f64>` with `None` value for transactions that do not have an associated amount,
the risk of failing to handle the amount properly has been judged too high.

This is why the main `Transaction` model is represented as an enum with fields that are only active depending on the type of the transaction