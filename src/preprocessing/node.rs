//! Contains a syntax tree struct for HCTL formulae and functionality regarding the manipulation with it.

use crate::preprocessing::operator_enums::*;
use crate::preprocessing::parser::parse_hctl_tokens;
use crate::preprocessing::tokenizer::HctlToken;

use std::cmp;
use std::cmp::Ordering;
use std::fmt;

/// Enum of possible node types in a HCTL formula tree.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum NodeType {
    TerminalNode(Atomic),
    UnaryNode(UnaryOp, Box<HctlTreeNode>),
    BinaryNode(BinaryOp, Box<HctlTreeNode>, Box<HctlTreeNode>),
    HybridNode(HybridOp, String, Box<HctlTreeNode>),
}

/// Structure for a HCTL formula syntax tree.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct HctlTreeNode {
    pub subform_str: String,
    pub height: i32,
    pub node_type: NodeType,
}

/// Nodes are ordered by their height.
impl PartialOrd for HctlTreeNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.height.cmp(&other.height))
    }
    fn lt(&self, other: &Self) -> bool {
        self.height.lt(&other.height)
    }
    fn le(&self, other: &Self) -> bool {
        self.height.le(&other.height)
    }
    fn gt(&self, other: &Self) -> bool {
        self.height.gt(&other.height)
    }
    fn ge(&self, other: &Self) -> bool {
        self.height.ge(&other.height)
    }
}

/// Nodes are ordered by their height.
impl Ord for HctlTreeNode {
    fn cmp(&self, other: &Self) -> Ordering {
        self.height.cmp(&other.height)
    }
}

impl HctlTreeNode {
    /// Parse `tokens` of HCTL formula into an abstract syntax tree using recursive steps.
    /// It is recommended to not use this method for parsing, but rather choose from functions
    /// provided in `preprocessing::parser` module.
    pub fn new(tokens: &[HctlToken]) -> Result<HctlTreeNode, String> {
        parse_hctl_tokens(tokens)
    }

    /// Create a hybrid node from given arguments.
    pub fn mk_hybrid_node(child: HctlTreeNode, var: String, op: HybridOp) -> HctlTreeNode {
        HctlTreeNode {
            subform_str: format!("({} {{{}}}: {})", op, var, child.subform_str),
            height: child.height + 1,
            node_type: NodeType::HybridNode(op, var, Box::new(child)),
        }
    }

    /// Create an unary node from given arguments.
    pub fn mk_unary_node(child: HctlTreeNode, op: UnaryOp) -> HctlTreeNode {
        HctlTreeNode {
            subform_str: format!("({} {})", op, child.subform_str),
            height: child.height + 1,
            node_type: NodeType::UnaryNode(op, Box::new(child)),
        }
    }

    /// Create a binary node from given arguments.
    pub fn mk_binary_node(left: HctlTreeNode, right: HctlTreeNode, op: BinaryOp) -> HctlTreeNode {
        HctlTreeNode {
            subform_str: format!("({} {} {})", left.subform_str, op, right.subform_str),
            height: cmp::max(left.height, right.height) + 1,
            node_type: NodeType::BinaryNode(op, Box::new(left), Box::new(right)),
        }
    }

    /// Create a terminal `variable` node from given arguments.
    pub fn mk_var_node(var_name: String) -> HctlTreeNode {
        HctlTreeNode {
            subform_str: format!("{{{var_name}}}"),
            height: 0,
            node_type: NodeType::TerminalNode(Atomic::Var(var_name)),
        }
    }

    /// Create a terminal `proposition` node from given arguments.
    pub fn mk_prop_node(prop_name: String) -> HctlTreeNode {
        HctlTreeNode {
            subform_str: prop_name.clone(),
            height: 0,
            node_type: NodeType::TerminalNode(Atomic::Prop(prop_name)),
        }
    }

    /// Create a terminal `constant` node (true/false) from given arguments.
    /// `constant` should only be "true" or "false"
    pub fn mk_constant_node(constant_val: bool) -> HctlTreeNode {
        if constant_val {
            HctlTreeNode {
                subform_str: "True".to_string(),
                height: 0,
                node_type: NodeType::TerminalNode(Atomic::True),
            }
        } else {
            HctlTreeNode {
                subform_str: "False".to_string(),
                height: 0,
                node_type: NodeType::TerminalNode(Atomic::False),
            }
        }
    }

    /// Create a terminal `wild-card proposition` node from given arguments.
    pub fn mk_wild_card_node(prop_name: String) -> HctlTreeNode {
        HctlTreeNode {
            subform_str: format!("%{prop_name}%"),
            height: 0,
            node_type: NodeType::TerminalNode(Atomic::WildCardProp(prop_name)),
        }
    }
}

impl fmt::Display for HctlTreeNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.subform_str)
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

        // higher tree should be greater
        assert!(node1 > node2);
        assert!(node2 <= node1);

        // test display
        let node1_str = "(Bind {x}: (Exists {y}: (Jump {x}: (((~ {y}) & (%subst% & True)) ^ v1))))";
        let node2_str = "(Bind {x}: (Ax {x}))";
        assert_eq!(node1.to_string(), node1_str.to_string());
        assert_eq!(node2.to_string(), node2_str.to_string());
    }
}
