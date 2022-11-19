use crate::formula_evaluation::algorithm::{eval_minimized_tree, eval_minimized_tree_unsafe_ex};
use crate::formula_preprocessing::operation_enums::*;
use crate::formula_preprocessing::parser::*;
use crate::formula_preprocessing::rename_vars::minimize_number_of_state_vars;
#[allow(unused_imports)]
use crate::formula_preprocessing::tokenizer::{print_tokens, tokenize_formula};
use crate::result_print::{print_results, print_results_fast};

use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use biodivine_lib_param_bn::BooleanNetwork;

use std::collections::{HashMap, HashSet};
use std::time::SystemTime;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PrintOptions {
    NoPrint,
    ShortPrint,
    LongPrint,
}

/// Returns the set of all uniquely named HCTL variables in the formula tree
/// Variable names are collected from quantifiers (bind, exists, forall)
/// (which is sufficient, since the formula has to be closed to be evaluated)
fn collect_unique_hctl_vars(formula_tree: Node, mut seen_vars: HashSet<String>) -> HashSet<String> {
    match formula_tree.node_type {
        NodeType::TerminalNode(_) => {}
        NodeType::UnaryNode(_, child) => {
            seen_vars.extend(collect_unique_hctl_vars(*child, seen_vars.clone()));
        }
        NodeType::BinaryNode(_, left, right) => {
            seen_vars.extend(collect_unique_hctl_vars(*left, seen_vars.clone()));
            seen_vars.extend(collect_unique_hctl_vars(*right, seen_vars.clone()));
        }
        // collect variables from exist and binder nodes
        NodeType::HybridNode(op, var_name, child) => {
            match op {
                HybridOp::Bind | HybridOp::Exists | HybridOp::Forall => {
                    seen_vars.insert(var_name); // we do not care whether insert is successful
                }
                _ => {}
            }
            seen_vars.extend(collect_unique_hctl_vars(*child, seen_vars.clone()));
        }
    }
    seen_vars
}

/// Performs the whole model checking process, including complete parsing at the beginning
/// Prints selected amount of result info (no prints / summary / all results printed)
pub fn analyse_formula(bn: BooleanNetwork, formula: String, print_option: PrintOptions) {
    let start = SystemTime::now();
    let print_progress = print_option != PrintOptions::NoPrint;

    let tokens = match tokenize_formula(formula) {
        Ok(r) => r,
        Err(e) => {
            println!("{}", e);
            Vec::new()
        }
    };
    //print_tokens(&tokens);

    match parse_hctl_formula(&tokens) {
        Ok(tree) => {
            if print_progress {
                println!("Parsed formula:   {}", tree.subform_str);
            }
            let new_tree = minimize_number_of_state_vars(*tree, HashMap::new(), String::new());
            if print_progress {
                println!("Modified formula: {}", new_tree.subform_str);
                println!("-----");
            }

            // count the number of needed HCTL vars and instantiate graph with it
            let num_hctl_vars = collect_unique_hctl_vars(new_tree.clone(), HashSet::new()).len();
            let graph = SymbolicAsyncGraph::new(bn, num_hctl_vars as i16).unwrap();

            if print_progress {
                println!(
                    "Loaded BN with {} components and {} parameters",
                    graph.as_network().num_vars(),
                    graph.symbolic_context().num_parameter_vars()
                );
                println!(
                    "Formula parse + graph build time: {}ms",
                    start.elapsed().unwrap().as_millis()
                );
            }

            let result = eval_minimized_tree(new_tree, &graph, print_progress);

            if print_progress {
                println!("Evaluation time: {}ms", start.elapsed().unwrap().as_millis());
                println!("-----");
            }

            match print_option {
                PrintOptions::LongPrint => print_results(&graph, &result, true),
                PrintOptions::ShortPrint => print_results_fast(&result),
                _ => {}
            }
        }
        Err(message) => println!("{}", message),
    }
}

