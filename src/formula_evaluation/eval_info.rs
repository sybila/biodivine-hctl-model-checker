use crate::formula_evaluation::mark_duplicate_subform::*;
use crate::formula_preprocessing::parser::Node;

use biodivine_lib_param_bn::symbolic_async_graph::GraphColoredVertices;

use std::collections::HashMap;

/// Struct holding information for efficient caching during the main computation
pub struct EvalInfo {
    // duplicate sub-formulae and their counter
    pub duplicates: HashMap<String, i32>,
    // cached sub-formulae and their result + variable renaming mapping
    pub cache: HashMap<String, (GraphColoredVertices, HashMap<String, String>)>,
}

impl EvalInfo {
    /// instantiate the struct with precomputed duplicates and empty cache
    pub fn new(duplicates: HashMap<String, i32>) -> EvalInfo {
        EvalInfo {
            duplicates,
            cache: HashMap::new(),
        }
    }

    /// instantiate the struct with precomputed duplicates and empty cache
    pub fn from_single_tree(tree: &Node) -> EvalInfo {
        EvalInfo {
            duplicates: mark_duplicates_canonized_single(tree),
            cache: HashMap::new(),
        }
    }

    /// instantiate the struct with precomputed duplicates and empty cache
    pub fn from_multiple_trees(trees: &Vec<Node>) -> EvalInfo {
        EvalInfo {
            duplicates: mark_duplicates_canonized_multiple(trees),
            cache: HashMap::new(),
        }
    }

    pub fn get_duplicates(&self) -> HashMap<String, i32> {
        self.duplicates.clone()
    }
}
