//! Contains the structure to hold useful data to speed-up the computation.

use crate::evaluation::mark_duplicate_subform::{
    mark_duplicates_canonized_multiple, mark_duplicates_canonized_single,
};
use crate::preprocessing::node::HctlTreeNode;

use biodivine_lib_param_bn::symbolic_async_graph::GraphColoredVertices;

use std::collections::HashMap;

/// Struct holding information for efficient caching during the main computation.
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
