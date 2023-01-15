use biodivine_lib_param_bn::symbolic_async_graph::GraphColors;
use biodivine_lib_param_bn::BooleanNetwork;

use biodivine_hctl_model_checker::model_checking::get_extended_symbolic_graph;

use clap::Parser;

use std::fs::{read_dir, read_to_string, File};
use std::path::PathBuf;

/// Structure to collect CLI arguments
#[derive(Parser)]
#[clap()]
struct Arguments {
    /// Path to a file with BN model file in one of supported formats.
    model_path: String,

    /// Path to an existing directory with report and dumped BDDs.
    results_dir: String,
}

fn main() {
    let args = Arguments::parse();
    let results_dir = args.results_dir.as_str();

    // load number of HCTL variables from computation metadata
    let metadata_file_path = PathBuf::from(results_dir).join("metadata.txt");
    let num_hctl_vars: u16 = read_to_string(metadata_file_path)
        .unwrap()
        .parse::<u16>()
        .unwrap();

    let model_string = read_to_string(args.model_path).unwrap();
    let bn = BooleanNetwork::try_from(model_string.as_str()).unwrap();
    let graph = get_extended_symbolic_graph(&bn, num_hctl_vars);

    let files = read_dir(results_dir).unwrap();

    // expects only BDD dumps (individual files) and a report&metadata in the directory (for now)
    for file in files {
        let path = file.unwrap().path().clone();
        let file_name = path.file_name().unwrap().to_str().unwrap();
        if file_name == "report.txt" || file_name == "metadata.txt" {
            continue;
        }
        let mut file = File::open(path).unwrap();
        let bdd = biodivine_lib_bdd::Bdd::read_as_string(&mut file).unwrap();

        let color_set = GraphColors::new(bdd, graph.symbolic_context());
        println!("{}", color_set.exact_cardinality());
    }
}
