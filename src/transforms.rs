use swc_core::common::DUMMY_SP;
use swc_core::ecma::ast::*;

use crate::detection::*;

/// Remove `_ts_metadata(...)` calls from a decorators array.
///
/// Before: `[decorator1, _ts_metadata("design:type", Function), decorator2]`
/// After:  `[decorator1, decorator2]`
pub fn strip_metadata_calls(elems: &mut Vec<Option<ExprOrSpread>>) {
    elems.retain(|elem| {
        if let Some(ExprOrSpread { expr, .. }) = elem {
            if let Expr::Call(call) = &**expr {
                return !is_ts_metadata_call(call);
            }
        }
        true
    });
}

/// Unwrap arrow function arguments passed directly to decorator calls.
///
/// Before: `(0, _graphql.ResolveField)(() => String)`
/// After:  `(0, _graphql.ResolveField)(String)`
///
/// Also handles nested calls like `_ts_param(0, (0, _graphql.Args)(() => String))`.
pub fn unwrap_decorator_arrow_args(elems: &mut Vec<Option<ExprOrSpread>>) {
    for elem in elems.iter_mut().flatten() {
        unwrap_arrows_in_call_args(&mut elem.expr);
    }
}

fn unwrap_arrows_in_call_args(expr: &mut Box<Expr>) {
    if let Expr::Call(call) = &mut **expr {
        for arg in &mut call.args {
            // Recurse into nested calls first (e.g., _ts_param wrapping another call)
            unwrap_arrows_in_call_args(&mut arg.expr);

            // Then unwrap arrow functions at this level
            if let Expr::Arrow(arrow) = &*arg.expr {
                if is_simple_arrow(arrow) {
                    if let BlockStmtOrExpr::Expr(body) = &*arrow.body {
                        arg.expr = body.clone();
                    }
                }
            }
        }
    }
}

/// Unwrap arrow functions in `type:` key-value properties within
/// decorator option objects.
///
/// Before: `_ts_param(0, (0, _graphql.Args)('id', { type: () => String }))`
/// After:  `_ts_param(0, (0, _graphql.Args)('id', { type: String }))`
pub fn unwrap_type_arrow_props(elems: &mut Vec<Option<ExprOrSpread>>) {
    for elem in elems.iter_mut().flatten() {
        unwrap_type_props_in_expr(&mut elem.expr);
    }
}

fn unwrap_type_props_in_expr(expr: &mut Box<Expr>) {
    match &mut **expr {
        Expr::Call(call) => {
            for arg in &mut call.args {
                unwrap_type_props_in_expr(&mut arg.expr);
            }
        }
        Expr::Object(obj) => {
            for prop in &mut obj.props {
                if let PropOrSpread::Prop(prop) = prop {
                    if let Prop::KeyValue(kv) = &mut **prop {
                        if is_type_key(&kv.key) {
                            if let Expr::Arrow(arrow) = &*kv.value {
                                if is_simple_arrow(arrow) {
                                    if let BlockStmtOrExpr::Expr(body) = &*arrow.body {
                                        kv.value = body.clone();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }
}

fn is_type_key(key: &PropName) -> bool {
    match key {
        PropName::Ident(id) => id.sym.as_ref() == "type",
        PropName::Str(s) => &*s.value == "type",
        _ => false,
    }
}

/// Simplify typeof guard conditionals inside `_ts_metadata("design:paramtypes", ...)` arguments.
///
/// Only targets `design:paramtypes` — other keys like `design:type` are preserved
/// because `@nestjs/mongoose` `@Prop()` uses `design:type` to infer schema types.
///
/// Before: `_ts_metadata("design:paramtypes", [typeof X === "undefined" ? Object : X])`
/// After:  `_ts_metadata("design:paramtypes", [Object])`
pub fn simplify_metadata_typeof_guards(elems: &mut Vec<Option<ExprOrSpread>>) {
    for elem in elems.iter_mut().flatten() {
        if let Expr::Call(call) = &mut *elem.expr {
            if !is_ts_metadata_call(call) {
                continue;
            }

            if !is_paramtypes_metadata(call) {
                continue;
            }

            for arg in call.args.iter_mut().skip(1) {
                simplify_typeofs_in_expr(&mut arg.expr);
            }
        }
    }
}

fn is_paramtypes_metadata(call: &CallExpr) -> bool {
    matches!(
        call.args.first(),
        Some(ExprOrSpread { expr, .. })
            if matches!(&**expr, Expr::Lit(Lit::Str(s)) if &*s.value == "design:paramtypes")
    )
}

fn simplify_typeofs_in_expr(expr: &mut Box<Expr>) {
    match &**expr {
        Expr::Cond(cond) if is_typeof_guard_conditional(cond) => {
            *expr = Box::new(Expr::Ident(Ident::new_no_ctxt(
                "Object".into(),
                DUMMY_SP,
            )));
        }
        Expr::Array(_) => {
            if let Expr::Array(array) = &mut **expr {
                for elem in array.elems.iter_mut().flatten() {
                    simplify_typeofs_in_expr(&mut elem.expr);
                }
            }
        }
        _ => {}
    }
}
