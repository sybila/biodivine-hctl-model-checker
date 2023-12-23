//! Contains a syntax tree struct for HCTL formulae and functionality regarding the manipulation with it.

use crate::preprocessing::operator_enums::*;
use crate::preprocessing::parser::parse_hctl_tokens;
use crate::preprocessing::tokenizer::HctlToken;

use rand::prelude::StdRng;
use rand::{RngCore, SeedableRng};
use std::cmp;
use std::fmt;

/// Enum of possible node types in a HCTL formula tree.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum NodeType {
    /// Leaf nodes with atomic sub-formulas.
    TerminalNode(Atomic),
    /// Unary nodes with unary operation and a single child.
    UnaryNode(UnaryOp, Box<HctlTreeNode>),
    /// Binary nodes with binary operation and two children.
    BinaryNode(BinaryOp, Box<HctlTreeNode>, Box<HctlTreeNode>),
    /// Hybrid nodes with hybrid operation, variable name, optional variable domain, and a child.
    HybridNode(HybridOp, String, Option<String>, Box<HctlTreeNode>),
}

/// Structure for a HCTL formula syntax tree.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct HctlTreeNode {
    pub formula_str: String,
    pub height: i32,
    pub node_type: NodeType,
}

impl HctlTreeNode {
    /// Parse `tokens` of HCTL formula into an abstract syntax tree using recursive steps.
    /// It is recommended to not use this method for parsing, but rather choose from functions
    /// provided in `preprocessing::parser` module.
    pub fn new(tokens: &[HctlToken]) -> Result<HctlTreeNode, String> {
        parse_hctl_tokens(tokens)
    }

    /// Create a hybrid node from given arguments.
    pub fn mk_hybrid_node(
        child: HctlTreeNode,
        var: String,
        domain: Option<String>,
        op: HybridOp,
    ) -> HctlTreeNode {
        let domain_string = if domain.is_some() {
            format!(" in %{}%", domain.clone().unwrap())
        } else {
            String::new()
        };
        HctlTreeNode {
            formula_str: format!("({op}{{{var}}}{domain_string}: {child})"),
            height: child.height + 1,
            node_type: NodeType::HybridNode(op, var, domain, Box::new(child)),
        }
    }

    /// Create an unary node from given arguments.
    pub fn mk_unary_node(child: HctlTreeNode, op: UnaryOp) -> HctlTreeNode {
        let subform_str = if matches!(op, UnaryOp::Not) {
            format!("({op}{child})")
        } else {
            format!("({op} {child})")
        };
        HctlTreeNode {
            formula_str: subform_str,
            height: child.height + 1,
            node_type: NodeType::UnaryNode(op, Box::new(child)),
        }
    }

    /// Create a binary node from given arguments.
    pub fn mk_binary_node(left: HctlTreeNode, right: HctlTreeNode, op: BinaryOp) -> HctlTreeNode {
        HctlTreeNode {
            formula_str: format!("({left} {op} {right})"),
            height: cmp::max(left.height, right.height) + 1,
            node_type: NodeType::BinaryNode(op, Box::new(left), Box::new(right)),
        }
    }

    /// Create a terminal `variable` node from given arguments.
    pub fn mk_var_node(var_name: String) -> HctlTreeNode {
        HctlTreeNode {
            formula_str: format!("{{{var_name}}}"),
            height: 0,
            node_type: NodeType::TerminalNode(Atomic::Var(var_name)),
        }
    }

    /// Create a terminal `proposition` node from given arguments.
    pub fn mk_prop_node(prop_name: String) -> HctlTreeNode {
        HctlTreeNode {
            formula_str: prop_name.clone(),
            height: 0,
            node_type: NodeType::TerminalNode(Atomic::Prop(prop_name)),
        }
    }

    /// Create a terminal `constant` node (true/false) from given arguments.
    /// `constant` should only be "true" or "false"
    pub fn mk_constant_node(constant_val: bool) -> HctlTreeNode {
        if constant_val {
            HctlTreeNode {
                formula_str: "True".to_string(),
                height: 0,
                node_type: NodeType::TerminalNode(Atomic::True),
            }
        } else {
            HctlTreeNode {
                formula_str: "False".to_string(),
                height: 0,
                node_type: NodeType::TerminalNode(Atomic::False),
            }
        }
    }

    /// Create a terminal `wild-card proposition` node from given arguments.
    pub fn mk_wild_card_node(prop_name: String) -> HctlTreeNode {
        HctlTreeNode {
            formula_str: format!("%{prop_name}%"),
            height: 0,
            node_type: NodeType::TerminalNode(Atomic::WildCardProp(prop_name)),
        }
    }

    /// Create a new random tree containing Boolean operations and propositions. The `tree_height`
    /// is the number of levels in the tree (so the number of leaves will be `2^tree_height`).
    pub fn new_random_boolean(
        tree_height: u8,
        propositions: &Vec<String>,
        seed: u64,
    ) -> HctlTreeNode {
        let num_props = propositions.len() as u32;
        let mut rand = StdRng::seed_from_u64(seed);

        if tree_height == 1 {
            let prop_index = rand.next_u32() % num_props;
            let prop = propositions.get(prop_index as usize).unwrap();
            return HctlTreeNode::mk_prop_node(prop.clone());
        }

        let binary_op = match rand.next_u32() % 5 {
            0 => BinaryOp::And,
            1 => BinaryOp::Or,
            2 => BinaryOp::Xor,
            3 => BinaryOp::Imp,
            _ => BinaryOp::Iff,
        };

        let binary_node = HctlTreeNode::mk_binary_node(
            HctlTreeNode::new_random_boolean(tree_height - 1, propositions, rand.next_u64()),
            HctlTreeNode::new_random_boolean(tree_height - 1, propositions, rand.next_u64()),
            binary_op,
        );

        let negate = rand.next_u32() % 2 == 0;
        if negate {
            HctlTreeNode::mk_unary_node(binary_node, UnaryOp::Not)
        } else {
            binary_node
        }
    }
}

impl HctlTreeNode {
    pub fn as_str(&self) -> &str {
        self.formula_str.as_str()
    }
}

impl fmt::Display for HctlTreeNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.formula_str)
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
        let node1 = HctlTreeNode::new(&tokens1).unwrap();
        let node2 = HctlTreeNode::new(&tokens2).unwrap();

        // test display
        let node1_str = "(!{x}: (3{y}: (@{x}: (((~{y}) & (%subst% & True)) ^ v1))))";
        let node2_str = "(!{x}: (AX {x}))";
        assert_eq!(node1.to_string(), node1_str.to_string());
        assert_eq!(node2.to_string(), node2_str.to_string());
    }
}
