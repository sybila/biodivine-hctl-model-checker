//! Contains the high-level model-checking algorithm and few optimisations.

use crate::_aeon_algorithms::scc_computation::compute_attractor_states;
use crate::evaluation::canonization::get_canonical_and_renaming;
use crate::evaluation::eval_context::EvalContext;
use crate::evaluation::hctl_operators_eval::*;
use crate::evaluation::low_level_operations::{
    compute_valid_domain_for_var, restrict_stg_unit_bdd, substitute_hctl_var,
};
use crate::evaluation::{VarDomainMap, VarRenameMap};
use crate::preprocessing::hctl_tree::{HctlTreeNode, NodeType};
use crate::preprocessing::operator_enums::*;

use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::fixed_points::FixedPoints;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};

/// Recursively evaluate the formula represented by a sub-tree `node` on the given `graph`.
///
/// `eval_context` holds the current version additional data used for optimization, such as
/// pre-computed set of `duplicate` sub-formulae and corresponding `cache` set, as well as domains of free vars.
///
/// The `steady_states` are needed to include self-loops in computing predecessors.
pub fn eval_node(
    node: HctlTreeNode,
    graph: &SymbolicAsyncGraph,
    eval_context: &mut EvalContext,
    steady_states: &GraphColoredVertices,
) -> GraphColoredVertices {
    // first check whether this node does not belong in the duplicates
    let mut save_to_cache = false;

    // get canonized form of this sub-formula, and mapping between original and canonized variable names
    let (canonized_form, renaming) = get_canonical_and_renaming(node.to_string());
    // rename the variables in the domain map to their canonical form (duplicate formulae are always canonical)
    // only include the FREE canonical variables that are actually contained in the sub-formula
    // example: given "!{x}:!{y}: (AX {y})", its sub-formula "AX {x}" would have one "None" domain for "var0"
    let mut canonical_domains: VarDomainMap = VarDomainMap::new();
    for (variable, domain) in &eval_context.free_var_domains {
        if renaming.contains_key(variable) {
            canonical_domains.insert(renaming.get(variable).unwrap().clone(), domain.clone());
        }
    }
    // canonical version of the current formula and canonized mappings of its domains
    let canonized_formula_with_domains = (canonized_form.clone(), canonical_domains.clone());

    if eval_context
        .duplicates
        .contains_key(&canonized_formula_with_domains)
    {
        if eval_context
            .cache
            .contains_key(&canonized_formula_with_domains)
        {
            // decrement number of duplicates left
            *eval_context
                .duplicates
                .get_mut(&canonized_formula_with_domains)
                .unwrap() -= 1;

            // get cached result, but it might be using differently named state-variables
            // so we might have to rename them later
            let (mut result, result_renaming) = eval_context
                .cache
                .get(&canonized_formula_with_domains)
                .unwrap()
                .clone();

            // if we already visited all of the duplicates, lets delete the cached value
            if eval_context.duplicates[&canonized_formula_with_domains] == 0 {
                eval_context
                    .duplicates
                    .remove(&canonized_formula_with_domains);
                eval_context.cache.remove(&canonized_formula_with_domains);
            }

            // since we are working with canonical cache, we might need to rename vars in result bdd
            let mut reverse_renaming: VarRenameMap = VarRenameMap::new();
            for (var_curr, var_canon) in renaming.iter() {
                reverse_renaming.insert(var_canon.clone(), var_curr.clone());
            }
            for (var_res, var_canon) in result_renaming.iter() {
                let var_curr = reverse_renaming.get(var_canon).unwrap();
                result = substitute_hctl_var(graph, &result, var_res, var_curr);
            }
            return result;
        } else {
            // if the cache does not contain result for this subformula, set insert flag
            save_to_cache = true;
        }
    }

    // first lets check for special cases, which can be optimised:
    // 1) attractors
    if is_attractor_pattern(&node) {
        let result = compute_attractor_states(graph, graph.mk_unit_colored_vertices());
        if save_to_cache {
            eval_context
                .cache
                .insert(canonized_formula_with_domains, (result.clone(), renaming));
        }
        return result;
    }
    // 2) fixed-points
    if is_fixed_point_pattern(&node) {
        return steady_states.clone();
    }

    let result = match node.node_type {
        NodeType::TerminalNode(atom) => match atom {
            Atomic::True => graph.mk_unit_colored_vertices(),
            Atomic::False => graph.mk_empty_colored_vertices(),
            Atomic::Var(name) => eval_hctl_var(graph, name.as_str()),
            Atomic::Prop(name) => eval_prop(graph, &name),
            // should not be reachable, as wild-card nodes are always evaluated earlier using cache
            Atomic::WildCardProp(_) => unreachable!(),
        },
        NodeType::UnaryNode(op, child) => match op {
            UnaryOp::Not => eval_neg(
                graph,
                &eval_node(*child, graph, eval_context, steady_states),
            ),
            UnaryOp::EX => eval_ex(
                graph,
                &eval_node(*child, graph, eval_context, steady_states),
                steady_states,
            ),
            UnaryOp::AX => eval_ax(
                graph,
                &eval_node(*child, graph, eval_context, steady_states),
                steady_states,
            ),
            UnaryOp::EF => eval_ef_saturated(
                graph,
                &eval_node(*child, graph, eval_context, steady_states),
            ),
            UnaryOp::AF => eval_af(
                graph,
                &eval_node(*child, graph, eval_context, steady_states),
                steady_states,
            ),
            UnaryOp::EG => eval_eg(
                graph,
                &eval_node(*child, graph, eval_context, steady_states),
                steady_states,
            ),
            UnaryOp::AG => eval_ag(
                graph,
                &eval_node(*child, graph, eval_context, steady_states),
            ),
        },
        NodeType::BinaryNode(op, left, right) => {
            match op {
                BinaryOp::And => eval_node(*left, graph, eval_context, steady_states)
                    .intersect(&eval_node(*right, graph, eval_context, steady_states)),
                BinaryOp::Or => eval_node(*left, graph, eval_context, steady_states)
                    .union(&eval_node(*right, graph, eval_context, steady_states)),
                BinaryOp::Xor => eval_xor(
                    graph,
                    &eval_node(*left, graph, eval_context, steady_states),
                    &eval_node(*right, graph, eval_context, steady_states),
                ),
                BinaryOp::Imp => eval_imp(
                    graph,
                    &eval_node(*left, graph, eval_context, steady_states),
                    &eval_node(*right, graph, eval_context, steady_states),
                ),
                BinaryOp::Iff => eval_equiv(
                    graph,
                    &eval_node(*left, graph, eval_context, steady_states),
                    &eval_node(*right, graph, eval_context, steady_states),
                ),
                BinaryOp::EU => eval_eu_saturated(
                    graph,
                    &eval_node(*left, graph, eval_context, steady_states),
                    &eval_node(*right, graph, eval_context, steady_states),
                ),
                BinaryOp::AU => eval_au(
                    graph,
                    &eval_node(*left, graph, eval_context, steady_states),
                    &eval_node(*right, graph, eval_context, steady_states),
                    steady_states,
                ),
                BinaryOp::EW => eval_ew(
                    graph,
                    &eval_node(*left, graph, eval_context, steady_states),
                    &eval_node(*right, graph, eval_context, steady_states),
                    steady_states,
                ),
                BinaryOp::AW => eval_aw(
                    graph,
                    &eval_node(*left, graph, eval_context, steady_states),
                    &eval_node(*right, graph, eval_context, steady_states),
                ),
            }
        }
        NodeType::HybridNode(op, var, maybe_domain, child) => {
            // add the variable's domain to the eval context (the variable will be free in the sub-formulae)
            // only do this for quantifiers
            if !matches!(op.clone(), HybridOp::Jump) {
                eval_context
                    .free_var_domains
                    .insert(var.clone(), maybe_domain.clone());
            }

            // two different options depending on if the quantified variable has restricted domain
            let res = match maybe_domain {
                None => eval_hybrid_node(
                    graph,
                    graph,
                    eval_context,
                    steady_states,
                    op.clone(),
                    var.clone(),
                    *child,
                ),
                Some(domain) => {
                    // get a domain set from EvalContext, can use unwrap as it is previously checked
                    let domain_set = eval_context.domain_raw_sets.get(domain.as_str()).unwrap();

                    // check edge case of an empty domain (in that case we cannot restrict the domain,
                    // there would be an error)
                    if domain_set.is_empty() {
                        return match op.clone() {
                            HybridOp::Bind => graph.mk_empty_colored_vertices(),
                            HybridOp::Exists => graph.mk_empty_colored_vertices(),
                            // forall
                            _ => graph.mk_unit_colored_vertices(),
                        };
                    }

                    // restrict the var domain in unit BDD of the graph
                    let var_domain = compute_valid_domain_for_var(graph, domain_set, var.as_str());
                    let restricted_graph = restrict_stg_unit_bdd(graph, &var_domain);
                    eval_hybrid_node(
                        graph,
                        &restricted_graph,
                        eval_context,
                        steady_states,
                        op.clone(),
                        var.clone(),
                        *child,
                    )
                }
            };

            // remove the domain of this (no longer free) variable (only do this for quantifiers)
            if !matches!(op, HybridOp::Jump) {
                eval_context.free_var_domains.remove(&var);
            }
            res
        }
    };

    // save result to cache if needed
    if save_to_cache {
        eval_context
            .cache
            .insert(canonized_formula_with_domains, (result.clone(), renaming));
    }
    result
}

