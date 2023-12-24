use crate::preprocessing::node::HctlTreeNode;
use biodivine_lib_param_bn::BooleanNetwork;

/// Generate set of `num` syntactic trees for random boolean expressions.
/// The trees are almost full binary trees with given `height` (but there are random negation nodes between the
/// binary nodes).
/// The tree has `2^height` leaf nodes that take values from propositions of `bn`.
/// The `seed` specifies the seed for the initial tree, the seed for subsequent trees always increments.
pub(super) fn make_random_boolean_trees(
    num: u64,
    height: u8,
    bn: &BooleanNetwork,
    seed: u64,
) -> Vec<HctlTreeNode> {
    let props: Vec<String> = bn
        .variables()
        .map(|var_id| bn.get_variable_name(var_id).clone())
        .collect();

    (0..num)
        .map(|i| HctlTreeNode::new_random_boolean(height, &props, seed + i))
        .collect()
}
