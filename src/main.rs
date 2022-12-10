use biodivine_hctl_model_checker::analysis::analyse_formulae;
use biodivine_hctl_model_checker::result_print::PrintOptions;
use biodivine_lib_param_bn::BooleanNetwork;

use clap::builder::PossibleValuesParser;
use clap::Parser;

use std::fs::read_to_string;
use std::path::Path;


/* TODOs */
// TODO: optimisations for evaluator (modifying tree, etc.), maybe few more special cases
// TODO: check generating predecessors in EU_saturated (check including self-loops)
// TODO: more efficient fixed-point computation


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

    /// Path to the file with formulae to check
    formulae_path: String,

    /// Model format
    #[clap(short, long, default_value = "aeon", value_parser = PossibleValuesParser::new(["aeon", "sbml", "bnet"]))]
    model_format: String,

    /// Choice of the amount of output regarding computation and results
    #[clap(short, long, default_value = "short", value_parser = PossibleValuesParser::new(["none", "short", "medium", "full"]))]
    print_option: String,

    /// Report the results also to the given file (if none, only the CLI output is given)
    #[clap(short, long, default_value = "None")]
    report_file: String,
}

fn load_bn_model(format: &str, model_path: String) -> Result<BooleanNetwork, String> {
    let model_string = read_to_string(model_path).unwrap();
    return match format {
        "aeon" => BooleanNetwork::try_from(model_string.as_str()),
        "sbml" => Ok(BooleanNetwork::try_from_sbml(model_string.as_str()).unwrap().0),
        "bnet" => BooleanNetwork::try_from_bnet(model_string.as_str()),
        // this cant really happen, just here to be exhaustive
        _ => Err("Invalid model format".to_string()),
    };
}

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
    let maybe_bn = load_bn_model(args.model_format.as_str(), args.model_path);
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
        _ => Err(format!("Wrong print option \"{}\".", args.print_option.as_str())),
    };

    if res.is_err() {
        println!("{}", res.err().unwrap());
    }
}
