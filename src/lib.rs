use swc_core::ecma::{ast::Program, visit::visit_mut_pass};
use swc_core::plugin::{
    metadata::TransformPluginMetadataContextKind, plugin_transform,
    proxies::TransformPluginProgramMetadata,
};
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

impl Config {
    /// Merge an override on top of self. Override's Some values win; None inherits from self.
    pub fn merge_override(&self, override_config: &Config) -> Config {
        Config {
            unwrap_type_arrows: override_config.unwrap_type_arrows.or(self.unwrap_type_arrows),
            strip_metadata: override_config.strip_metadata.or(self.strip_metadata),
            unwrap_decorator_arrows: override_config
                .unwrap_decorator_arrows
                .or(self.unwrap_decorator_arrows),
            simplify_metadata_typeofs: override_config
                .simplify_metadata_typeofs
                .or(self.simplify_metadata_typeofs),
            simplify_design_type_typeofs: override_config
                .simplify_design_type_typeofs
                .or(self.simplify_design_type_typeofs),
        }
    }
}

/// Top-level config shape. Backward compatible via #[serde(flatten)].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginConfig {
    /// Base config options (flattened for backward compat with existing JSON shape)
    #[serde(flatten)]
    pub base: Config,

    /// Per-file override rules, applied in order (later wins)
    #[serde(default)]
    pub overrides: Vec<OverrideRule>,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            base: Config::default(),
            overrides: Vec::new(),
        }
    }
}

impl PluginConfig {
    /// Resolve the final Config for a given filename by applying all matching overrides.
    pub fn resolve(&self, filename: Option<&str>) -> Config {
        let mut config = self.base.clone();

        let filename = match filename {
            Some(f) if !f.is_empty() => f,
            _ => return config, // No filename → base config only
        };

        for rule in &self.overrides {
            if rule.matches(filename) {
                config = config.merge_override(&rule.config);
            }
        }

        config
    }
}

/// A single override rule: glob patterns + config options to apply when matched.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverrideRule {
    /// Glob patterns to match against the filename. Any match triggers the override.
    pub files: Vec<String>,
    /// Config options to override. Only specified (Some) fields are applied.
    pub config: Config,
}

impl OverrideRule {
    /// Check if any of this rule's patterns match the given filename.
    fn matches(&self, filename: &str) -> bool {
        let normalized: std::borrow::Cow<str> = if filename.contains('\\') {
            std::borrow::Cow::Owned(filename.replace('\\', "/"))
        } else {
            std::borrow::Cow::Borrowed(filename)
        };

        self.files
            .iter()
            .any(|pattern| glob_match::glob_match(pattern, &normalized))
    }
}

#[plugin_transform]
pub fn process_transform(
    program: Program,
    metadata: TransformPluginProgramMetadata,
) -> Program {
    let plugin_config: PluginConfig = serde_json::from_str(
        &metadata
            .get_transform_plugin_config()
            .unwrap_or_default(),
    )
    .unwrap_or_default();

    let filename = metadata.get_context(&TransformPluginMetadataContextKind::Filename);
    let config = plugin_config.resolve(filename.as_deref());

    program.apply(visit_mut_pass(DecoratorCoverageVisitor::new(config)))
}
