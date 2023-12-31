//! Components regarding the evaluation of formulae, including the main model-checking algorithm.

use biodivine_lib_param_bn::symbolic_async_graph::GraphColoredVertices;
use std::collections::{BTreeMap, HashMap};

pub mod algorithm;
pub mod eval_context;
pub mod mark_duplicates;

mod canonization;
mod hctl_operators_eval;
mod low_level_operations;

/// Shorthand for mapping of free variables to (optional) labels of their domain.
pub type VarDomainMap = BTreeMap<String, Option<String>>;

/// Shorthand for sub-formula with mapping of its free variables (optional) labels of their domain .
pub type FormulaWithDomains = (String, VarDomainMap);

/// Shorthand for mapping between variable names (usually from original to canonical form).
pub type VarRenameMap = HashMap<String, String>;

/// Shorthand for mapping between string labels (domain label, proposition, formula) and the corresponding
/// set it evaluates to.
pub type LabelToSetMap = HashMap<String, GraphColoredVertices>;
