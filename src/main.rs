use biodivine_hctl_model_checker::analysis::{analyse_formula, PrintOptions};
use biodivine_lib_param_bn::BooleanNetwork;

use clap::builder::PossibleValuesParser;
use clap::Parser;

use std::fs::read_to_string;


/* TODOs */
// TODO: optimisations for evaluator (changing tree, etc.), maybe few more special cases
// TODO: add check that formula doesnt contain same var quantified more times - like "!x: (EF (!x: x))
// TODO: add check that formula doesnt contain free vars (during parsing or var collecting)
// TODO: check generating predecessors in EU_saturated (check including self-loops)
// TODO: create general function combining functionality of "model_check" and "model_check_unsafe", and make new version of "model_check_unsafe" way more hacky


/// Structure to collect CLI arguments
#[derive(Parser)]
#[clap(
    author = "Ondrej Huvar",
    version,
    about = "Symbolic HCTL model checker for Boolean network models"
)]
struct Arguments {
    /// Path to the file with BN model file in one of readable formats
    model_path: String,

    /// Formula to check
    formula: String,

    /// Model format
    #[clap(short, long, default_value = "aeon", value_parser = PossibleValuesParser::new(["aeon", "sbml", "bnet"]))]
    model_format: String,

    /// Choice of output mode for results
    #[clap(short, long, default_value = "short", value_parser = PossibleValuesParser::new(["none", "short", "full"]))]
    print_option: String,
}

fn parse_bn_model(format: &str, model_string: &str) -> Result<BooleanNetwork, String> {
    return match format {
        "aeon" => BooleanNetwork::try_from(model_string),
        "sbml" => Ok(BooleanNetwork::try_from_sbml(model_string).unwrap().0),
        "bnet" => BooleanNetwork::try_from_bnet(model_string),
        // this cant really happen, just here to be exhaustive
        _ => Err("Invalid model format".to_string()),
    };
}

fn main() {
    let args = Arguments::parse();

    if args.print_option.as_str() != "none" {
        println!("original formula: {}", args.formula);
    }

    let model_string = read_to_string(args.model_path).unwrap();
    let bn = parse_bn_model(args.model_format.as_str(), model_string.as_str()).unwrap();

    match args.print_option.as_str() {
        "none" => analyse_formula(bn, args.formula, PrintOptions::NoPrint),
        "short" => analyse_formula(bn, args.formula, PrintOptions::ShortPrint),
        "full" => analyse_formula(bn, args.formula, PrintOptions::LongPrint),
        // this cant really happen, just here to be exhaustive
        _ => println!("Wrong print option \"{}\".", args.print_option.as_str()),
    }
}
