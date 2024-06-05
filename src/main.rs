use duckdb::arrow::util::pretty::print_batches;
use duckdb::{params, Connection, Result};

use std::fs::read_to_string;

// TODO use the more efficent method described in the rust docs
fn read_lines(filename: &str) -> Vec<String> {
    let mut result = Vec::new();
    for line in read_to_string(filename).unwrap().lines() {
        result.push(line.to_string());
    }
    result
}

/// Function responsible for buidling the database
fn build_database() -> Result<()> {
    let lines = read_lines("test_file.txt");
    for line in lines {
        println!("{}", line);
    }

    let conn = Connection::open_in_memory()?;
    let mut stmt = conn.prepare("SELECT * from test_file.txt");
    Ok(())
}

/// Function responsible for preprocessing the file and loading it into the duckdb database
fn preprocess_file() {
    // TODO: Wrap in timing metrics.
    // Does rust have something built in for that?
    // Does rust have a built in profiler / tracer as well?
    let _ = build_database();
}

/// Function responsible for getting a specific line from the database
fn get_line() {}

fn main() {
    preprocess_file();
    // println!("Hello, world!");
}