/// Performs the model checking on GIVEN graph and returns resulting colored set
/// Panics if given symbolic graph does not support enough HCTL state-variables
pub fn model_check_formula(
    formula: String,
    stg: &SymbolicAsyncGraph,
) -> GraphColoredVertices {
    let tokens = tokenize_formula(formula).unwrap();
    let tree = parse_hctl_formula(&tokens).unwrap();
    let modified_tree = minimize_number_of_state_vars(*tree, HashMap::new(), String::new());

    // check that given extended symbolic graph supports enough stated variables
    let num_vars_formula = collect_unique_hctl_vars(modified_tree.clone(), HashSet::new()).len();
    if num_vars_formula > stg.symbolic_context().num_hctl_var_sets() as usize {
        panic!("Graph does not support enough HCTL state variables");
    }

    // get the result while not printing progress information
    eval_minimized_tree(modified_tree, stg, false)
}

#[allow(dead_code)]
/// Performs the model checking on GIVEN graph and returns resulting colored set.
/// Panics if given symbolic graph does not support enough HCTL state-variables.
/// Self-loops are not pre-computed, and thus are ignored in EX computation, which is fine for
/// some formulae, but incorrect for others - it is UNSAFE optimisation - only use it if you are
/// sure everything will work fine (+must not be used if formula involves !{x}:AX{x} sub-formulae).
pub fn model_check_formula_unsafe_ex(
    formula: String,
    stg: &SymbolicAsyncGraph,
) -> GraphColoredVertices {
    let tokens = tokenize_formula(formula).unwrap();
    let tree = parse_hctl_formula(&tokens).unwrap();
    let modified_tree = minimize_number_of_state_vars(*tree, HashMap::new(), String::new());

    // check that given extended symbolic graph supports enough stated variables
    let num_vars_formula = collect_unique_hctl_vars(modified_tree.clone(), HashSet::new()).len();
    if num_vars_formula > stg.symbolic_context().num_hctl_var_sets() as usize {
        panic!("Graph does not support enough HCTL state variables");
    }

    // do not consider self-loops during EX computation (UNSAFE optimisation)
    let self_loop_states = stg.mk_empty_vertices();
    eval_minimized_tree_unsafe_ex(modified_tree, stg, self_loop_states)
}

#[cfg(test)]
mod tests {
    use crate::analysis::{collect_unique_hctl_vars, model_check_formula};
    use crate::formula_preprocessing::parser::parse_hctl_formula;
    use crate::formula_preprocessing::rename_vars::minimize_number_of_state_vars;
    use crate::formula_preprocessing::tokenizer::tokenize_formula;
    use biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph;
    use biodivine_lib_param_bn::BooleanNetwork;
    use std::collections::{HashMap, HashSet};

    // model FISSION-YEAST-2008
    const MODEL_FISSION_YEAST: &str = r"
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

    // model MAMMALIAN-CELL-CYCLE-2006
    const MODEL_MAMMALIAN_CELL_CYCLE: &str = r"
targets,factors
v_Cdc20, v_CycB
v_Cdh1, ((v_Cdc20 | (v_p27 & !v_CycB)) | !(((v_p27 | v_CycB) | v_CycA) | v_Cdc20))
v_CycA, ((v_CycA & !(((v_Cdh1 & v_UbcH10) | v_Cdc20) | v_Rb)) | (v_E2F & !(((v_Cdh1 & v_UbcH10) | v_Cdc20) | v_Rb)))
v_CycB, !(v_Cdc20 | v_Cdh1)
v_CycE, (v_E2F & !v_Rb)
v_E2F, ((v_p27 & !(v_CycB | v_Rb)) | !(((v_p27 | v_Rb) | v_CycB) | v_CycA))
v_Rb, ((v_p27 & !(v_CycD | v_CycB)) | !((((v_CycE | v_p27) | v_CycB) | v_CycD) | v_CycA))
v_UbcH10, (((((v_UbcH10 & ((v_Cdh1 & ((v_CycB | v_Cdc20) | v_CycA)) | !v_Cdh1)) | (v_CycA & !v_Cdh1)) | (v_Cdc20 & !v_Cdh1)) | (v_CycB & !v_Cdh1)) | !((((v_UbcH10 | v_Cdh1) | v_CycB) | v_Cdc20) | v_CycA))
v_p27, ((v_p27 & !((v_CycD | (v_CycA & v_CycE)) | v_CycB)) | !((((v_CycE | v_p27) | v_CycB) | v_CycD) | v_CycA))
";

