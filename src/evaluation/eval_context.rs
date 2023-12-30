//! Contains the structure to hold useful data to speed-up the computation.

use crate::evaluation::mark_duplicates::{
    mark_duplicates_canonized_multiple, mark_duplicates_canonized_single,
};
use crate::evaluation::{FormulaWithDomains, VarDomainMap, VarRenameMap};
use crate::preprocessing::hctl_tree::HctlTreeNode;
use biodivine_lib_param_bn::symbolic_async_graph::GraphColoredVertices;
use std::collections::HashMap;

/// Struct holding information for efficient caching during the main computation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvalContext {
    /// Remaining duplicates - mapping from `sub-formula and its free var domains` to `the counter` of that duplicate.
    pub duplicates: HashMap<FormulaWithDomains, i32>,
    /// Mapping between `cached sub-formulae` and their corresponding tuple of 1) the `resulting relation`,
    /// 2) mapping from sub-formula's ` original variable names` of the to their `canonical form`.
    pub cache: HashMap<FormulaWithDomains, (GraphColoredVertices, VarRenameMap)>,
    /// Mapping of the `variable domain names` to the corresponding `raw sets`.
    /// Var domain raw set (its bdd) must not depend on any symbolic variables, thus needs no renaming.
    pub domain_raw_sets: HashMap<String, GraphColoredVertices>,
    /// Mapping the sub-formula's `free variable names` their `domain names` (if specified).
    /// The domains are needed if we are to compare two sub-formulae with free variables for equivalence.
    pub free_var_domains: VarDomainMap,
}

impl EvalContext {
    /// Instantiate the struct with precomputed duplicates. Cache and domain-related fields are left empty.
    pub fn new(duplicates: HashMap<FormulaWithDomains, i32>) -> EvalContext {
        EvalContext {
            duplicates,
            cache: HashMap::new(),
            domain_raw_sets: HashMap::new(),
            free_var_domains: VarDomainMap::new(),
        }
    }

    /// Instantiate the struct with duplicates computed from a syntactic `tree`.
    /// Cache and domain-related fields are left empty.
    pub fn from_single_tree(tree: &HctlTreeNode) -> EvalContext {
        EvalContext {
            duplicates: mark_duplicates_canonized_single(tree),
            cache: HashMap::new(),
            domain_raw_sets: HashMap::new(),
            free_var_domains: VarDomainMap::new(),
        }
    }

    /// Instantiate the struct with duplicates computed from multiple given syntactic `trees`.
    /// Cache and domain-related fields are left empty.
    pub fn from_multiple_trees(trees: &Vec<HctlTreeNode>) -> EvalContext {
        EvalContext {
            duplicates: mark_duplicates_canonized_multiple(trees),
            cache: HashMap::new(),
            domain_raw_sets: HashMap::new(),
            free_var_domains: VarDomainMap::new(),
        }
    }

    /// Get a ref to the `duplicates` field containing the sub-formulae and their counter.
    pub fn get_duplicates(&self) -> &HashMap<FormulaWithDomains, i32> {
        &self.duplicates
    }

    /// Get a ref to the `cache` field containing the cached sub-formulae, their result and var renaming.
    pub fn get_cache(&self) -> &HashMap<FormulaWithDomains, (GraphColoredVertices, VarRenameMap)> {
        &self.cache
    }

    /// Get a ref to the `domain_raw_sets` field containing the cached domain names and the raw sets.
    pub fn get_domain_raw_sets(&self) -> &HashMap<String, GraphColoredVertices> {
        &self.domain_raw_sets
    }

    /// Get a ref to the `free_var_domains` field containing the mapping from free vars to their domains.
    pub fn get_free_var_domains(&self) -> &VarDomainMap {
        &self.free_var_domains
    }

    /// Extend the standard evaluation context with two kinds of "pre-computed context" regarding wild-cards.
    ///
    /// `subst_context_properties` describes context of classical `wild-card properties` and it is put
    /// directly to the `cache` field.
    ///
    /// `subst_context_domains` describes context of `variable domains` and is put into the `domain_raw_sets` field.
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
            // also, there are no free variables, so domain list is empty
            let sub_formula_with_domains = (sub_formula.clone(), VarDomainMap::new());
            if self.duplicates.contains_key(&sub_formula_with_domains) {
                self.duplicates.insert(
                    sub_formula_with_domains.clone(),
                    self.duplicates[&sub_formula_with_domains] + 1,
                );
            } else {
                self.duplicates.insert(sub_formula_with_domains.clone(), 1);
            }

