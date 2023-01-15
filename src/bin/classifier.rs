//! Symbolic classifier for BN models based on dynamic properties.
//!
//! Takes an input path to a partially defined BN model and a 2 paths to sets of HCTL formulae.
//! These are 2 kinds of formulae - assertions that must be satisfied, and properties that are
//! used for classification.
//!
//! First, conjunction of assertions is model-checked, and then the set of remaining colors is
//! decomposed into categories based on properties.
//!

use biodivine_hctl_model_checker::bn_classification::classify;
use clap::Parser;
use std::path::Path;

/// Structure to collect CLI arguments
#[derive(Parser)]
#[clap(
    author = "Ondřej Huvar",
    version,
    about = "Symbolic classifier for BN models based on dynamic properties."
)]
struct Arguments {
    /// Path to a file with BN model file in one of supported formats.
    model_path: String,

    /// Path to a file with assertion formulae that must be satisfied.
    assertion_formulae_path: String,

    /// Path to a file with property formulae that are used for classification.
    property_formulae_path: String,

    /// Path to an existing directory to which report and BDD results will be dumped.
    output_dir: String,
}

/// Wrapper function to invoke the classifier and feed it with CLI arguments.
fn main() {
    let args = Arguments::parse();
    println!("Loading input files...");

    let assertion_formulae_path = args.assertion_formulae_path;
    let property_formulae_path = args.property_formulae_path;
    let model_path = args.model_path;
    let output_dir = args.output_dir;

    // check if given paths are valid
    if !Path::new(assertion_formulae_path.as_str()).is_file() {
        println!("{} is not valid file", assertion_formulae_path);
        return;
    }
    if !Path::new(property_formulae_path.as_str()).is_file() {
        println!("{} is not valid file", property_formulae_path);
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
        assertion_formulae_path.as_str(),
        property_formulae_path.as_str(),
    );

    if classification_res.is_err() {
        println!(
            "Error during computation: {}",
            classification_res.err().unwrap()
        )
    }
}
