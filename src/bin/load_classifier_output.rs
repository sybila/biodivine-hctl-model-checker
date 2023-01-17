//! Binary for testing the classifier output loading.

use biodivine_hctl_model_checker::bn_classification::load_classifier_output;

use clap::Parser;

/// Structure to collect CLI arguments
#[derive(Parser)]
#[clap()]
struct Arguments {
    /// Path to a file with BN model file in one of supported formats.
    model_path: String,

    /// Path to an existing directory with report and dumped BDDs.
    results_dir: String,
}

/// Wrapper function to invoke the loading process and feed it with CLI arguments.
fn main() {
    let args = Arguments::parse();

    // load the color sets that represent the classification results
    let color_sets = load_classifier_output(args.results_dir.as_str(), args.model_path.as_str());

    for (name, color_set) in color_sets {
        println!("{}: {}", name, color_set.exact_cardinality());
    }
}
