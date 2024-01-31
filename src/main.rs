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

/// Structure to collect CLI arguments
#[derive(Parser, Debug)]
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

    /// Path to the zip with resulting BDD dumps. If not specified, only selected summary is printed.
    #[clap(short, long)]
    output_bundle: Option<String>,

    /// Choice of the amount of output regarding computation and results.
    /// Default is just an aggregated information regarding the number of satisfying states/colors
    #[clap(short, long, default_value = "summary", value_parser = PossibleValuesParser::new(["no-print", "summary", "with-progress", "exhaustive"]))]
    print_option: String,

    /// Model-check extended formula (that may contain wild-card propositions and variable domains) by providing
    /// a path to zip bundle of BDDs specifying context of wild-cards.
    #[clap(short, long)]
    extended_context: Option<String>,
}

/// Wrapper function to invoke the model checker, works with CLI arguments.
fn main() {
    let args = Arguments::parse();

    // read the BN model
    let maybe_bn = BooleanNetwork::try_from_file(args.model_path.as_str());
    if maybe_bn.is_err() {
        println!("Model is corrupted or does not have any supported format.");
        println!("{}", maybe_bn.err().unwrap());
        return;
    }
    let bn = maybe_bn.unwrap();

    // read the formulae
    let maybe_formulae = load_formulae(args.formulae_path.as_str());
    if maybe_formulae.is_err() {
        println!("Formulae file is corrupted or does not have the supported format.");
        println!("{}", maybe_formulae.err().unwrap());
        return;
    }
    let formulae = maybe_formulae.unwrap();

    // compute the results
    let print_option = match args.print_option.as_str() {
        "no-print" => PrintOptions::NoPrint,
        "summary" => PrintOptions::JustSummary,
        "with-progress" => PrintOptions::WithProgress,
        "exhaustive" => PrintOptions::Exhaustive,
        // this cant really happen (would cause error earlier), just here to have exhaustive match
        _ => panic!(
            "{}",
            format!("Wrong print option \"{}\".", args.print_option.as_str())
        ),
    };

    let res = analyse_formulae(
        &bn,
        formulae,
        print_option,
        args.output_bundle,
        args.extended_context,
    );

    if res.is_err() {
        println!("{}", res.err().unwrap());
    }
}
