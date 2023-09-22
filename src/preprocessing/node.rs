//! A syntax tree struct for HCTL formulae and functionality for its manipulation.

use crate::preprocessing::operator_enums::*;
use crate::preprocessing::parser::parse_hctl_tokens;
use crate::preprocessing::tokenizer::HctlToken;

use std::cmp;
use std::cmp::Ordering;
use std::fmt;

/// Enum of possible node data types in a HCTL formula syntax tree.
///
/// In particular, a node type can be:
///     - A "terminal" node, containing a single atomic proposition.
///     - A "unary" node, with a `UnaryOp` and a sub-formula.
///     - A "binary" node, with a `BinaryOp` and two sub-formulae.
///     - A "hybrid" node, with a `HybridOp`, a string variable name, and a sub-formula.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum NodeType {
    Terminal(Atomic),
    Unary(UnaryOp, Box<HctlTreeNode>),
    Binary(BinaryOp, Box<HctlTreeNode>, Box<HctlTreeNode>),
    Hybrid(HybridOp, String, Box<HctlTreeNode>),
}

/// A single node in a syntax tree of a HCTL formula.
///
/// Each node tracks its:
///     - `height`; A positive integer starting from 0 (for atomic propositions).
///     - `node_type`; A collection of node data represented through `NodeType`.
///     - `subform_str`; A canonical string representation of the HCTL formula, which is
///     used for uniqueness testing during simplification and canonization.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct HctlTreeNode {
    pub subform_str: String,
    pub height: u32,
    pub node_type: NodeType,
}

/// Nodes are ordered by their height, with atomic propositions being the "smallest".
impl PartialOrd for HctlTreeNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.height.partial_cmp(&other.height)
    }
}

/// Nodes are ordered by their height, with atomic propositions being the "smallest".
///
/// Note that while this sort is "total" in the sense that every pair of nodes can be compared,
/// there are many "semantically equivalent" nodes that have the same height.
impl Ord for HctlTreeNode {
    fn cmp(&self, other: &Self) -> Ordering {
        self.height.cmp(&other.height)
    }
}

impl fmt::Display for HctlTreeNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.subform_str)
    }
}

impl HctlTreeNode {
    /// "Parse" a new [HctlTreeNode] from a list of [HctlToken] objects.
    ///
    /// Note that this is a very "low-level" function. Unless you know what you are doing,
    /// you should probably use some of the functions in [crate::preprocessing::parser] instead.
    pub fn from_tokens(tokens: &[HctlToken]) -> Result<HctlTreeNode, String> {
        parse_hctl_tokens(tokens)
    }

    /// Create a "hybrid" [HctlTreeNode] from the given arguments.
    ///
    /// See also [NodeType::Hybrid].
    pub fn mk_hybrid(child: HctlTreeNode, var: &str, op: HybridOp) -> HctlTreeNode {
        HctlTreeNode {
            subform_str: format!("({}{{{}}}: {})", op, var, child.subform_str),
            height: child.height + 1,
            node_type: NodeType::Hybrid(op, var.to_string(), Box::new(child)),
        }
    }

    /// Create a "hybrid" [HctlTreeNode] from the given arguments.
    ///
    /// See also [NodeType::Unary].
    pub fn mk_unary(child: HctlTreeNode, op: UnaryOp) -> HctlTreeNode {
        HctlTreeNode {
            subform_str: format!("({} {})", op, child.subform_str),
            height: child.height + 1,
            node_type: NodeType::Unary(op, Box::new(child)),
        }
    }

    /// Create a "binary" [HctlTreeNode] from the given arguments.
    ///
    /// See also [NodeType::Binary].
    pub fn mk_binary(left: HctlTreeNode, right: HctlTreeNode, op: BinaryOp) -> HctlTreeNode {
        HctlTreeNode {
            subform_str: format!("({} {} {})", left.subform_str, op, right.subform_str),
            height: cmp::max(left.height, right.height) + 1,
            node_type: NodeType::Binary(op, Box::new(left), Box::new(right)),
        }
    }

    /// Create a [HctlTreeNode] representing a Boolean constant.
    ///
    /// See also [NodeType::Terminal] and [Atomic::True] / [Atomic::False].
    pub fn mk_constant(constant_val: bool) -> HctlTreeNode {
        Self::mk_atom(Atomic::from(constant_val))
    }

    /// Create a [HctlTreeNode] representing a HCTL variable proposition.
    ///
    /// See also [NodeType::Terminal] and [Atomic::Var].
    pub fn mk_var(var_name: &str) -> HctlTreeNode {
        Self::mk_atom(Atomic::Var(var_name.to_string()))
    }

    /// Create a [HctlTreeNode] representing an atomic proposition.
    ///
    /// See also [NodeType::Terminal] and [Atomic::Prop].
    pub fn mk_prop(prop_name: &str) -> HctlTreeNode {
        Self::mk_atom(Atomic::Prop(prop_name.to_string()))
    }

    /// Create a [HctlTreeNode] representing a "wild-card" proposition.
    ///
    /// See also [NodeType::Terminal] and [Atomic::WildCardProp].
    pub fn mk_wild_card(prop_name: &str) -> HctlTreeNode {
        Self::mk_atom(Atomic::WildCardProp(prop_name.to_string()))
    }

    /// A helper function which creates a new [HctlTreeNode] for the given [Atomic] value.
    fn mk_atom(atom: Atomic) -> HctlTreeNode {
        HctlTreeNode {
            subform_str: atom.to_string(),
            height: 0,
            node_type: NodeType::Terminal(atom),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::preprocessing::node::HctlTreeNode;
    use crate::preprocessing::tokenizer::{try_tokenize_extended_formula, try_tokenize_formula};

    #[test]
    /// Test creation, ordering, and display of HCTL tree nodes.
    fn test_tree_nodes() {
        // formula containing all kinds of operators and terminals (even wild-card propositions)
        let formula1 = "!{x}: 3{y}: (@{x}: ~{y} & %subst% & True ^ v1)".to_string();
        // much shorter formula to generate shallower tree
        let formula2 = "!{x}: AX {x}".to_string();

        // test `new` function works
        let tokens1 = try_tokenize_extended_formula(formula1).unwrap();
        let tokens2 = try_tokenize_formula(formula2).unwrap();
        let node1 = HctlTreeNode::from_tokens(&tokens1).unwrap();
        let node2 = HctlTreeNode::from_tokens(&tokens2).unwrap();

        // higher tree should be greater
        assert!(node1 > node2);
        assert!(node2 <= node1);

        // Test display:
        let node1_str = "(!{x}: (3{y}: (@{x}: (((~ {y}) & (%subst% & True)) ^ v1))))";
        let node2_str = "(!{x}: (AX {x}))";
        assert_eq!(node1.to_string(), node1_str.to_string());
        assert_eq!(node2.to_string(), node2_str.to_string());

        // Check that display output can be parsed (note that tokens could be different due
        // to extra parentheses).
        let tokens11 = try_tokenize_extended_formula(node1.to_string()).unwrap();
        let tokens22 = try_tokenize_formula(node2.to_string()).unwrap();
        let node11 = HctlTreeNode::from_tokens(&tokens11).unwrap();
        let node22 = HctlTreeNode::from_tokens(&tokens22).unwrap();
        assert_eq!(node1, node11);
        assert_eq!(node2, node22);
    }
}
