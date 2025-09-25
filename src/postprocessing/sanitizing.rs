//! Contains operations to sanitize bdds of their additional symbolic variables,
//! making them compatible with remaining biodivine libraries.
use biodivine_lib_param_bn::symbolic_async_graph::{
    GraphColoredVertices, GraphColors, GraphVertices, SymbolicAsyncGraph,
};

/// Sanitize underlying BDD of a given coloured state set by removing the symbolic variables
/// that were used for representing HCTL state-variables. At the moment, we remove all symbolic
/// variables.
pub fn sanitize_colored_vertices(
    stg: &SymbolicAsyncGraph,
    colored_vertices: &GraphColoredVertices,
) -> GraphColoredVertices {
    let canonical_context = stg.symbolic_context().as_canonical_context();
    let sanitized_result_bdd = canonical_context
        .transfer_from(colored_vertices.as_bdd(), stg.symbolic_context())
        .unwrap();
    GraphColoredVertices::new(sanitized_result_bdd, &canonical_context)
}

/// Sanitize underlying BDD of a given colour set by removing the symbolic variables
/// that were used for representing HCTL state-variables. At the moment, we remove all symbolic
/// variables.
pub fn sanitize_colors(stg: &SymbolicAsyncGraph, colors: &GraphColors) -> GraphColors {
    let canonical_context = stg.symbolic_context().as_canonical_context();
    let sanitized_result_bdd = canonical_context
        .transfer_from(colors.as_bdd(), stg.symbolic_context())
        .unwrap();
    GraphColors::new(sanitized_result_bdd, &canonical_context)
}

/// Sanitize underlying BDD of a given set of vertices by removing the symbolic variables
/// that were used for representing HCTL state-variables. At the moment, we remove all symbolic
/// variables.
pub fn sanitize_vertices(stg: &SymbolicAsyncGraph, vertices: &GraphVertices) -> GraphVertices {
    let canonical_context = stg.symbolic_context().as_canonical_context();
    let sanitized_result_bdd = canonical_context
        .transfer_from(vertices.as_bdd(), stg.symbolic_context())
        .unwrap();
    GraphVertices::new(sanitized_result_bdd, &canonical_context)
}

#[cfg(test)]
mod tests {
    use crate::evaluation::algorithm::compute_steady_states;
    use crate::mc_utils::get_extended_symbolic_graph;
    use crate::postprocessing::sanitizing::{
        sanitize_colored_vertices, sanitize_colors, sanitize_vertices,
    };
    use biodivine_lib_param_bn::BooleanNetwork;
    use biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph;

    const MODEL: &str = r"
    targets,factors
    A, B | C
    B, C
    C, A
    ";

    #[test]
    fn test_sanitize() {
        let bn = BooleanNetwork::try_from_bnet(MODEL).unwrap();
        let canonical_stg = SymbolicAsyncGraph::new(&bn).unwrap();
        let extended_stg = get_extended_symbolic_graph(&bn, 1).unwrap();

        let fp_canonical = compute_steady_states(&canonical_stg);
        let fp_extended = compute_steady_states(&extended_stg);
        assert_ne!(
            fp_canonical.as_bdd().to_string(),
            fp_extended.as_bdd().to_string()
        );
        assert_ne!(
            fp_canonical.colors().as_bdd().to_string(),
            fp_extended.colors().as_bdd().to_string()
        );
        assert_ne!(
            fp_canonical.vertices().as_bdd().to_string(),
            fp_extended.vertices().as_bdd().to_string()
        );

        let fp_extended_sanitized = sanitize_colored_vertices(&extended_stg, &fp_extended);
        let fp_colors_sanitized = sanitize_colors(&extended_stg, &fp_extended.colors());
        let fp_vertices_sanitized = sanitize_vertices(&extended_stg, &fp_extended.vertices());

        assert_eq!(
            fp_canonical.as_bdd().to_string(),
            fp_extended_sanitized.as_bdd().to_string()
        );
        assert_eq!(
            fp_canonical.colors().as_bdd().to_string(),
            fp_colors_sanitized.as_bdd().to_string()
        );
        assert_eq!(
            fp_canonical.vertices().as_bdd().to_string(),
            fp_vertices_sanitized.as_bdd().to_string()
        );
    }
}
