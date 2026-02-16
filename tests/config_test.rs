use swc_jest_coverage_nestjs_plugin::{Config, OverrideRule, PluginConfig};

fn base_config(simplify_design_type: bool) -> PluginConfig {
    PluginConfig {
        base: Config {
            simplify_design_type_typeofs: Some(simplify_design_type),
            ..Config::default()
        },
        overrides: Vec::new(),
    }
}

fn override_rule(patterns: &[&str], simplify_design_type: Option<bool>) -> OverrideRule {
    OverrideRule {
        files: patterns.iter().map(|s| s.to_string()).collect(),
        config: Config {
            unwrap_type_arrows: None,
            strip_metadata: None,
            unwrap_decorator_arrows: None,
            simplify_metadata_typeofs: None,
            simplify_design_type_typeofs: simplify_design_type,
        },
    }
}

// --- Basic resolution ---

#[test]
fn empty_overrides_returns_base() {
    let pc = base_config(false);
    let resolved = pc.resolve(Some("/src/models/venue.model.ts"));
    assert_eq!(resolved.simplify_design_type_typeofs, Some(false));
}

#[test]
fn override_matches_and_merges() {
    let mut pc = base_config(false);
    pc.overrides
        .push(override_rule(&["**/*.model.*"], Some(true)));
    let resolved = pc.resolve(Some("/src/models/venue.model.ts"));
    assert_eq!(resolved.simplify_design_type_typeofs, Some(true));
    // Other fields inherited from base
    assert_eq!(resolved.unwrap_type_arrows, Some(true));
}

#[test]
fn no_filename_uses_base() {
    let mut pc = base_config(false);
    pc.overrides
        .push(override_rule(&["**/*.model.*"], Some(true)));
    let resolved = pc.resolve(None);
    assert_eq!(resolved.simplify_design_type_typeofs, Some(false));
}

#[test]
fn empty_filename_uses_base() {
    let mut pc = base_config(false);
    pc.overrides
        .push(override_rule(&["**/*.model.*"], Some(true)));
    let resolved = pc.resolve(Some(""));
    assert_eq!(resolved.simplify_design_type_typeofs, Some(false));
}

#[test]
fn non_matching_file_uses_base() {
    let mut pc = base_config(false);
    pc.overrides
        .push(override_rule(&["**/*.model.*"], Some(true)));
    let resolved = pc.resolve(Some("/src/services/venue.service.ts"));
    assert_eq!(resolved.simplify_design_type_typeofs, Some(false));
}

// --- Override precedence ---

#[test]
fn later_override_wins() {
    let mut pc = base_config(false);
    pc.overrides
        .push(override_rule(&["**/*.model.*"], Some(true)));
    pc.overrides
        .push(override_rule(&["**/venue.model.*"], Some(false)));
    let resolved = pc.resolve(Some("/src/models/venue.model.ts"));
    // Both match, but the later override (false) wins
    assert_eq!(resolved.simplify_design_type_typeofs, Some(false));
}

#[test]
fn override_explicit_false_beats_base_true() {
    let mut pc = PluginConfig {
        base: Config {
            simplify_metadata_typeofs: Some(true),
            ..Config::default()
        },
        overrides: Vec::new(),
    };
    pc.overrides.push(OverrideRule {
        files: vec!["**/special.*".to_string()],
        config: Config {
            unwrap_type_arrows: None,
            strip_metadata: None,
            unwrap_decorator_arrows: None,
            simplify_metadata_typeofs: Some(false),
            simplify_design_type_typeofs: None,
        },
    });
    let resolved = pc.resolve(Some("/src/special.ts"));
    assert_eq!(resolved.simplify_metadata_typeofs, Some(false));
}