            // Add the raw sets directly to the cache to be used during evaluation.
            // The mapping for var renaming is empty, because there are no HCTL vars.
            self.cache.insert(
                (sub_formula, VarDomainMap::new()),
                (raw_set.clone(), HashMap::new()),
            );
        }

        // For each `domain` in `subst_context_domains`, just put it inside domains.
        for (domain_name, raw_set) in subst_context_domains.iter() {
            self.domain_raw_sets
                .insert(domain_name.clone(), raw_set.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::evaluation::eval_context::{EvalContext, VarDomainMap};
    use crate::mc_utils::get_extended_symbolic_graph;
    use crate::preprocessing::parser::{parse_extended_formula, parse_hctl_formula};

    use biodivine_lib_param_bn::BooleanNetwork;

    use std::collections::HashMap;

    #[test]
    /// Test equivalent ways to generate EvalContext object.
    fn generating_eval_context() {
        let formula = "!{x}: (AX {x} & AX {x})";
        let syntax_tree = parse_hctl_formula(formula).unwrap();

        let expected_duplicates = HashMap::from([(
            (
                "(AX {var0})".to_string(),
                VarDomainMap::from([("var0".to_string(), None)]),
            ),
            1,
        )]);
        let eval_info = EvalContext::new(expected_duplicates.clone());

        assert_eq!(eval_info, EvalContext::from_single_tree(&syntax_tree));
        assert_eq!(
            eval_info,
            EvalContext::from_multiple_trees(&vec![syntax_tree])
        );
        assert_eq!(eval_info.get_duplicates(), &expected_duplicates);

        // check that cache and domain sets are always initially empty
        assert!(eval_info.get_cache().is_empty());
        assert!(eval_info.get_domain_raw_sets().is_empty());
        // check that free variable domains are always initially empty (formula cant have free vars)
        assert!(eval_info.get_free_var_domains().is_empty());
    }

    #[test]
    /// Test extension of the EvalContext with "pre-computed cache" regarding wild-card nodes and var domains.
    fn eval_context_extending() {
        // prepare placeholder BN and STG
        let bn = BooleanNetwork::try_from_bnet("v1, v1").unwrap();
        let stg = get_extended_symbolic_graph(&bn, 2).unwrap();

        let formula =
            "!{var0} in %domain1%: 3{var1}: (@{var0}: ~{var1} & %subst%) & (@{var1}: %subst%)";
        let syntax_tree = parse_extended_formula(formula).unwrap();
        let mut eval_info = EvalContext::from_single_tree(&syntax_tree);
        let expected_duplicates =
            HashMap::from([(("%subst%".to_string(), VarDomainMap::new()), 1)]);

        assert_eq!(eval_info.get_duplicates(), &expected_duplicates);
        assert_eq!(eval_info.get_cache(), &HashMap::new());
        assert_eq!(eval_info.get_domain_raw_sets(), &HashMap::new());

        let raw_set_1 = stg.mk_unit_colored_vertices();
        let raw_set_2 = stg.mk_empty_colored_vertices();
        let subst_context_props = HashMap::from([("subst".to_string(), raw_set_1.clone())]);
        let subst_context_domains = HashMap::from([("domain1".to_string(), raw_set_2.clone())]);

        eval_info.extend_context_with_wild_cards(&subst_context_props, &subst_context_domains);
        let expected_cache = HashMap::from([(
            ("%subst%".to_string(), VarDomainMap::new()),
            (raw_set_1, HashMap::new()),
        )]);
        let expected_domains = HashMap::from([("domain1".to_string(), raw_set_2)]);
        // there should be one more duplicate for "%subst%" domain - all its occurrences will be cached
        let expected_duplicates =
            HashMap::from([(("%subst%".to_string(), VarDomainMap::new()), 2)]);

        assert_eq!(eval_info.get_duplicates(), &expected_duplicates);
        assert_eq!(eval_info.get_cache(), &expected_cache);
        assert_eq!(eval_info.get_domain_raw_sets(), &expected_domains);
    }
}
