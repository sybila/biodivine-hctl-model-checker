//! High-level functionality regarding the whole model-checking process.

use crate::formula_evaluation::algorithm::{compute_steady_states, eval_node};
use crate::formula_evaluation::eval_info::EvalInfo;
use crate::formula_preprocessing::node::{HctlTreeNode, NodeType};
use crate::formula_preprocessing::operator_enums::HybridOp;
use crate::formula_preprocessing::parser::parse_hctl_formula;
use crate::formula_preprocessing::tokenizer::try_tokenize_formula;
use crate::formula_preprocessing::vars_props_manipulation::check_props_and_rename_vars;

use biodivine_lib_param_bn::symbolic_async_graph::{
    GraphColoredVertices, SymbolicAsyncGraph, SymbolicContext,
};
use biodivine_lib_param_bn::BooleanNetwork;

use std::collections::{HashMap, HashSet};

/// Create an extended symbolic graph that supports the number of needed HCTL variables.
pub fn get_extended_symbolic_graph(bn: &BooleanNetwork, num_hctl_vars: u16) -> SymbolicAsyncGraph {
    // for each BN var, `num_hctl_vars` new BDD vars must be created
    let mut map_num_vars = HashMap::new();
    for bn_var in bn.variables() {
        map_num_vars.insert(bn_var, num_hctl_vars);
    }
    let context = SymbolicContext::with_extra_state_variables(bn, &map_num_vars).unwrap();
    let unit = context.mk_constant(true);

    SymbolicAsyncGraph::with_custom_context(bn.clone(), context, unit).unwrap()
}

