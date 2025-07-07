#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! clap = { version = "4.5.40", features = ["derive"] }
//! monitor = { path = "../monitor" }
//! ```

use std::path::PathBuf;
use std::fs;
use clap::Parser;

/// Utility that parses an input JSON file into a serialized Rust DFA object, optionally copying it over to a
/// specified output directory
#[derive(Parser, Debug)]
struct Args {
    /// JSON DFA file to parse from
    #[arg(value_name = "FILE")]
    input_file: PathBuf,

    /// Optional output directory
    #[arg(short, long, value_name = "DIR")]
    output_dir: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();
    let dfa = monitor::Dfa::deserialize_from_json(args.input_file);
    let saved_path = dfa.serialize();

    if let Some(out_dir) = args.output_dir {
        assert!(fs::metadata(&out_dir).unwrap().is_dir(), "Specified output file is not a directory");
        let dfa_saved_file_name = &saved_path.file_name().unwrap();
        let new_file = out_dir.join(dfa_saved_file_name);
        fs::copy(saved_path, &new_file).expect("Copying serialized DFA to target directory failed");
        println!("{:?}", new_file);
    } else {
        println!("{:?}", saved_path);
    }
}