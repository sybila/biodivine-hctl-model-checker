//! Symbolic classifier for BN models based on dynamic properties.
//!
//! Takes an input path to a partially defined BN model and a path to HCTL formulae.
//! There are 2 kinds of formulae - assertions that must be satisfied, and properties that are
//! used for classification (in the file divided by delimiter line `+++`).
//!
//! First, conjunction of assertions is model-checked, and then the set of remaining colors is
//! decomposed into categories based on properties.
//!

use biodivine_hctl_model_checker::bn_classification::classify;
use clap::Parser;
use std::path::Path;

/// Structure to collect CLI arguments
#[derive(Parser)]
#[clap(about = "Symbolic classifier for BN models based on dynamic properties.")]
struct Arguments {
    /// Path to a file with BN model file in one of supported formats.
    model_path: String,

    /// Path to a file with assertion and property formulae.
    formulae_path: String,

    /// Path to an existing directory to which report and BDD results will be dumped.
    output_dir: String,
}

/// Wrapper function to invoke the classifier and feed it with CLI arguments.
fn main() {
    let args = Arguments::parse();
    println!("Loading input files...");

    let formulae_path = args.formulae_path;
    let model_path = args.model_path;
    let output_dir = args.output_dir;

    // check if given paths are valid
    if !Path::new(formulae_path.as_str()).is_file() {
        println!("{} is not valid file", formulae_path);
        return;
    }
    if !Path::new(model_path.as_str()).is_file() {
        println!("{} is not valid file", model_path);
        return;
    }
    if !Path::new(output_dir.as_str()).is_dir() {
        println!("{} is not valid directory", output_dir);
        return;
    }

    let classification_res = classify(
        output_dir.as_str(),
        model_path.as_str(),
        formulae_path.as_str(),
    );

    if classification_res.is_err() {
        println!(
            "Error during computation: {}",
            classification_res.err().unwrap()
        )
    }
}
