use std::path::PathBuf;
use swc_core::ecma::{
    parser::{EsSyntax, Syntax},
    transforms::testing::test_fixture,
    visit::visit_mut_pass,
};

use swc_jest_coverage_nestjs_plugin::{Config, PluginConfig, visitor::DecoratorCoverageVisitor};

fn load_config(input: &PathBuf) -> Config {
    let config_path = input.with_file_name("config.json");
    if config_path.exists() {
        let config_str = std::fs::read_to_string(&config_path).unwrap();
        // Deserialize through PluginConfig to prove backward compatibility:
        // existing config.json files (which don't have "overrides") must still work.
        let plugin_config: PluginConfig = serde_json::from_str(&config_str).unwrap();
        // Resolve with the fixture's directory name as a synthetic filename
        let filename = input.to_string_lossy();
        plugin_config.resolve(Some(&filename))
    } else {
        Config::default()
    }
}

#[testing::fixture("tests/fixture/*/input.js")]
fn fixture_test(input: PathBuf) {
    let output = input.with_file_name("output.js");
    let config = load_config(&input);
    test_fixture(
        Syntax::Es(EsSyntax::default()),
        &|_| visit_mut_pass(DecoratorCoverageVisitor::new(config.clone())),
        &input,
        &output,
        Default::default(),
    );
}