    // fully parametrized version of the model ASYMMETRIC-CELL-DIVISION-B
    const MODEL_ASYMMETRIC_CELL_DIVISION: &str = r"
DivJ -?? DivK
PleC -?? DivK
DivK -?? DivL
DivL -?? CckA
CckA -?? ChpT
ChpT -?? CpdR
CpdR -?? ClpXP_RcdA
ChpT -?? CtrAb
ClpXP_RcdA -?? CtrAb
DivK -?? DivJ
PleC -?? DivJ
DivK -?? PleC
";

    /// Run the evaluation tests for the set of given formulae on given model
    /// Compare numbers of results with the expected numbers given
    /// `test_tuples` consist of <formula, num_total, num_colors, num_states>
    fn test_model_check_basic_formulae(
        test_tuples: Vec<(&str, f64, f64, f64)>,
        bn: BooleanNetwork,
    ) {
        // test formulae use 3 HCTL vars at most
        let stg = SymbolicAsyncGraph::new(bn, 3).unwrap();

        for (formula, num_total, num_colors, num_states) in test_tuples {
            let result = model_check_formula(formula.to_string(), &stg);
            assert_eq!(num_total, result.approx_cardinality());
            assert_eq!(num_colors, result.colors().approx_cardinality());
            assert_eq!(num_states, result.vertices().approx_cardinality());
        }
    }

    #[test]
    /// Test evaluation of several important formulae on model FISSION-YEAST-2008
    /// Compare numbers of results with the numbers acquired by Python model checker or AEON
    fn test_model_check_basic_formulae_yeast() {
        // tuples consisting of <formula, num_total, num_colors, num_states>
        // num_x are numbers of expected results
        let test_tuples = vec![
            ("!{x}: AG EF {x}", 76., 2., 76.),
            ("!{x}: AX {x}", 12., 1., 12.),
            ("!{x}: AX EF {x}", 132., 2., 132.),
            ("AF (!{x}: AX {x})", 60., 1., 60.),
            ("!{x}: 3{y}: (@{x}: ~{y} & AX {x}) & (@{y}: AX {y})", 12., 1., 12.),
            ("3{x}: 3{y}: (@{x}: ~{y} & AX {x}) & (@{y}: AX {y}) & EF ({x} & (!{z}: AX {z})) & EF ({y} & (!{z}: AX {z})) & AX (EF ({x} & (!{z}: AX {z})) ^ EF ({y} & (!{z}: AX {z})))", 11., 1., 11.),
            ("!{x}: (AX (AF {x}))", 12., 1., 12.),
            ("AF (!{x}: (AX (~{x} & AF {x})))", 0., 0., 0.),
            ("AF (!{x}: ((AX (~{x} & AF {x})) & (EF (!{y}: EX ~AF {y}))))", 0., 0., 0.),
            // TODO: more tests regarding formulae for inference using concrete observations
        ];

        // model is in bnet format
        let bn = BooleanNetwork::try_from_bnet(MODEL_FISSION_YEAST).unwrap();
        test_model_check_basic_formulae(test_tuples, bn);
    }

