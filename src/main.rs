use hctl_model_checker::analysis::{analyse_formula, PrintOptions};
use std::env;
use std::fs::read_to_string;

/* TODOs to implement for the model checking part */
// TODO: USE PROPER DUPLICATE MARKING AND IMPLEMENT PROPER CACHE FOR EVALUATOR
// TODO: optimisations for evaluator, maybe few more special cases
// TODO: think of some equivalent method to saturation for EG,AU ?
// TODO: more efficient operators on GraphColoredVertices (like imp, xor, equiv)?
// TODO: printer for all correct valuations in all three color/vertex sets
// TODO: documentation

/* BUGs and issues to fix */
// TODO: does formula 4 from TACAS and CAV work?
// TODO: is parsing and operator priority right? - probably ok, just needs right parentheses
/*
   AF !{x}: (AX (~{x} & AF {x})) parses as (Bind {x}: (Ax ((~ {x}) & (Af {x}))))
   3{x}: ({x} & !{y}: {y}) parses as (Exist {x}: (Bind {y}: {y}))
   "!{var}: AG EF {var} & !{var}: AG EF {var}" does not parse "correctly"
*/

// TODO: "!{var}: AG EF {var} & & !{var}: AG EF {var}" DOES NOT CAUSE ERROR
// TODO: check that formula doesnt contain stuff like "!x (EF (!x x)) - same var quantified more times

/* TODOs to implement for the inference part */
// TODO: implement "model checking with macros" - one element in the formula will be a placeholder for some precomputed value
// TODO: parse attractors from binarixed 1/0 data
// TODO: optimisation for conjunction computing - restrict colors for the second conjunct if possible

// TODO: printing satisfying BNs? or do something with the resulting colors

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("2 arguments expected, got {}", args.len() - 1);
        println!("Usage: ./hctl-model-checker model_file formula");
        return;
    }
    let aeon_string = read_to_string(args[1].clone()).unwrap();
    println!("original formula: {}", args[2].clone());
    analyse_formula(aeon_string, args[2].clone(), PrintOptions::LongPrint);
}

#[cfg(test)]
mod tests {
    use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
    use biodivine_lib_param_bn::BooleanNetwork;
    use hctl_model_checker::analysis::model_check_formula;

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
    fn basic_formulas() {

        let bn = BooleanNetwork::try_from_bnet(BNET_MODEL).unwrap();
        let stg = SymbolicAsyncGraph::new(bn).unwrap();

        let mut result = model_check_formula("!{x}: AG EF {x}".to_string(), &stg);
        assert_eq!(76., result.approx_cardinality());
        assert_eq!(2., result.colors().approx_cardinality());
        assert_eq!(76., result.vertices().approx_cardinality());

        result = model_check_formula("!{x}: AX {x}".to_string(), &stg);
        assert_eq!(12., result.approx_cardinality());
        assert_eq!(1., result.colors().approx_cardinality());
        assert_eq!(12., result.vertices().approx_cardinality());

        result = model_check_formula("!{x}: AX EF {x}".to_string(), &stg);
        assert_eq!(132., result.approx_cardinality());
        assert_eq!(2., result.colors().approx_cardinality());
        assert_eq!(132., result.vertices().approx_cardinality());

        result = model_check_formula("AF (!{x}: AX {x})".to_string(), &stg);
        assert_eq!(60., result.approx_cardinality());
        assert_eq!(1., result.colors().approx_cardinality());
        assert_eq!(60., result.vertices().approx_cardinality());

        result = model_check_formula(
            "!{x}: 3{y}: (@{x}: ~{y} & AX {x}) & (@{y}: AX {y})".to_string(),
            &stg,
        );
        assert_eq!(12., result.approx_cardinality());
        assert_eq!(1., result.colors().approx_cardinality());
        assert_eq!(12., result.vertices().approx_cardinality());
    }
}
