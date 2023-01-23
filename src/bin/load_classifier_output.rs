//! Binary for testing the classifier output loading.

use biodivine_hctl_model_checker::model_checking::get_extended_symbolic_graph;
use biodivine_lib_bdd::Bdd;
use biodivine_lib_param_bn::symbolic_async_graph::GraphColors;
use biodivine_lib_param_bn::BooleanNetwork;

use clap::Parser;
use std::fs::{read_to_string, File};
use std::io::Read;
use zip::ZipArchive;

/// Structure to collect CLI arguments
#[derive(Parser)]
#[clap()]
struct Arguments {
    /// Path to a file with BN model file in one of supported formats.
    model_path: String,

    /// Path to an existing zip archive with report and files with BDDs.
    results_archive: String,
}

/// Collect the results of classification, which are BDDs representing color sets.
///
/// Each BDD is dumped in a file in `results_archive`. Moreover, excluding these BDD files, the dir
/// contains a report and a metadata file. Metadata file contains information regarding the number
/// of extended symbolic HCTL variables supported by the BDDs.
///
/// The file at `model_path` contains the original parametrized model that was used for the
/// classification.
pub fn load_classifier_output(
    results_archive: &str,
    model_path: &str,
) -> Vec<(String, GraphColors)> {
    // open the zip archive with results
    let archive_file = File::open(results_archive).unwrap();
    let mut archive = ZipArchive::new(archive_file).unwrap();

    // load number of HCTL variables from computation metadata
    let mut metadata_file = archive.by_name("metadata.txt").unwrap();
    let mut buffer = String::new();
    metadata_file.read_to_string(&mut buffer).unwrap();
    let num_hctl_vars: u16 = buffer.parse::<u16>().unwrap();
    drop(metadata_file);

    // load the BN model and generate extended symbolic graph
    let model_string = read_to_string(model_path).unwrap();
    let bn = BooleanNetwork::try_from(model_string.as_str()).unwrap();
    let graph = get_extended_symbolic_graph(&bn, num_hctl_vars);

    // collect the colored sets from the BDD dumps together with their "names"
    let mut named_color_sets = Vec::new();

    // iterate over all files by indices
    // expects only BDD dumps (individual files) and a report&metadata in the archive (for now)
    for idx in 0..archive.len() {
        let mut entry = archive.by_index(idx).unwrap();
        let file_name = entry.name().to_string();
        if file_name == *"report.txt" || file_name == *"metadata.txt" {
            continue;
        }

        // read the raw BDD
        let mut bdd_str = String::new();
        entry.read_to_string(&mut bdd_str).unwrap();
        let bdd = Bdd::from_string(bdd_str.as_str());

        let color_set = GraphColors::new(bdd, graph.symbolic_context());
        named_color_sets.push((file_name, color_set));
    }

    named_color_sets
}

/// Wrapper function to invoke the loading process by feeding it with CLI arguments.
fn main() {
    let args = Arguments::parse();

    // load the color sets that represent the classification results
    let color_sets =
        load_classifier_output(args.results_archive.as_str(), args.model_path.as_str());

    for (name, color_set) in color_sets {
        println!("{}: {}", name, color_set.exact_cardinality());
    }
}
