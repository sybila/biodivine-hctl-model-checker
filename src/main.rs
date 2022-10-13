use biodivine_hctl_model_checker::analysis::{analyse_formula, PrintOptions};
use clap::Parser;
use std::fs::read_to_string;


/* TODOs */
// TODO: USE PROPER DUPLICATE MARKING AND IMPLEMENT PROPER CACHE FOR EVALUATOR
// TODO: optimisations for evaluator (changing tree, etc.), maybe few more special cases
// TODO: add check that formula doesnt contain same var quantified more times - like "!x: (EF (!x: x))
// TODO: add check that formula doesnt contain free vars (during parsing or var collecting)
// TODO: check generating predecessors in EU_saturated (check including self-loops)


/// Structure to collect CLI arguments
#[derive(Parser)]
#[clap(
    author = "Ondrej Huvar",
    version,
    about = "Symbolic HCTL model checker for Boolean networks"
)]
struct Arguments {
    /// Path to the file with BN model in aeon format
    model_path: String,
    /// Formula to check
    formula: String,
    /// Choice of output mode: none/short/full
    #[clap(short, long, default_value = "short")]
    print_option: String,
}

fn main() {
    let args = Arguments::parse();
    let aeon_string = read_to_string(args.model_path).unwrap();
    println!("original formula: {}", args.formula);

    match args.print_option.as_str() {
        "none" => analyse_formula(aeon_string, args.formula, PrintOptions::NoPrint),
        "short" => analyse_formula(aeon_string, args.formula, PrintOptions::ShortPrint),
        "full" => analyse_formula(aeon_string, args.formula, PrintOptions::LongPrint),
        _ => println!("Wrong print option \"{}\".", args.print_option.as_str()),
    }
}
