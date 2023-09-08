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

    /// Extend the standard evaluation context with "pre-computed cache" regarding wild-card nodes.
    pub fn extend_context_with_wild_cards(
        &mut self,
        substitution_context: HashMap<String, GraphColoredVertices>
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
    use crate::evaluation::eval_info::EvalContext;
    use crate::preprocessing::parser::parse_hctl_formula;
    use std::collections::HashMap;

    #[test]
    /// Test equivalent ways to generate EvalContext object.
    fn test_eval_info_creation() {
        let formula = "!{x}: (AX {x} & AX {x})".to_string();
        let syntax_tree = parse_hctl_formula(formula.as_str()).unwrap();

        let expected_duplicates = HashMap::from([("(Ax {var0})".to_string(), 1)]);
        let eval_info = EvalContext::new(expected_duplicates.clone());

        assert_eq!(eval_info, EvalContext::from_single_tree(&syntax_tree));
        assert_eq!(eval_info, EvalContext::from_multiple_trees(&vec![syntax_tree]));
        assert_eq!(eval_info.get_duplicates(), expected_duplicates);
    }
}
