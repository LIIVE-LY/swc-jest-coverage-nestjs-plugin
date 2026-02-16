use swc_core::ecma::{ast::Program, visit::visit_mut_pass};
use swc_core::plugin::{plugin_transform, proxies::TransformPluginProgramMetadata};
use serde::Deserialize;

mod detection;
mod transforms;
pub mod visitor;

use visitor::DecoratorCoverageVisitor;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    /// Unwrap simple arrow functions in decorator type params (default: true)
    /// e.g., `type: () => String` -> `type: String`
    pub unwrap_type_arrows: Option<bool>,
    /// Strip _ts_metadata calls from _ts_decorate arrays (default: false)
    /// Removes design:type, design:paramtypes, design:returntype
    pub strip_metadata: Option<bool>,
    /// Unwrap arrow function arguments to decorator calls (default: true)
    /// e.g., `ResolveField(() => String)` -> `ResolveField(String)`
    pub unwrap_decorator_arrows: Option<bool>,
    /// Simplify typeof guard conditionals inside _ts_metadata args to `Object` (default: true)
    /// e.g., `typeof Express === "undefined" || ... ? Object : Express.Multer.File` -> `Object`
    pub simplify_metadata_typeofs: Option<bool>,
    /// Simplify typeof guard conditionals inside _ts_metadata("design:type", ...) args (default: false)
    /// Only enable if your design:type metadata contains member-expression types (e.g. mongoose.Types.ObjectId)
    pub simplify_design_type_typeofs: Option<bool>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            unwrap_type_arrows: Some(true),
            strip_metadata: Some(false),
            unwrap_decorator_arrows: Some(true),
            simplify_metadata_typeofs: Some(true),
            simplify_design_type_typeofs: Some(false),
        }
    }
}

#[plugin_transform]
pub fn process_transform(
    program: Program,
    metadata: TransformPluginProgramMetadata,
) -> Program {
    let config: Config = serde_json::from_str(
        &metadata
            .get_transform_plugin_config()
            .unwrap_or_default(),
    )
    .unwrap_or_default();

    program.apply(visit_mut_pass(DecoratorCoverageVisitor::new(config)))
}