/// Wrapper to recursively evaluate the formula represented by a sub-tree beginning at hybrid node
/// specified by its `operator`, `variable` and `child_node`.
///
/// `graph` gives the context for evaluating the hybrid operator itself, while `graph_to_propagate` should
/// be used to evaluate the successors of the node. The two graphs might be the same. Having two
/// distinct versions allows to evaluate the sub-tree on a different graph with smaller unit bdd,
/// thus limiting the validity domain of the `variable`.
///
/// `eval_context` holds the current version additional data used for optimization, such as
/// pre-computed set of `duplicate` sub-formulae and corresponding `cache` set.
/// The `steady_states` are needed to include self-loops in computing predecessors.
fn eval_hybrid_node(
    graph: &SymbolicAsyncGraph,
    graph_to_propagate: &SymbolicAsyncGraph,
    eval_info: &mut EvalContext,
    steady_states: &GraphColoredVertices,
    operator: HybridOp,
    variable: String,
    child_node: HctlTreeNode,
) -> GraphColoredVertices {
    match operator {
        HybridOp::Bind => eval_bind(
            graph,
            &eval_node(child_node, graph_to_propagate, eval_info, steady_states),
            variable.as_str(),
        ),
        HybridOp::Jump => eval_jump(
            graph,
            &eval_node(child_node, graph_to_propagate, eval_info, steady_states),
            variable.as_str(),
        ),
        HybridOp::Exists => eval_exists(
            graph,
            &eval_node(child_node, graph_to_propagate, eval_info, steady_states),
            variable.as_str(),
        ),
        // evaluate `forall x in A. phi` as `not exists x in A. not phi`
        // do it directly there so that the domain for negations are handled correctly
        HybridOp::Forall => eval_neg(
            graph,
            &eval_exists(
                graph,
                &eval_neg(
                    graph_to_propagate,
                    &eval_node(child_node, graph_to_propagate, eval_info, steady_states),
                ),
                variable.as_str(),
            ),
        ),
    }
}

