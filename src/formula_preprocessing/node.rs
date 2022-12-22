//! Contains a syntax tree struct for HCTL formulae and functionality regarding the manipulation with it.

use crate::formula_preprocessing::operator_enums::*;

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

impl Default for HctlTreeNode {
    fn default() -> Self {
        Self::new()
    }
}

impl HctlTreeNode {
    /// Create a default node - the `True` constant (terminal) node.
    pub fn new() -> Self {
        Self {
            subform_str: "True".to_string(),
            height: 0,
            node_type: NodeType::TerminalNode(Atomic::True),
        }
    }
}

impl fmt::Display for HctlTreeNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.subform_str)
    }
}

/// Create a hybrid node from given arguments.
pub fn create_hybrid(child: Box<HctlTreeNode>, var: String, op: HybridOp) -> HctlTreeNode {
    HctlTreeNode {
        subform_str: format!("({} {{{}}}: {})", op, var, child.subform_str),
        height: child.height + 1,
        node_type: NodeType::HybridNode(op, var, child),
    }
}

/// Create an unary node from given arguments.
pub fn create_unary(child: Box<HctlTreeNode>, op: UnaryOp) -> HctlTreeNode {
    HctlTreeNode {
        subform_str: format!("({} {})", op, child.subform_str),
        height: child.height + 1,
        node_type: NodeType::UnaryNode(op, child),
    }
}

/// Create a binary node from given arguments.
pub fn create_binary(
    left: Box<HctlTreeNode>,
    right: Box<HctlTreeNode>,
    op: BinaryOp,
) -> HctlTreeNode {
    HctlTreeNode {
        subform_str: format!("({} {} {})", left.subform_str, op, right.subform_str),
        height: cmp::max(left.height, right.height) + 1,
        node_type: NodeType::BinaryNode(op, left, right),
    }
}
