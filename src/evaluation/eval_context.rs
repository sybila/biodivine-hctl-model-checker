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
    /// Similar to cache, but this time just mapping the name of var domain to the raw set.
    /// Var domain bdd must not depend on any symbolic variables, thus needs no renaming.
    pub var_domains: HashMap<String, GraphColoredVertices>,
}

impl EvalContext {
    /// Instantiate the struct with precomputed duplicates and empty cache.
    pub fn new(duplicates: HashMap<String, i32>) -> EvalContext {
        EvalContext {
            duplicates,
            cache: HashMap::new(),
            var_domains: HashMap::new(),
        }
    }

    /// Instantiate the struct with precomputed duplicates and empty cache.
    pub fn from_single_tree(tree: &HctlTreeNode) -> EvalContext {
        EvalContext {
            duplicates: mark_duplicates_canonized_single(tree),
            cache: HashMap::new(),
            var_domains: HashMap::new(),
        }
    }

    /// Instantiate the struct with precomputed duplicates and empty cache.
    pub fn from_multiple_trees(trees: &Vec<HctlTreeNode>) -> EvalContext {
        EvalContext {
            duplicates: mark_duplicates_canonized_multiple(trees),
            cache: HashMap::new(),
            var_domains: HashMap::new(),
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

    /// Get the var_domains field containing the cached domain names and the raw sets.
    pub fn get_var_domains(&self) -> HashMap<String, GraphColoredVertices> {
        self.var_domains.clone()
    }

    /// Extend the standard evaluation context with two kinds of "pre-computed cache" regarding
    /// wild-cards.
    ///
    /// `subst_context_properties` describes context of classical `wild-card properties` and is put
    /// directly to the `cache` field.
    ///
    /// `subst_context_domains` describes context of `variable domains` and is put into the
    /// `var_domains` field.
    pub fn extend_context_with_wild_cards(
        &mut self,
        subst_context_properties: &HashMap<String, GraphColoredVertices>,
        subst_context_domains: &HashMap<String, GraphColoredVertices>,
    ) {
        // For each `wild-card proposition` in `subst_context_properties`, increment its duplicate
        // counter. That way, the first occurrence will also be treated as duplicate and taken from
        // cache directly.
        for (prop_name, raw_set) in subst_context_properties.iter() {
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
            self.cache
                .insert(sub_formula, (raw_set.clone(), HashMap::new()));
        }

        // For each `domain` in `subst_context_domains`, just put it inside domains.
        for (domain_name, raw_set) in subst_context_domains.iter() {
            self.var_domains
                .insert(domain_name.clone(), raw_set.clone());
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
    fn generating_eval_context() {
        let formula = "!{x}: (AX {x} & AX {x})";
        let syntax_tree = parse_hctl_formula(formula).unwrap();

        let expected_duplicates = HashMap::from([("(AX {var0})".to_string(), 1)]);
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
    /// Test extension of the EvalContext with "pre-computed cache" regarding wild-card nodes and var domains.
    fn eval_context_extending() {
        // prepare placeholder BN and STG
        let bn = BooleanNetwork::try_from_bnet("v1, v1").unwrap();
        let stg = get_extended_symbolic_graph(&bn, 2).unwrap();

        let formula = "!{x} in %domain1%: 3{y}: (@{x}: ~{y} & %subst%) & (@{y}: %subst%)";
        let syntax_tree = parse_extended_formula(formula).unwrap();
        let mut eval_info = EvalContext::from_single_tree(&syntax_tree);

        assert_eq!(
            eval_info.get_duplicates(),
            HashMap::from([("%subst%".to_string(), 1)])
        );
        assert_eq!(eval_info.get_cache(), HashMap::new());
        assert_eq!(eval_info.get_var_domains(), HashMap::new());

        let raw_set_1 = stg.mk_unit_colored_vertices();
        let raw_set_2 = stg.mk_empty_colored_vertices();
        let subst_context_props = HashMap::from([("subst".to_string(), raw_set_1.clone())]);
        let subst_context_domains = HashMap::from([("domain1".to_string(), raw_set_2.clone())]);

        eval_info.extend_context_with_wild_cards(&subst_context_props, &subst_context_domains);
        let expected_cache = HashMap::from([("%subst%".to_string(), (raw_set_1, HashMap::new()))]);
        let expected_domains = HashMap::from([("domain1".to_string(), raw_set_2)]);

        assert_eq!(
            eval_info.get_duplicates(),
            HashMap::from([("%subst%".to_string(), 2)])
        );
        assert_eq!(eval_info.get_cache(), expected_cache);
        assert_eq!(eval_info.get_var_domains(), expected_domains);
    }
}
