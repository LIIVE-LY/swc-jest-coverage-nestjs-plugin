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

/// Check if an expression is `typeof X === "undefined"` or `typeof X.Y === "undefined"`.
///
/// Matches both operand orders (typeof on left or right).
pub fn is_typeof_undefined_check(expr: &Expr) -> bool {
    if let Expr::Bin(bin) = expr {
        if bin.op != BinaryOp::EqEqEq {
            return false;
        }
        is_typeof_undefined_pair(&bin.left, &bin.right)
            || is_typeof_undefined_pair(&bin.right, &bin.left)
    } else {
        false
    }
}

fn is_typeof_undefined_pair(maybe_typeof: &Expr, maybe_lit: &Expr) -> bool {
    let is_typeof = matches!(
        maybe_typeof,
        Expr::Unary(UnaryExpr { op: UnaryOp::TypeOf, .. })
    );
    let is_undefined_str = matches!(
        maybe_lit,
        Expr::Lit(Lit::Str(s)) if &*s.value == "undefined"
    );
    is_typeof && is_undefined_str
}

/// Check if a conditional expression is a typeof guard pattern:
///
///   `typeof X === "undefined" ? Object : X`                           (simple)
///   `typeof X === "undefined" || typeof X.Y === "undefined" ? Object : X.Y`  (chained)
///
/// The test must be a single typeof check or a `||` chain of typeof checks.
/// The consequent must be the `Object` identifier.
pub fn is_typeof_guard_conditional(cond: &CondExpr) -> bool {
    let test_is_typeof_chain = is_typeof_or_chain(&cond.test);
    let consequent_is_object = matches!(
        &*cond.cons,
        Expr::Ident(ident) if ident.sym.as_ref() == "Object"
    );
    test_is_typeof_chain && consequent_is_object
}

fn is_typeof_or_chain(expr: &Expr) -> bool {
    match expr {
        _ if is_typeof_undefined_check(expr) => true,
        Expr::Bin(bin) if bin.op == BinaryOp::LogicalOr => {
            is_typeof_or_chain(&bin.left) && is_typeof_or_chain(&bin.right)
        }
        _ => false,
    }
}

/// Check if a `_ts_decorate` call is decorating a constructor (vs a method/property).
///
/// Constructor: `_ts_decorate([...], ClassName, undefined, null)`
/// Method:      `_ts_decorate([...], ClassName.prototype, "methodName", null)`
///
/// The third argument (index 2) is `undefined` for constructors.
pub fn is_constructor_decorate(call: &CallExpr) -> bool {
    match call.args.get(2) {
        None => true,
        Some(ExprOrSpread { expr, .. }) => {
            matches!(&**expr, Expr::Ident(ident) if ident.sym.as_ref() == "undefined")
        }
    }
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
