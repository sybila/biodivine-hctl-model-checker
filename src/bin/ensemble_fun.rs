use std::collections::HashMap;
use biodivine_lib_bdd::Bdd;
use biodivine_lib_param_bn::{BinaryOp, BooleanNetwork, FnUpdate, ParameterId, VariableId};
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph, SymbolicContext};
use hctl_model_checker::analysis::minimize_number_of_state_vars;
use hctl_model_checker::evaluator::eval_minimized_tree;
use hctl_model_checker::parser::parse_hctl_formula;
use hctl_model_checker::tokenizer::tokenize_recursive;

fn main() {

    let args = std::env::args().collect::<Vec<_>>();

    let mut models = Vec::new();
    for file in std::fs::read_dir(args[1].as_str()).unwrap() {
        let file = file.unwrap();
        let file_name = file.file_name().into_string().unwrap();
        if file_name.ends_with(".bnet") {
            println!("Reading {}", file_name);

            let bnet_string = std::fs::read_to_string(file.path()).unwrap();
            let bnet_model = BooleanNetwork::try_from_bnet(bnet_string.as_str()).unwrap();
            models.push(bnet_model);
        }
    }

    println!("Loaded {} models.", models.len());
    let canonical_model = models[0].clone();
    let canonical_context = SymbolicContext::new(&canonical_model, 2).unwrap();
    let canonical_stg = SymbolicAsyncGraph::new(canonical_model.clone(), 2).unwrap();

    let v_apoptosis = canonical_model.as_graph().find_variable("Apoptosis").unwrap();
    let v_cell_cycle_arrest = canonical_model.as_graph().find_variable("CellCycleArrest").unwrap();
    let v_emt = canonical_model.as_graph().find_variable("EMT").unwrap();
    let v_invasion = canonical_model.as_graph().find_variable("Invasion").unwrap();
    let v_migration = canonical_model.as_graph().find_variable("Migration").unwrap();
    let v_metastasis = canonical_model.as_graph().find_variable("Metastasis").unwrap();

    let phenotype_hs = canonical_stg.mk_subspace(&[
        (v_apoptosis, false),
        (v_cell_cycle_arrest, false),
        (v_emt, false),
        (v_invasion, false),
        (v_migration, false),
        (v_metastasis, false),
    ]);

    let phenotype_a_ca = canonical_stg.mk_subspace(&[
        (v_apoptosis, true),
        (v_cell_cycle_arrest, true),
        (v_emt, false),
        (v_invasion, false),
        (v_migration, false),
        (v_metastasis, false),
    ]);

    let phenotype_ca_emt = canonical_stg.mk_subspace(&[
        (v_apoptosis, false),
        (v_cell_cycle_arrest, true),
        (v_emt, true),
        (v_invasion, false),
        (v_migration, false),
        (v_metastasis, false),
    ]);

    let phenotype_meta_ca = canonical_stg.mk_subspace(&[
        (v_apoptosis, false),
        (v_cell_cycle_arrest, true),
        (v_emt, true),
        (v_invasion, true),
        (v_migration, true),
        (v_metastasis, true),
    ]);

    let merged = merge_ensemble(&models);
    //println!("{}", merged.to_string());
    let merged_stg = SymbolicAsyncGraph::new(merged.clone(), 2).unwrap();
    println!("Merged STG: {}", merged_stg.unit_colored_vertices().approx_cardinality());
    let sinks = merged_stg.mk_unit_colored_vertices().minus(&merged_stg.can_post(merged_stg.unit_colored_vertices()));
    println!("Merged sinks: {} / {}", sinks.vertices().approx_cardinality(), sinks.colors().approx_cardinality());

    let hs_phenotype_formula = "~Apoptosis & ~CellCycleArrest & ~EMT & ~Invasion & ~Migration & ~Metastasis";
    let hs_bistable = format!("3{{x}}: 3{{y}}: EF ({{x}} & AX ({{x}} & {})) & EF ({{y}} & AX ({{y}} & {})) & (@{{x}}: ~{{y}})", hs_phenotype_formula, hs_phenotype_formula);
    let states = model_check_formula_unsafe(hs_bistable, &merged_stg);
    println!("Two HS sinks: {} / {}", states.vertices().approx_cardinality(), states.colors().approx_cardinality());

    /*for (i, model) in models.iter().enumerate() {
        let stg = SymbolicAsyncGraph::new(model.clone(), 2).unwrap();
        let sinks = stg.mk_unit_colored_vertices().minus(&stg.can_post(stg.unit_colored_vertices()));

        // Check that every sink belongs to some phenotype and that all phenotypes are represented.
        assert!(!sinks.intersect(&phenotype_hs).is_empty());
        assert!(!sinks.intersect(&phenotype_a_ca).is_empty());
        assert!(!sinks.intersect(&phenotype_ca_emt).is_empty());
        assert!(!sinks.intersect(&phenotype_meta_ca).is_empty());

        assert!(sinks
            .minus(&phenotype_hs)
            .minus(&phenotype_a_ca)
            .minus(&phenotype_ca_emt)
            .minus(&phenotype_meta_ca)
            .is_empty()
        );

        println!("[{}] Sinks: {}", i, sinks.approx_cardinality());

        println!("HS sinks: {}", sinks.intersect(&phenotype_hs).approx_cardinality());
        let hs_phenotype_formula = "~Apoptosis & ~CellCycleArrest & ~EMT & ~Invasion & ~Migration & ~Metastasis";

        //let hs_bistable = format!("3{{x}}: 3{{y}}: EF {{x}} & EF {{y}} & (@{{x}}: ~{{y}}) & (@{{x}}: (AX {{x}} & ({}))) & (@{{y}}: (AX {{y}} & ({}))) ", hs_phenotype_formula, hs_phenotype_formula);
        let hs_bistable = format!("3{{x}}: 3{{y}}: EF ({{x}} & AX {{x}}) & EF ({{y}} & AX {{y}}) & (@{{x}}: ~{{y}}) & (@{{x}}: (AX {{x}} & ({}))) & (@{{y}}: (AX {{y}} & ({}))) ", hs_phenotype_formula, hs_phenotype_formula);
        let states = model_check_formula_unsafe(hs_bistable, &stg);
        println!("Two HS sinks: {}", states.approx_cardinality());
    }*/

    //find_shared_values(&canonical_model, &canonical_context, &models)
}

