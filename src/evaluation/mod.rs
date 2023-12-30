//! Components regarding the evaluation of formulae, including the main model-checking algorithm.

pub mod algorithm;
pub mod eval_context;
pub mod mark_duplicates;

mod canonization;
mod hctl_operators_eval;
mod low_level_operations;
