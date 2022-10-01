use clap::Parser;
use hctl_model_checker::analysis::{analyse_formula, PrintOptions};
use std::fs::read_to_string;

/* TODOs for the general model checking */
// TODO: USE PROPER DUPLICATE MARKING AND IMPLEMENT PROPER CACHE FOR EVALUATOR
// TODO: optimisations for evaluator (changing tree, etc.), maybe few more special cases
// TODO: add documentation, tests
// TODO: refactor tokenizer (remove duplicity, divide functionality, etc)
// TODO: check that formula doesnt contain same var quantified more times - like "!x: (EF (!x: x))
// TODO: check generating predecessors in EU_saturated (check including self-loops)
// TODO: modify aeon SCC computation to not print everything

/* Potential BUGS and issues to fix */
// TODO: parse / tokenize issues
   // "AU !{x}: {x}" is parsed as valid
   // "!{var}: AG EF {var} & & !{var}: AG EF {var}" is parsed as valid

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

#[cfg(test)]
mod tests {
    use biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph;
    use biodivine_lib_param_bn::BooleanNetwork;
    use hctl_model_checker::analysis::model_check_formula_unsafe;

    // model FISSION-YEAST-2008
    const BNET_MODEL: &str = r"
targets,factors
Cdc25, ((!Cdc2_Cdc13 & (Cdc25 & !PP)) | ((Cdc2_Cdc13 & (!Cdc25 & !PP)) | (Cdc2_Cdc13 & Cdc25)))
Cdc2_Cdc13, (!Ste9 & (!Rum1 & !Slp1))
Cdc2_Cdc13_A, (!Ste9 & (!Rum1 & (!Slp1 & (!Wee1_Mik1 & Cdc25))))
PP, Slp1
Rum1, ((!SK & (!Cdc2_Cdc13 & (!Rum1 & (!Cdc2_Cdc13_A & PP)))) | ((!SK & (!Cdc2_Cdc13 & (Rum1 & !Cdc2_Cdc13_A))) | ((!SK & (!Cdc2_Cdc13 & (Rum1 & (Cdc2_Cdc13_A & PP)))) | ((!SK & (Cdc2_Cdc13 & (Rum1 & (!Cdc2_Cdc13_A & PP)))) | (SK & (!Cdc2_Cdc13 & (Rum1 & (!Cdc2_Cdc13_A & PP))))))))
SK, Start
Slp1, Cdc2_Cdc13_A
Start, 0
Ste9, ((!SK & (!Cdc2_Cdc13 & (!Ste9 & (!Cdc2_Cdc13_A & PP)))) | ((!SK & (!Cdc2_Cdc13 & (Ste9 & !Cdc2_Cdc13_A))) | ((!SK & (!Cdc2_Cdc13 & (Ste9 & (Cdc2_Cdc13_A & PP)))) | ((!SK & (Cdc2_Cdc13 & (Ste9 & (!Cdc2_Cdc13_A & PP)))) | (SK & (!Cdc2_Cdc13 & (Ste9 & (!Cdc2_Cdc13_A & PP))))))))
Wee1_Mik1, ((!Cdc2_Cdc13 & (!Wee1_Mik1 & PP)) | ((!Cdc2_Cdc13 & Wee1_Mik1) | (Cdc2_Cdc13 & (Wee1_Mik1 & PP))))
";

    #[test]
    /// Test evaluation of several important formulae on model FISSION-YEAST-2008
    /// Compare numbers of results with the numbers acquired by Python model checker or AEON
    fn test_model_check_basic_formulae() {
        let bn = BooleanNetwork::try_from_bnet(BNET_MODEL).unwrap();
        // test formulae use 3 HCTL vars at most
        let stg = SymbolicAsyncGraph::new(bn, 3).unwrap();

        let mut result = model_check_formula_unsafe("!{x}: AG EF {x}".to_string(), &stg);
        assert_eq!(76., result.approx_cardinality());
        assert_eq!(2., result.colors().approx_cardinality());
        assert_eq!(76., result.vertices().approx_cardinality());

        result = model_check_formula_unsafe("!{x}: AX {x}".to_string(), &stg);
        assert_eq!(12., result.approx_cardinality());
        assert_eq!(1., result.colors().approx_cardinality());
        assert_eq!(12., result.vertices().approx_cardinality());

        result = model_check_formula_unsafe("!{x}: AX EF {x}".to_string(), &stg);
        assert_eq!(132., result.approx_cardinality());
        assert_eq!(2., result.colors().approx_cardinality());
        assert_eq!(132., result.vertices().approx_cardinality());

        result = model_check_formula_unsafe("AF (!{x}: AX {x})".to_string(), &stg);
        assert_eq!(60., result.approx_cardinality());
        assert_eq!(1., result.colors().approx_cardinality());
        assert_eq!(60., result.vertices().approx_cardinality());

        result = model_check_formula_unsafe(
            "!{x}: 3{y}: (@{x}: ~{y} & AX {x}) & (@{y}: AX {y})".to_string(),
            &stg,
        );
        assert_eq!(12., result.approx_cardinality());
        assert_eq!(1., result.colors().approx_cardinality());
        assert_eq!(12., result.vertices().approx_cardinality());

        result = model_check_formula_unsafe(
            "3{x}: 3{y}: (@{x}: ~{y} & AX {x}) & (@{y}: AX {y}) & EF ({x} & (!{z}: AX {z})) & EF ({y} & (!{z}: AX {z})) & AX (EF ({x} & (!{z}: AX {z})) ^ EF ({y} & (!{z}: AX {z})))".to_string(),
            &stg,
        );
        assert_eq!(11., result.approx_cardinality());
        assert_eq!(1., result.colors().approx_cardinality());
        assert_eq!(11., result.vertices().approx_cardinality());

        result = model_check_formula_unsafe("!{x}: (AX (AF {x}))".to_string(), &stg);
        assert_eq!(12., result.approx_cardinality());
        assert_eq!(1., result.colors().approx_cardinality());
        assert_eq!(12., result.vertices().approx_cardinality());

        result = model_check_formula_unsafe("AF (!{x}: (AX (~{x} & AF {x})))".to_string(), &stg);
        assert_eq!(0., result.approx_cardinality());
        assert_eq!(0., result.colors().approx_cardinality());
        assert_eq!(0., result.vertices().approx_cardinality());

        result = model_check_formula_unsafe(
            "AF (!{x}: ((AX (~{x} & AF {x})) & (EF (!{y}: EX ~AF {y}))))".to_string(),
            &stg,
        );
        assert_eq!(0., result.approx_cardinality());
        assert_eq!(0., result.colors().approx_cardinality());
        assert_eq!(0., result.vertices().approx_cardinality());
    }
}