/// Just performs the model checking on GIVEN graph and returns result, no prints happen
/// UNSAFE - does not parse the graph from formula, assumes that graph was created correctly
/// Graph must have enough HCTL variables for the formula
pub fn model_check_formula_unsafe(
    formula: String,
    stg: &SymbolicAsyncGraph,
) -> GraphColoredVertices {
    let tokens = tokenize_recursive(&mut formula.chars().peekable(), true).unwrap();
    let tree = parse_hctl_formula(&tokens).unwrap();
    let modified_tree = minimize_number_of_state_vars(*tree, HashMap::new(), String::new());
    eval_minimized_tree(modified_tree, stg)
}


pub fn merge_ensemble(ensemble: &[BooleanNetwork]) -> BooleanNetwork {
    let mut merged = BooleanNetwork::new(ensemble[0].as_graph().clone());
    let mut params = Vec::new();
    for i in 0..10 {
        let id = merged.add_parameter(format!("p{}", i).as_str(), 0).unwrap();
        params.push(id);
        println!("Added p{}", i);
    }

    fn id_to_expression(params: &[ParameterId], id: usize) -> FnUpdate {
        let param = params[0];
        let atom = if id % 2 == 1 {
            FnUpdate::mk_param(param, &[])
        } else {
            FnUpdate::mk_not(FnUpdate::mk_param(param, &[]))
        };
        if params.len() == 1 {
            assert!(id < 2);
            atom
        } else {
            let rest = id_to_expression(&params[1..], id / 2);
            FnUpdate::mk_binary(BinaryOp::And, atom, rest)
        }
    }

    for var in merged.variables() {
        let mut merged_fn = FnUpdate::mk_true();
        for (id, model) in ensemble.iter().enumerate() {
            let model_fn = model.get_update_function(var).clone().unwrap();
            let p_expr = id_to_expression(&params, id);
            let clause = FnUpdate::mk_binary(BinaryOp::Imp, p_expr, model_fn);
            merged_fn = FnUpdate::mk_binary(BinaryOp::And, merged_fn, clause);
        }
        merged.set_update_function(var, Some(merged_fn)).unwrap()
    }

    merged
}


#[allow(unused)]
pub fn find_shared_values(canonical_model: &BooleanNetwork, canonical_context: &SymbolicContext, models: &[BooleanNetwork]) {
    // BDDs that evaluate to "one" for input vectors where all models evaluate to one (for the associated variable).
    let mut shared_ones: HashMap<VariableId, Bdd> = canonical_model.variables()
        .map(|var| (var, canonical_context.mk_constant(true)))
        .collect();
    // The same but for zero.
    let mut shared_zeros: HashMap<VariableId, Bdd> = shared_ones.clone();

    for model in models.iter() {
        for var in model.variables() {
            let function = model.get_update_function(var).as_ref().unwrap();
            let fn_update = canonical_context.mk_fn_update_true(function);
            let shared_one = shared_ones.get_mut(&var).unwrap();
            *shared_one = shared_one.and(&fn_update);
            let shared_zero = shared_zeros.get_mut(&var).unwrap();
            *shared_zero = shared_zero.and_not(&fn_update);
        }
    }

    for var in canonical_model.variables() {
        let shared_one = shared_ones.get(&var).unwrap();
        let shared_zero = shared_zeros.get(&var).unwrap();
        let name = canonical_model.get_variable_name(var);

        let extra_order = (canonical_model.num_vars() - canonical_model.regulators(var).len()) as i32;
        let denominator = 2.0_f64.powi(extra_order);
        println!("{}: {} / {}", name, shared_one.cardinality() / denominator, shared_zero.cardinality() / denominator);
    }
}