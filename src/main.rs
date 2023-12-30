//! Symbolic HCTL model checker for BN models.
//!
//! Takes an input path to a BN model and a path to a set of HCTL formulae, and runs a model-checking analysis.
//! During (and after) evaluation, prints the selected amount of results (default is just
//! an aggregated information regarding the number of satisfying states/colors).
//!

use biodivine_hctl_model_checker::analysis::analyse_formulae;
use biodivine_hctl_model_checker::load_inputs::load_formulae;
use biodivine_hctl_model_checker::result_print::PrintOptions;

use clap::builder::PossibleValuesParser;
use clap::Parser;

use biodivine_lib_param_bn::BooleanNetwork;
use std::path::Path;

/// Structure to collect CLI arguments
#[derive(Parser)]
#[clap(
    author = "Ondřej Huvar",
    version,
    about = "Symbolic HCTL model checker for Boolean network models."
)]
struct Arguments {
    /// Path to a file with BN model file in one of supported formats (aeon, sbml, bnet).
    model_path: String,

    /// Path to a file with formulae to check.
    formulae_path: String,

    /// Choice of the amount of output regarding computation and results.
    /// Default is just an aggregated information regarding the number of satisfying states/colors
    #[clap(short, long, default_value = "summary", value_parser = PossibleValuesParser::new(["no-print", "summary", "with-progress", "exhaustive"]))]
    print_option: String,

    /// Model-check extended formula (that may contain wild-card propositions and variable domains) by providing
    /// a path to zip bundle of BDDs specifying context of wild-cards.
    #[clap(short, long)]
    extended_context: Option<String>,

    /// Path to the zip with resulting BDD dumps. If not specified, only selected summary is printed.
    #[clap(short, long)]
    output_bundle: Option<String>,
}

/// Wrapper function to invoke the model checker, works with CLI arguments.
fn main() {
    // TODO: utilize the extended context

    let args = Arguments::parse();

    // check if given paths are valid
    if !Path::new(args.formulae_path.as_str()).is_file() {
        println!("{} is not valid file", args.formulae_path);
        return;
    }
    if !Path::new(args.model_path.as_str()).is_file() {
        println!("{} is not valid file", args.model_path);
        return;
    }
    if let Some(extended_context) = args.extended_context {
        if !Path::new(extended_context.as_str()).is_file() {
            println!("{extended_context} is not valid file");
            return;
        }
    }

    // read the model and formulae
    let formulae = load_formulae(args.formulae_path.as_str());
    let maybe_bn = BooleanNetwork::try_from_file(args.model_path.as_str());
    if maybe_bn.is_err() {
        println!("Model is incorrect or does not have any supported format.");
        println!("{}", maybe_bn.err().unwrap());
        return;
    }
    let bn = maybe_bn.unwrap();

    // compute the results
    let res = match args.print_option.as_str() {
        "no-print" => analyse_formulae(&bn, formulae, PrintOptions::NoPrint, args.output_bundle),
        "summary" => analyse_formulae(&bn, formulae, PrintOptions::JustSummary, args.output_bundle),
        "with-progress" => analyse_formulae(
            &bn,
            formulae,
            PrintOptions::WithProgress,
            args.output_bundle,
        ),
        "exhaustive" => {
            analyse_formulae(&bn, formulae, PrintOptions::Exhaustive, args.output_bundle)
        }
        // this cant really happen (would cause error earlier), just here to have exhaustive match
        _ => Err(format!(
            "Wrong print option \"{}\".",
            args.print_option.as_str()
        )),
    };

    if res.is_err() {
        println!("{}", res.err().unwrap());
    }
}
