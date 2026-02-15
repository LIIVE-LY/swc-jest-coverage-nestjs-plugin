use std::path::PathBuf;
use swc_core::ecma::{
    parser::{EsSyntax, Syntax},
    transforms::testing::test_fixture,
    visit::visit_mut_pass,
};

use swc_jest_coverage_nestjs_plugin::{Config, visitor::DecoratorCoverageVisitor};

#[testing::fixture("tests/fixture/*/input.js")]
fn fixture_test(input: PathBuf) {
    let output = input.with_file_name("output.js");
    test_fixture(
        Syntax::Es(EsSyntax::default()),
        &|_| visit_mut_pass(DecoratorCoverageVisitor::new(Config::default())),
        &input,
        &output,
        Default::default(),
    );
}
