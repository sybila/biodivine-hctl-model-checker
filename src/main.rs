//! Symbolic HCTL model checker for BN models.
//!
//! Takes an input path to a BN model (format may be specified, aeon format is default) and
//! a path to a set of HCTL formulae, and model-checks these formulae on that model.
//! During (and after) evaluation, prints the selected amount of results (default is just
//! an aggregated information regarding the number of satisfying states/colors).
//!

use biodivine_hctl_model_checker::analysis::analyse_formulae;
use biodivine_hctl_model_checker::result_print::PrintOptions;
use biodivine_lib_param_bn::BooleanNetwork;

use clap::builder::PossibleValuesParser;
use clap::Parser;

use std::fs::read_to_string;
use std::path::Path;

/// Structure to collect CLI arguments
#[derive(Parser)]
#[clap(
    author = "Ondřej Huvar",
    version,
    about = "Symbolic HCTL model checker for Boolean network models."
)]
struct Arguments {
    /// Path to a file with BN model file in one of supported formats.
    model_path: String,

    /// Path to a file with formulae to check.
    formulae_path: String,

    /// Format of the BN model.
    #[clap(short, long, default_value = "aeon", value_parser = PossibleValuesParser::new(["aeon", "sbml", "bnet"]))]
    model_format: String,

    /// Choice of the amount of output regarding computation and results.
    #[clap(short, long, default_value = "short", value_parser = PossibleValuesParser::new(["none", "short", "medium", "full"]))]
    print_option: String,
}

/// Load and parse the BN model in a given format from the specified file.
/// Return error if model is invalid.
fn load_and_parse_bn_model(format: &str, model_path: String) -> Result<BooleanNetwork, String> {
    let model_string = read_to_string(model_path).unwrap();
    return match format {
        "aeon" => BooleanNetwork::try_from(model_string.as_str()),
        "sbml" => Ok(BooleanNetwork::try_from_sbml(model_string.as_str())
            .unwrap()
            .0),
        "bnet" => BooleanNetwork::try_from_bnet(model_string.as_str()),
        // this cant really happen, just here to be exhaustive
        _ => Err("Invalid model format".to_string()),
    };
}

/// Read the formulae from the specified file.
/// The syntax of these formulae is checked later during parsing.
fn load_formulae(formulae_path: String) -> Vec<String> {
    let formulae_string = read_to_string(formulae_path).unwrap();
    let mut formulae: Vec<String> = Vec::new();
    for line in formulae_string.lines() {
        if !line.trim().is_empty() {
            formulae.push(line.trim().to_string());
        }
    }
    formulae
}

/// Wrapper function to invoke the model checker, works with CLI arguments.
fn main() {
    let args = Arguments::parse();

    // check if given paths are valid
    if !Path::new(args.formulae_path.as_str()).is_file() {
        println!("{} is not valid file", args.formulae_path);
        return;
    }
    if !Path::new(args.model_path.as_str()).is_file() {
        println!("{} is not valid file", args.formulae_path);
        return;
    }

    // read the model and formulae
    let formulae = load_formulae(args.formulae_path);
    let maybe_bn = load_and_parse_bn_model(args.model_format.as_str(), args.model_path);
    if maybe_bn.is_err() {
        println!("Model does not have correct format");
        return;
    }
    let bn = maybe_bn.unwrap();

    // compute the results
    let res = match args.print_option.as_str() {
        "none" => analyse_formulae(&bn, formulae, PrintOptions::NoPrint),
        "short" => analyse_formulae(&bn, formulae, PrintOptions::ShortPrint),
        "medium" => analyse_formulae(&bn, formulae, PrintOptions::MediumPrint),
        "full" => analyse_formulae(&bn, formulae, PrintOptions::FullPrint),
        // this cant really happen, just here to be exhaustive
        _ => Err(format!(
            "Wrong print option \"{}\".",
            args.print_option.as_str()
        )),
    };

    if res.is_err() {
        println!("{}", res.err().unwrap());
    }
}
