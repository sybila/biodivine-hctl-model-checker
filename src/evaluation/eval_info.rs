//! Contains the structure to hold useful data to speed-up the computation.

use crate::evaluation::mark_duplicate_subform::{
    mark_duplicates_canonized_multiple, mark_duplicates_canonized_single,
};
use crate::preprocessing::node::HctlTreeNode;

use biodivine_lib_param_bn::symbolic_async_graph::GraphColoredVertices;

use std::collections::HashMap;

/// Struct holding information for efficient caching during the main computation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvalInfo {
    /// Duplicate sub-formulae and their counter
    pub duplicates: HashMap<String, i32>,
    /// Cached sub-formulae and their result + variable renaming mapping
    pub cache: HashMap<String, (GraphColoredVertices, HashMap<String, String>)>,
}

impl EvalInfo {
    /// Instantiate the struct with precomputed duplicates and empty cache.
    pub fn new(duplicates: HashMap<String, i32>) -> EvalInfo {
        EvalInfo {
            duplicates,
            cache: HashMap::new(),
        }
    }

    /// Instantiate the struct with precomputed duplicates and empty cache.
    pub fn from_single_tree(tree: &HctlTreeNode) -> EvalInfo {
        EvalInfo {
            duplicates: mark_duplicates_canonized_single(tree),
            cache: HashMap::new(),
        }
    }

    /// Instantiate the struct with precomputed duplicates and empty cache.
    pub fn from_multiple_trees(trees: &Vec<HctlTreeNode>) -> EvalInfo {
        EvalInfo {
            duplicates: mark_duplicates_canonized_multiple(trees),
            cache: HashMap::new(),
        }
    }

    pub fn get_duplicates(&self) -> HashMap<String, i32> {
        self.duplicates.clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::evaluation::eval_info::EvalInfo;
    use crate::preprocessing::parser::parse_hctl_formula;
    use crate::preprocessing::tokenizer::try_tokenize_formula;
    use std::collections::HashMap;

    #[test]
    /// Test equivalent ways to generate EvalInfo object.
    fn test_eval_info_creation() {
        let formula = "!{x}: (AX {x} & AX {x})".to_string();
        let tokens = try_tokenize_formula(formula).unwrap();
        let syntax_tree = *parse_hctl_formula(&tokens).unwrap();

        let expected_duplicates = HashMap::from([("(Ax {var0})".to_string(), 1)]);
        let eval_info = EvalInfo::new(expected_duplicates.clone());

        assert_eq!(eval_info, EvalInfo::from_single_tree(&syntax_tree));
        assert_eq!(eval_info, EvalInfo::from_multiple_trees(&vec![syntax_tree]));
        assert_eq!(eval_info.get_duplicates(), expected_duplicates);
    }
}