#[test]
fn multiple_overrides_only_matching_one_applies() {
    let mut pc = base_config(false);
    pc.overrides
        .push(override_rule(&["**/models/**"], Some(true)));
    pc.overrides.push(OverrideRule {
        files: vec!["**/services/**".to_string()],
        config: Config {
            unwrap_type_arrows: None,
            strip_metadata: Some(true),
            unwrap_decorator_arrows: None,
            simplify_metadata_typeofs: None,
            simplify_design_type_typeofs: None,
        },
    });
    // Only first override matches
    let resolved = pc.resolve(Some("/src/models/venue.model.ts"));
    assert_eq!(resolved.simplify_design_type_typeofs, Some(true));
    assert_eq!(resolved.strip_metadata, Some(false)); // inherited from base, not second override
}

#[test]
fn multiple_overrides_with_different_fields() {
    let mut pc = base_config(false);
    // First override enables simplify_design_type_typeofs
    pc.overrides
        .push(override_rule(&["**/models/**"], Some(true)));
    // Second override enables strip_metadata for models
    pc.overrides.push(OverrideRule {
        files: vec!["**/models/**".to_string()],
        config: Config {
            unwrap_type_arrows: None,
            strip_metadata: Some(true),
            unwrap_decorator_arrows: None,
            simplify_metadata_typeofs: None,
            simplify_design_type_typeofs: None,
        },
    });
    let resolved = pc.resolve(Some("/src/models/venue.model.ts"));
    // Both overrides applied
    assert_eq!(resolved.simplify_design_type_typeofs, Some(true));
    assert_eq!(resolved.strip_metadata, Some(true));
}

// --- Merge behavior ---

#[test]
fn override_only_specified_fields() {
    let mut pc = base_config(false);
    pc.overrides
        .push(override_rule(&["**/*.model.*"], Some(true)));
    let resolved = pc.resolve(Some("/src/models/venue.model.ts"));
    assert_eq!(resolved.simplify_design_type_typeofs, Some(true));
    assert_eq!(resolved.unwrap_type_arrows, Some(true));
    assert_eq!(resolved.strip_metadata, Some(false));
    assert_eq!(resolved.unwrap_decorator_arrows, Some(true));
    assert_eq!(resolved.simplify_metadata_typeofs, Some(true));
}

#[test]
fn override_with_all_none_fields_is_noop() {
    let mut pc = base_config(true);
    pc.overrides.push(OverrideRule {
        files: vec!["**/*".to_string()],
        config: Config {
            unwrap_type_arrows: None,
            strip_metadata: None,
            unwrap_decorator_arrows: None,
            simplify_metadata_typeofs: None,
            simplify_design_type_typeofs: None,
        },
    });
    let resolved = pc.resolve(Some("/src/anything.ts"));
    assert_eq!(resolved.unwrap_type_arrows, Some(true));
    assert_eq!(resolved.strip_metadata, Some(false));
    assert_eq!(resolved.unwrap_decorator_arrows, Some(true));
    assert_eq!(resolved.simplify_metadata_typeofs, Some(true));
    assert_eq!(resolved.simplify_design_type_typeofs, Some(true));
}

#[test]
fn default_plugin_config_matches_default_config() {
    let pc = PluginConfig::default();
    let resolved = pc.resolve(Some("/src/anything.ts"));
    let default = Config::default();
    assert_eq!(resolved.unwrap_type_arrows, default.unwrap_type_arrows);
    assert_eq!(resolved.strip_metadata, default.strip_metadata);
    assert_eq!(
        resolved.unwrap_decorator_arrows,
        default.unwrap_decorator_arrows
    );
    assert_eq!(
        resolved.simplify_metadata_typeofs,
        default.simplify_metadata_typeofs
    );
    assert_eq!(
        resolved.simplify_design_type_typeofs,
        default.simplify_design_type_typeofs
    );
}

// --- Glob patterns ---

#[test]
fn windows_backslash_paths() {
    let mut pc = base_config(false);
    pc.overrides
        .push(override_rule(&["**/*.model.*"], Some(true)));
    let resolved = pc.resolve(Some("src\\models\\venue.model.ts"));
    assert_eq!(resolved.simplify_design_type_typeofs, Some(true));
}

