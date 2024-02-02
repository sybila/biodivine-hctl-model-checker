use crate::_test_model_checking::{MODEL_CELL_CYCLE, MODEL_CELL_DIVISION, MODEL_YEAST};
use crate::mc_utils::get_extended_symbolic_graph;
use crate::model_checking::{model_check_formula, model_check_formula_unsafe_ex};
use crate::postprocessing::sanitizing::sanitize_colored_vertices;
use biodivine_lib_param_bn::BooleanNetwork;

#[test]
/// Test that the (potentially unsafe) optimization works on several formulae where it
/// should be applicable without any loss of information.
fn test_unsafe_optimization() {
    let bn1 = BooleanNetwork::try_from(MODEL_CELL_DIVISION).unwrap();
    let bn2 = BooleanNetwork::try_from_bnet(MODEL_CELL_CYCLE).unwrap();
    let bn3 = BooleanNetwork::try_from_bnet(MODEL_YEAST).unwrap();
    let bns = vec![bn1, bn2, bn3];

    for bn in bns {
        let stg = get_extended_symbolic_graph(&bn, 3).unwrap();
        // use formula for attractors that won't be recognized as the "attractor pattern"
        let formula = "!{x}: AG EF ({x} & {x})";
        let result1 = model_check_formula(formula, &stg).unwrap();
        // result of the unsafe eval must be sanitized
        let result2 =
            sanitize_colored_vertices(&stg, &model_check_formula_unsafe_ex(formula, &stg).unwrap());
        assert!(result1.as_bdd().iff(result2.as_bdd()).is_true());
    }
}
