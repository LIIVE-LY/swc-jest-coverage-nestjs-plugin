use swc_core::ecma::ast::*;
use swc_core::ecma::visit::{VisitMut, VisitMutWith};

use crate::detection::*;
use crate::transforms::*;
use crate::Config;

pub struct DecoratorCoverageVisitor {
    config: Config,
}

impl DecoratorCoverageVisitor {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}

impl VisitMut for DecoratorCoverageVisitor {
    fn visit_mut_call_expr(&mut self, call: &mut CallExpr) {
        if is_ts_decorate_call(call) {
            if let Some(ExprOrSpread { expr, .. }) = call.args.first_mut() {
                if let Expr::Array(array) = &mut **expr {
                    if self.config.strip_metadata.unwrap_or(true) {
                        strip_metadata_calls(&mut array.elems);
                    }

                    if self.config.unwrap_decorator_arrows.unwrap_or(true) {
                        unwrap_decorator_arrow_args(&mut array.elems);
                    }

                    if self.config.unwrap_type_arrows.unwrap_or(true) {
                        unwrap_type_arrow_props(&mut array.elems);
                    }
                }
            }
        }

        call.visit_mut_children_with(self);
    }
}
