use swc_core::ecma::ast::*;

/// Check if a CallExpr is a `_ts_decorate(...)` call.
pub fn is_ts_decorate_call(call: &CallExpr) -> bool {
    matches!(
        &call.callee,
        Callee::Expr(expr) if matches!(
            &**expr,
            Expr::Ident(ident) if ident.sym.as_ref() == "_ts_decorate"
        )
    )
}

/// Check if a CallExpr is a `_ts_metadata(...)` call.
pub fn is_ts_metadata_call(call: &CallExpr) -> bool {
    matches!(
        &call.callee,
        Callee::Expr(expr) if matches!(
            &**expr,
            Expr::Ident(ident) if ident.sym.as_ref() == "_ts_metadata"
        )
    )
}

/// Check if an arrow function has a simple body (single expression return).
/// Only these are safe to unwrap — complex bodies may have side effects.
///
/// Safe patterns:
/// - `() => String`           (Ident)
/// - `() => SomeModule.Type`  (Member)
/// - `() => [Type]`           (Array)
pub fn is_simple_arrow(arrow: &ArrowExpr) -> bool {
    match &*arrow.body {
        BlockStmtOrExpr::Expr(expr) => matches!(
            &**expr,
            Expr::Ident(_)    // () => String
            | Expr::Member(_) // () => SomeModule.Type
            | Expr::Array(_)  // () => [Type]
        ),
        _ => false,
    }
}