#[test]
fn multiple_patterns_in_files() {
    let mut pc = base_config(false);
    pc.overrides.push(override_rule(
        &["**/venue.model*", "**/user.model*"],
        Some(true),
    ));
    assert_eq!(
        pc.resolve(Some("/src/models/venue.model.ts"))
            .simplify_design_type_typeofs,
        Some(true)
    );
    assert_eq!(
        pc.resolve(Some("/src/models/user.model.ts"))
            .simplify_design_type_typeofs,
        Some(true)
    );
    assert_eq!(
        pc.resolve(Some("/src/models/item.model.ts"))
            .simplify_design_type_typeofs,
        Some(false)
    );
}

#[test]
fn brace_expansion_patterns() {
    let mut pc = base_config(false);
    pc.overrides
        .push(override_rule(&["**/*.{model,schema}.*"], Some(true)));
    assert_eq!(
        pc.resolve(Some("/src/venue.model.ts"))
            .simplify_design_type_typeofs,
        Some(true)
    );
    assert_eq!(
        pc.resolve(Some("/src/venue.schema.ts"))
            .simplify_design_type_typeofs,
        Some(true)
    );
    assert_eq!(
        pc.resolve(Some("/src/venue.service.ts"))
            .simplify_design_type_typeofs,
        Some(false)
    );
}

#[test]
fn absolute_path_matching() {
    let mut pc = base_config(false);
    pc.overrides
        .push(override_rule(&["**/venue.model*"], Some(true)));
    let resolved = pc.resolve(Some("/home/user/project/src/models/venue.model.ts"));
    assert_eq!(resolved.simplify_design_type_typeofs, Some(true));
}

// --- JSON deserialization ---

#[test]
fn backward_compat_no_overrides_field() {
    let json = r#"{"simplifyDesignTypeTypeofs": true}"#;
    let pc: PluginConfig = serde_json::from_str(json).unwrap();
    assert_eq!(pc.base.simplify_design_type_typeofs, Some(true));
    assert!(pc.overrides.is_empty());
}

#[test]
fn empty_overrides_array_in_json() {
    let json = r#"{"simplifyDesignTypeTypeofs": true, "overrides": []}"#;
    let pc: PluginConfig = serde_json::from_str(json).unwrap();
    assert!(pc.overrides.is_empty());
    let resolved = pc.resolve(Some("/src/anything.ts"));
    assert_eq!(resolved.simplify_design_type_typeofs, Some(true));
}

#[test]
fn empty_json_deserializes_to_defaults() {
    let json = "{}";
    let pc: PluginConfig = serde_json::from_str(json).unwrap();
    assert!(pc.overrides.is_empty());
    // All base fields are None (serde default), not Some(true/false)
    assert_eq!(pc.base.unwrap_type_arrows, None);
}

#[test]
fn full_config_deserialization() {
    let json = r#"{
        "simplifyDesignTypeTypeofs": false,
        "overrides": [
            {
                "files": ["**/venue.model*", "**/user.model*"],
                "config": { "simplifyDesignTypeTypeofs": true }
            }
        ]
    }"#;
    let pc: PluginConfig = serde_json::from_str(json).unwrap();
    assert_eq!(pc.base.simplify_design_type_typeofs, Some(false));
    assert_eq!(pc.overrides.len(), 1);
    assert_eq!(
        pc.overrides[0].config.simplify_design_type_typeofs,
        Some(true)
    );
    assert_eq!(
        pc.resolve(Some("/src/models/venue.model.ts"))
            .simplify_design_type_typeofs,
        Some(true)
    );
    assert_eq!(
        pc.resolve(Some("/src/models/item.model.ts"))
            .simplify_design_type_typeofs,
        Some(false)
    );
}