    #[test]
    /// Test evaluation of several important formulae on model MAMMALIAN-CELL-CYCLE-2006
    /// Compare numbers of results with the numbers acquired by Python model checker or AEON
    fn test_model_check_basic_formulae_mammal() {
        // tuples consisting of <formula, num_total, num_colors, num_states>
        // num_x are numbers of expected results
        let test_tuples = vec![
            ("!{x}: AG EF {x}", 113., 2., 113.),
            ("!{x}: AX {x}", 1., 1., 1.),
            ("!{x}: AX EF {x}", 425., 2., 425.),
            ("AF (!{x}: AX {x})", 32., 1., 32.),
            ("!{x}: 3{y}: (@{x}: ~{y} & AX {x}) & (@{y}: AX {y})", 0., 0., 0.),
            ("3{x}: 3{y}: (@{x}: ~{y} & AX {x}) & (@{y}: AX {y}) & EF ({x} & (!{z}: AX {z})) & EF ({y} & (!{z}: AX {z})) & AX (EF ({x} & (!{z}: AX {z})) ^ EF ({y} & (!{z}: AX {z})))", 0., 0., 0.),
            ("!{x}: (AX (AF {x}))", 1., 1., 1.),
            ("AF (!{x}: (AX (~{x} & AF {x})))", 0., 0., 0.),
            ("AF (!{x}: ((AX (~{x} & AF {x})) & (EF (!{y}: EX ~AF {y}))))", 0., 0., 0.),
            // TODO: more tests regarding formulae for inference using concrete observations
        ];

        // model is in bnet format
        let bn = BooleanNetwork::try_from_bnet(MODEL_MAMMALIAN_CELL_CYCLE).unwrap();
        test_model_check_basic_formulae(test_tuples, bn);
    }

    #[test]
    /// Test evaluation of several important formulae on model ASYMMETRIC-CELL-DIVISION-B
    /// Compare numbers of results with the numbers acquired by Python model checker or AEON
    fn test_model_check_basic_formulae_cell_division() {
        // tuples consisting of <formula, num_total, num_colors, num_states>
        // num_x are numbers of expected results
        let test_tuples = vec![
            ("!{x}: AG EF {x}", 109428736., 16777216., 512.),
            ("!{x}: AX {x}", 16777216., 13631488., 512.),
            ("!{x}: AX EF {x}", 143294464., 16777216., 512.),
            ("AF (!{x}: AX {x})", 5670699008., 13631488., 512.),
            ("!{x}: 3{y}: (@{x}: ~{y} & AX {x}) & (@{y}: AX {y})", 6291456., 3145728., 512.),
            ("3{x}: 3{y}: (@{x}: ~{y} & AX {x}) & (@{y}: AX {y}) & EF ({x} & (!{z}: AX {z})) & EF ({y} & (!{z}: AX {z})) & AX (EF ({x} & (!{z}: AX {z})) ^ EF ({y} & (!{z}: AX {z})))", 5767168., 3014656., 512.),
            ("!{x}: (AX (AF {x}))", 21102592., 14909440., 512.),
            ("AF (!{x}: (AX (~{x} & AF {x})))", 9830400., 1277952., 512.),
            ("AF (!{x}: ((AX (~{x} & AF {x})) & (EF (!{y}: EX ~AF {y}))))", 4718592., 589824., 512.),
            // TODO: more tests regarding formulae for inference using concrete observations
        ];

        // model is in aeon format
        let bn = BooleanNetwork::try_from(MODEL_ASYMMETRIC_CELL_DIVISION).unwrap();
        test_model_check_basic_formulae(test_tuples, bn);
    }