/// Compute the set of all uniquely named HCTL variables in the formula tree.
/// Variable names are collected from three quantifiers: bind, exists, forall (which is sufficient,
/// as the formula must not contain free variables).
pub fn collect_unique_hctl_vars(
    formula_tree: HctlTreeNode,
    mut seen_vars: HashSet<String>,
) -> HashSet<String> {
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

/// Check that extended symbolic graph's BDD supports enough extra variables for the computation.
/// There must be `num_hctl_vars` extra symbolic BDD vars for each BN variable.
fn check_hctl_var_support(stg: &SymbolicAsyncGraph, num_hctl_vars: usize) -> bool {
    for bn_var in stg.as_network().variables() {
        if num_hctl_vars > stg.symbolic_context().extra_state_variables(bn_var).len() {
            return false;
        }
    }
    true
}

/// Perform the model checking for the list of formulae on GIVEN graph and return the list
/// of resulting sets of colored vertices (in the same order as input formulae).
/// Return Error if the given extended symbolic graph does not support enough extra BDD variables to
/// represent all needed HCTL state-variables or if some formula is badly formed.
pub fn model_check_multiple_formulae(
    formulae: Vec<String>,
    stg: &SymbolicAsyncGraph,
) -> Result<Vec<GraphColoredVertices>, String> {
    // first parse all the formulae and check that graph supports enough HCTL vars
    let mut parsed_trees = Vec::new();
    for formula in formulae {
        let tokens = try_tokenize_formula(formula)?;
        let tree = parse_hctl_formula(&tokens)?;
        let modified_tree =
            check_props_and_rename_vars(*tree, HashMap::new(), String::new(), stg.as_network())?;

        // check that given extended symbolic graph supports enough stated variables
        let num_vars_formula =
            collect_unique_hctl_vars(modified_tree.clone(), HashSet::new()).len();
        if !check_hctl_var_support(stg, num_vars_formula) {
            return Err("Graph does not support enough HCTL state variables".to_string());
        }

        parsed_trees.push(modified_tree);
    }

    // find duplicate sub-formulae throughout all formulae + initiate caching structures
    let mut eval_info = EvalInfo::from_multiple_trees(&parsed_trees);
    // compute states with self-loops which will be needed, and add them to graph object
    let self_loop_states = compute_steady_states(stg);

    // evaluate the formulae (perform the actual model checking) and collect results
    let mut results: Vec<GraphColoredVertices> = Vec::new();
    for parse_tree in parsed_trees {
        results.push(eval_node(
            parse_tree,
            stg,
            &mut eval_info,
            &self_loop_states,
        ));
    }
    Ok(results)
}

/// Perform the model checking for given formula on GIVEN graph and return the resulting
/// set of colored vertices.
/// Return Error if the given extended symbolic graph does not support enough extra BDD variables
/// to represent all needed HCTL state-variables or if the formula is badly formed.
pub fn model_check_formula(
    formula: String,
    stg: &SymbolicAsyncGraph,
) -> Result<GraphColoredVertices, String> {
    let result = model_check_multiple_formulae(vec![formula], stg)?;
    Ok(result[0].clone())
}

#[allow(dead_code)]
/// Perform the model checking on GIVEN graph and return the resulting set of colored vertices.
/// Return Error if the given extended symbolic graph does not support enough extra BDD variables
/// to represent all needed HCTL state-variables or if some formula is badly formed.
/// Self-loops are not pre-computed, and thus are ignored in EX computation, which is fine for
/// some formulae, but incorrect for others - it is an UNSAFE optimisation - only use it if you are
/// sure everything will work fine.
/// This must NOT be used for formulae containing !{x}:AX{x} sub-formulae.
pub fn model_check_formula_unsafe_ex(
    formula: String,
    stg: &SymbolicAsyncGraph,
) -> Result<GraphColoredVertices, String> {
    let tokens = try_tokenize_formula(formula).unwrap();
    let tree = parse_hctl_formula(&tokens).unwrap();
    let modified_tree =
        check_props_and_rename_vars(*tree, HashMap::new(), String::new(), stg.as_network())?;
    // check that given extended symbolic graph supports enough stated variables
    let num_vars_formula = collect_unique_hctl_vars(modified_tree.clone(), HashSet::new()).len();
    if !check_hctl_var_support(stg, num_vars_formula) {
        return Err("Graph does not support enough HCTL state variables".to_string());
    }

    let mut eval_info = EvalInfo::from_single_tree(&modified_tree);

    // do not consider self-loops during EX computation (UNSAFE optimisation)
    Ok(eval_node(
        modified_tree,
        stg,
        &mut eval_info,
        &stg.mk_empty_vertices(),
    ))
}

#[cfg(test)]
mod tests {
    use crate::formula_preprocessing::parser::parse_hctl_formula;
    use crate::formula_preprocessing::tokenizer::try_tokenize_formula;
    use crate::formula_preprocessing::vars_props_manipulation::check_props_and_rename_vars;
    use crate::model_checking::{
        collect_unique_hctl_vars, get_extended_symbolic_graph, model_check_formula,
    };

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
Start, false
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

    /// Run the evaluation tests for the set of given formulae on given model.
    /// Compare numbers of results with the expected numbers given.
    /// The `test_tuples` consist of tuples <formula, num_total, num_colors, num_states>.
    fn test_model_check_basic_formulae(
        test_tuples: Vec<(&str, f64, f64, f64)>,
        bn: BooleanNetwork,
    ) {
        // test formulae use 3 HCTL vars at most
        let stg = get_extended_symbolic_graph(&bn, 3);

        for (formula, num_total, num_colors, num_states) in test_tuples {
            let result = model_check_formula(formula.to_string(), &stg).unwrap();
            assert_eq!(num_total, result.approx_cardinality());
            assert_eq!(num_colors, result.colors().approx_cardinality());
            assert_eq!(num_states, result.vertices().approx_cardinality());
        }
    }

    #[test]
    /// Test evaluation of several important formulae on model FISSION-YEAST-2008.
    /// Compare numbers of results with the numbers acquired by Python model checker or AEON.
    fn test_model_check_basic_formulae_yeast() {
        // tuples consisting of <formula, num_total, num_colors, num_states>
        // num_x are numbers of expected results
        let test_tuples = vec![
            ("!{x}: AG EF {x}", 12., 1., 12.),
            ("!{x}: AX {x}", 12., 1., 12.),
            ("!{x}: AX EF {x}", 68., 1., 68.),
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
    /// Test evaluation of several important formulae on model MAMMALIAN-CELL-CYCLE-2006.
    /// Compare numbers of results with the numbers acquired by Python model checker or AEON.
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
    /// Test evaluation of several important formulae on model ASYMMETRIC-CELL-DIVISION-B.
    /// Compare numbers of results with the numbers acquired by Python model checker or AEON.
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

    /// Test evaluation of pairs of equivalent formulae on given BN model.
    /// Compare whether the results are the same.
    fn test_model_check_equivalences(bn: BooleanNetwork) {
        // test formulae use 3 HCTL vars at most
        let stg = get_extended_symbolic_graph(&bn, 3);

        let equivalent_formulae_pairs = vec![
            // AU equivalence (where phi1 are attractors, and phi2 the rest)
            (
                "(~(!{x}:AG EF{x})) AU (!{x}:AG EF{x})",
                "~EG~(!{x}:AG EF{x}) & ~(~(!{x}:AG EF{x}) EU (~(!{x}:AG EF{x}) & (!{x}:AG EF{x})))",
            ),
            // formulae for attractors, one is evaluated directly through optimisation
            ("!{x}: AG EF {x}", "!{x}: AG EF ({x} & {x})"),
            // formulae for fixed-points, one is evaluated directly through optimisation
            ("!{x}: AX {x}", "!{x}: AX ({x} & {x})"),
            // formulae for fixed-points, but differently named variables
            ("!{x}: AX {x}", "!{y}: AX {y}"),
            // computation for one of these involves canonization and basic caching
            ("!{x}: AX {x}", "(!{x}: AX {x}) & (!{y}: AX {y})"),
            // AX equivalence
            ("!{x}: AX {x}", "!{x}: ~EX ~{x}"),
            // computation for one of these involves basic caching
            ("!{x}: ((AG EF {x}) & (AG EF {x}))", "!{x}: AG EF {x}"),
            // computation for one of these involves advanced canonized caching
            ("!{x}: !{y}: ((AG EF {x}) & (AG EF {y}))", "!{x}: AG EF {x}"),
            // different order of quantifiers
            (
                "3{x}: !{y}: ((AG EF {x}) & (AG EF {y}))",
                "!{x}: 3{y}: ((AG EF {y}) & (AG EF {x}))",
            ),
            // AF equivalence
            ("!{x}: AX AF {x}", "!{x}: AX ~EG ~{x}"),
            // steady-states in bi-stable dynamics expressed in different ways (2 vs 3 variables)
            (
                "!{x}: 3{y}: (@{x}: ~{y} & AX {x}) & (@{y}: AX {y})",
                "!{x}: 3{y}: (@{x}: ~{y} & (!{z}: AX {z})) & (@{y}: (!{z}: AX {z}))",
            ),
        ];

        for (formula1, formula2) in equivalent_formulae_pairs {
            let result1 = model_check_formula(formula1.to_string(), &stg).unwrap();
            let result2 = model_check_formula(formula2.to_string(), &stg).unwrap();
            assert_eq!(result1, result2);
        }
    }

    #[test]
    /// Test evaluation of pairs of equivalent formulae on model FISSION-YEAST-2008.
    fn test_model_check_equivalences_yeast() {
        let bn = BooleanNetwork::try_from_bnet(MODEL_FISSION_YEAST).unwrap();
        test_model_check_equivalences(bn);
    }

    #[test]
    /// Test evaluation of pairs of equivalent formulae on model MAMMALIAN-CELL-CYCLE-2006.
    fn test_model_check_equivalences_mammal() {
        let bn = BooleanNetwork::try_from_bnet(MODEL_MAMMALIAN_CELL_CYCLE).unwrap();
        test_model_check_equivalences(bn);
    }

    #[test]
    /// Test evaluation of pairs of equivalent formulae on model ASYMMETRIC-CELL-DIVISION-B.
    fn test_model_check_equivalences_cell_division() {
        let bn = BooleanNetwork::try_from(MODEL_ASYMMETRIC_CELL_DIVISION).unwrap();
        test_model_check_equivalences(bn);
    }

    #[test]
    /// Test that function errors correctly if graph does not support enough state variables.
    fn test_model_check_error_1() {
        // create symbolic graph supporting only one variable
        let bn = BooleanNetwork::try_from_bnet(MODEL_FISSION_YEAST).unwrap();
        let stg = get_extended_symbolic_graph(&bn, 1);

        // define formula with two variables
        let formula = "!{x}: !{y}: (AX {x} & AX {y})".to_string();
        assert!(model_check_formula(formula, &stg).is_err());
    }

    #[test]
    /// Test that function errors correctly if formula contains free variables.
    fn test_model_check_error_2() {
        // create placeholder symbolic graph
        let bn = BooleanNetwork::try_from_bnet(MODEL_FISSION_YEAST).unwrap();
        let stg = get_extended_symbolic_graph(&bn, 2);

        // define formula that contains free variable
        let formula = "AX {x}".to_string();
        assert!(model_check_formula(formula, &stg).is_err());
    }

    #[test]
    /// Test that function errors correctly if formula contains several times quantified vars.
    fn test_model_check_error_3() {
        // create placeholder symbolic graph
        let bn = BooleanNetwork::try_from_bnet(MODEL_FISSION_YEAST).unwrap();
        let stg = get_extended_symbolic_graph(&bn, 2);

        // define formula with several times quantified var
        let formula = "!{x}: !{x}: AX {x}".to_string();
        assert!(model_check_formula(formula, &stg).is_err());
    }

    #[test]
    /// Test that function errors correctly if formula contains invalid propositions.
    fn test_model_check_error_4() {
        // create placeholder symbolic graph
        let bn = BooleanNetwork::try_from_bnet(MODEL_FISSION_YEAST).unwrap();
        let stg = get_extended_symbolic_graph(&bn, 2);

        // define formula with invalid proposition
        let formula = "AX invalid_proposition".to_string();
        assert!(model_check_formula(formula, &stg).is_err());
    }

    #[test]
    /// Test regarding collecting state vars from HCTL formulae.
    fn test_state_var_collecting() {
        // formula "FORKS1 & FORKS2" - both parts are semantically same, just use different var names
        let formula = "(!{x}: 3{y}: (@{x}: ~{y} & (!{z}: AX {z})) & (@{y}: (!{z}: AX {z}))) & (!{x1}: 3{y1}: (@{x1}: ~{y1} & (!{z1}: AX {z1})) & (@{y1}: (!{z1}: AX {z1})))".to_string();
        let tokens = try_tokenize_formula(formula).unwrap();
        let tree = parse_hctl_formula(&tokens).unwrap();

        // test for original tree
        let expected_vars = vec![
            "x".to_string(),
            "y".to_string(),
            "z".to_string(),
            "x1".to_string(),
            "y1".to_string(),
            "z1".to_string(),
        ];
        assert_eq!(
            collect_unique_hctl_vars(*tree.clone(), HashSet::new()),
            HashSet::from_iter(expected_vars)
        );

        // define any placeholder bn
        let bn = BooleanNetwork::try_from_bnet("v1, v1").unwrap();

        // and for tree with minimized number of renamed state vars
        let modified_tree =
            check_props_and_rename_vars(*tree, HashMap::new(), String::new(), &bn).unwrap();
        let expected_vars = vec!["x".to_string(), "xx".to_string(), "xxx".to_string()];
        assert_eq!(
            collect_unique_hctl_vars(modified_tree, HashSet::new()),
            HashSet::from_iter(expected_vars)
        );
    }
}
