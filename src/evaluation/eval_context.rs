//! Contains the structure to hold useful data to speed-up the computation.

use crate::evaluation::mark_duplicate_subform::{
    mark_duplicates_canonized_multiple, mark_duplicates_canonized_single,
};
use crate::preprocessing::node::HctlTreeNode;

use biodivine_lib_param_bn::symbolic_async_graph::GraphColoredVertices;

use std::collections::HashMap;

/// Struct holding information for efficient caching during the main computation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvalContext {
    /// Duplicate sub-formulae and their counter
    pub duplicates: HashMap<String, i32>,
    /// Cached sub-formulae and their result + corresponding mapping of variable renaming
    pub cache: HashMap<String, (GraphColoredVertices, HashMap<String, String>)>,
}

impl EvalContext {
    /// Instantiate the struct with precomputed duplicates and empty cache.
    pub fn new(duplicates: HashMap<String, i32>) -> EvalContext {
        EvalContext {
            duplicates,
            cache: HashMap::new(),
        }
    }

    /// Instantiate the struct with precomputed duplicates and empty cache.
    pub fn from_single_tree(tree: &HctlTreeNode) -> EvalContext {
        EvalContext {
            duplicates: mark_duplicates_canonized_single(tree),
            cache: HashMap::new(),
        }
    }

    /// Instantiate the struct with precomputed duplicates and empty cache.
    pub fn from_multiple_trees(trees: &Vec<HctlTreeNode>) -> EvalContext {
        EvalContext {
            duplicates: mark_duplicates_canonized_multiple(trees),
            cache: HashMap::new(),
        }
    }

    /// Get the duplicates field containing the sub-formulae and their counter.
    pub fn get_duplicates(&self) -> HashMap<String, i32> {
        self.duplicates.clone()
    }

    /// Get the cache field containing the cached sub-formulae, their result and var renaming.
    pub fn get_cache(&self) -> HashMap<String, (GraphColoredVertices, HashMap<String, String>)> {
        self.cache.clone()
    }

    /// Extend the standard evaluation context with "pre-computed cache" regarding wild-card nodes.
    pub fn extend_context_with_wild_cards(
        &mut self,
        substitution_context: HashMap<String, GraphColoredVertices>,
    ) {
        // For each `wild-card proposition` in `substitution_context`, increment its duplicate
        // counter. That way, the first occurrence will also be treated as duplicate and taken from
        // cache directly.
        for (prop_name, raw_set) in substitution_context.into_iter() {
            // we dont have to compute canonical sub-formula, because there are no HCTL variables
            let sub_formula = format!("%{}%", prop_name);
            if self.duplicates.contains_key(sub_formula.as_str()) {
                self.duplicates.insert(
                    sub_formula.clone(),
                    self.duplicates[sub_formula.as_str()] + 1,
                );
            } else {
                self.duplicates.insert(sub_formula.clone(), 1);
            }

            // Add the raw sets directly to the cache to be used during evaluation.
            // The mapping for var renaming is empty, because there are no HCTL vars.
            self.cache.insert(sub_formula, (raw_set, HashMap::new()));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::evaluation::eval_context::EvalContext;
    use crate::mc_utils::get_extended_symbolic_graph;
    use crate::preprocessing::parser::{parse_extended_formula, parse_hctl_formula};

    use biodivine_lib_param_bn::BooleanNetwork;

    use std::collections::HashMap;

    #[test]
    /// Test equivalent ways to generate EvalContext object.
    fn test_eval_context_creation() {
        let formula = "!{x}: (AX {x} & AX {x})";
        let syntax_tree = parse_hctl_formula(formula).unwrap();

        let expected_duplicates = HashMap::from([("(Ax {var0})".to_string(), 1)]);
        let eval_info = EvalContext::new(expected_duplicates.clone());

        assert_eq!(eval_info, EvalContext::from_single_tree(&syntax_tree));
        assert_eq!(
            eval_info,
            EvalContext::from_multiple_trees(&vec![syntax_tree])
        );
        assert_eq!(eval_info.get_duplicates(), expected_duplicates);

        // check that cache is always initially empty
        assert!(eval_info.get_cache().is_empty());
    }

    #[test]
    /// Test extension of the EvalContext with "pre-computed cache" regarding wild-card nodes.
    fn test_eval_context_extension() {
        // prepare placeholder BN and STG
        let bn = BooleanNetwork::try_from_bnet("v1, v1").unwrap();
        let stg = get_extended_symbolic_graph(&bn, 2).unwrap();

        let formula = "!{x}: 3{y}: (@{x}: ~{y} & %subst%) & (@{y}: %subst%)";
        let syntax_tree = parse_extended_formula(formula).unwrap();
        let mut eval_info = EvalContext::from_single_tree(&syntax_tree);

        assert_eq!(
            eval_info.get_duplicates(),
            HashMap::from([("%subst%".to_string(), 1)])
        );
        assert_eq!(eval_info.get_cache(), HashMap::new());

        let raw_set = stg.mk_unit_colored_vertices();
        let substitution_context = HashMap::from([("subst".to_string(), raw_set.clone())]);
        eval_info.extend_context_with_wild_cards(substitution_context);
        let expected_cache = HashMap::from([("%subst%".to_string(), (raw_set, HashMap::new()))]);

        assert_eq!(
            eval_info.get_duplicates(),
            HashMap::from([("%subst%".to_string(), 2)])
        );
        assert_eq!(eval_info.get_cache(), expected_cache);
    }
}