/// Check whether a node represents the formula pattern for attractors `!{x}: AG EF {x}`.
/// This recognition step is used to later optimize the attractor pattern.
fn is_attractor_pattern(node: &HctlTreeNode) -> bool {
    match &node.node_type {
        NodeType::HybridNode(HybridOp::Bind, var1, None, child1) => match &child1.node_type {
            NodeType::UnaryNode(UnaryOp::AG, child2) => match &child2.node_type {
                NodeType::UnaryNode(UnaryOp::EF, child3) => match &child3.node_type {
                    NodeType::TerminalNode(Atomic::Var(var2)) => var1 == var2,
                    _ => false,
                },
                _ => false,
            },
            _ => false,
        },
        _ => false,
    }
}

/// Check whether a node represents the formula pattern for fixed-points `!{x}: AX {x}`.
/// This recognition step is used to later optimize the fixed-point pattern.
fn is_fixed_point_pattern(node: &HctlTreeNode) -> bool {
    match &node.node_type {
        NodeType::HybridNode(HybridOp::Bind, var1, None, child1) => match &child1.node_type {
            NodeType::UnaryNode(UnaryOp::AX, child2) => match &child2.node_type {
                NodeType::TerminalNode(Atomic::Var(var2)) => var1 == var2,
                _ => false,
            },
            _ => false,
        },
        _ => false,
    }
}

/// Wrapper for the computation of steady states.
/// Steady states are used for explicitly adding self-loops during the EX computation.
/// Can also be used as optimised procedure for formula `!{x}: AX {x}`.
pub fn compute_steady_states(graph: &SymbolicAsyncGraph) -> GraphColoredVertices {
    FixedPoints::symbolic(graph, &graph.mk_unit_colored_vertices())
    /*
    let context = graph.symbolic_context();
    let network = graph.as_network();
    let update_functions: Vec<Bdd> = network
        .as_graph()
        .variables()
        .map(|variable| {
            let regulators = network.regulators(variable);
            let function_is_one = network
                .get_update_function(variable)
                .as_ref()
                .map(|fun| context.mk_fn_update_true(fun))
                .unwrap_or_else(|| context.mk_implicit_function_is_true(variable, &regulators));
            let variable_is_one = context.mk_state_variable_is_true(variable);
            bdd!(variable_is_one <=> function_is_one)
        })
        .collect();

    GraphColoredVertices::new(
        update_functions
            .iter()
            .fold(graph.mk_unit_colored_vertices().into_bdd(), |r, v| r.and(v)),
        context,
    )
     */
}

#[cfg(test)]
mod tests {
    use crate::evaluation::algorithm::{is_attractor_pattern, is_fixed_point_pattern};
    use crate::preprocessing::hctl_tree::*;
    use crate::preprocessing::operator_enums::*;

    #[test]
    /// Test recognition of fixed-point pattern.
    fn test_fixed_point_pattern() {
        let tree = HctlTreeNode::mk_hybrid_node(
            HctlTreeNode::mk_unary_node(HctlTreeNode::mk_var_node("x".to_string()), UnaryOp::AX),
            "x".to_string(),
            None,
            HybridOp::Bind,
        );
        assert!(is_fixed_point_pattern(&tree));
    }

    #[test]
    /// Test recognition of attractor pattern.
    fn test_attractor_pattern() {
        let tree = HctlTreeNode::mk_hybrid_node(
            HctlTreeNode::mk_unary_node(
                HctlTreeNode::mk_unary_node(
                    HctlTreeNode::mk_var_node("x".to_string()),
                    UnaryOp::EF,
                ),
                UnaryOp::AG,
            ),
            "x".to_string(),
            None,
            HybridOp::Bind,
        );
        assert!(is_attractor_pattern(&tree));
    }
}