    /// Test evaluation of pairs of equivalent formulae on given BN model
    /// Compare whether the results are the same
    fn test_model_check_equivalences(bn: BooleanNetwork) {
        // test formulae use 3 HCTL vars at most
        let stg = SymbolicAsyncGraph::new(bn, 3).unwrap();

        let equivalent_formulae_pairs = vec![
            ("!{x}: AG EF {x}", "!{x}: AG EF ({x} & {x})"), // one is evaluated using attr pattern
            ("!{x}: AX {x}", "!{x}: AX ({x} & {x})"), // one is evaluated using fixed-point pattern
            ("!{x}: AX {x}", "!{x}: ~EX ~{x}"),
            ("!{x}: ((AG EF {x}) & (AG EF {x}))", "!{x}: AG EF {x}"), // one involves basic caching
            ("!{x}: !{y}: ((AG EF {x}) & (AG EF {y}))", "!{x}: AG EF {x}"), // one involves advanced caching
            ("3{x}: !{y}: ((AG EF {x}) & (AG EF {y}))", "!{x}: 3{y}: ((AG EF {y}) & (AG EF {x}))"), // one involves advanced caching
            ("!{x}: AX {x}", "!{y}: AX {y}"),
            ("!{x}: AX AF {x}", "!{x}: AX ~EG ~{x}"),
            ("!{x}: 3{y}: (@{x}: ~{y} & AX {x}) & (@{y}: AX {y})", "!{x}: 3{y}: (@{x}: ~{y} & (!{z}: AX {z})) & (@{y}: (!{z}: AX {z}))"),
        ];

        for (formula1, formula2) in equivalent_formulae_pairs {
            let result1 = model_check_formula(formula1.to_string(), &stg);
            let result2 = model_check_formula(formula2.to_string(), &stg);
            assert_eq!(result1, result2);
        }
    }

    #[test]
    /// Test evaluation of pairs of equivalent formulae on model FISSION-YEAST-2008
    fn test_model_check_equivalences_yeast() {
        let bn = BooleanNetwork::try_from_bnet(MODEL_FISSION_YEAST).unwrap();
        test_model_check_equivalences(bn);
    }

    #[test]
    /// Test evaluation of pairs of equivalent formulae on model MAMMALIAN-CELL-CYCLE-2006
    fn test_model_check_equivalences_mammal() {
        let bn = BooleanNetwork::try_from_bnet(MODEL_MAMMALIAN_CELL_CYCLE).unwrap();
        test_model_check_equivalences(bn);
    }

    #[test]
    /// Test evaluation of pairs of equivalent formulae on model ASYMMETRIC-CELL-DIVISION-B
    fn test_model_check_equivalences_cell_division() {
        let bn = BooleanNetwork::try_from(MODEL_ASYMMETRIC_CELL_DIVISION).unwrap();
        test_model_check_equivalences(bn);
    }

    #[test]
    #[should_panic]
    /// Test that function panics correctly if graph does not support enough state variables
    fn test_model_check_panic() {
        // create symbolic graph supporting only one variable
        let bn = BooleanNetwork::try_from_bnet(MODEL_FISSION_YEAST).unwrap();
        let stg = SymbolicAsyncGraph::new(bn, 1).unwrap();

        // define formula with two variables
        let formula = "!{x}: !{y}: (AX {x} & AX {y})".to_string();
        model_check_formula(formula, &stg);
    }

    #[test]
    /// Test regarding collecting state vars from HCTL formulae
    fn test_state_var_collecting() {
        // formula "FORKS1 & FORKS2" - both parts are semantically same, just use different var names
        let formula = "(!{x}: 3{y}: (@{x}: ~{y} & (!{z}: AX {z})) & (@{y}: (!{z}: AX {z}))) & (!{x1}: 3{y1}: (@{x1}: ~{y1} & (!{z1}: AX {z1})) & (@{y1}: (!{z1}: AX {z1})))".to_string();
        let tokens = tokenize_formula(formula).unwrap();
        let tree = parse_hctl_formula(&tokens).unwrap();

        // test for original tree
        let expected_vars = vec![
            "x".to_string(),
            "y".to_string(),
            "z".to_string(),
            "x1".to_string(),
            "y1".to_string(),
            "z1".to_string()
        ];
        assert_eq!(
            collect_unique_hctl_vars(*tree.clone(), HashSet::new()),
            HashSet::from_iter(expected_vars)
        );

        // and for tree with minimized number of renamed state vars
        let modified_tree = minimize_number_of_state_vars(*tree, HashMap::new(), String::new());
        let expected_vars = vec!["x".to_string(), "xx".to_string(), "xxx".to_string()];
        assert_eq!(
            collect_unique_hctl_vars(modified_tree, HashSet::new()),
            HashSet::from_iter(expected_vars)
        );
    }
}