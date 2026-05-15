use std::path::PathBuf;

use swc_core::{
    ecma::{
        parser::{Syntax, TsSyntax},
        transforms::testing::test_fixture,
        visit::visit_mut_pass,
    },
    testing,
};
use swc_plugin_react_data_testid::{PluginOptions, ReactDataTestIdTransform};

// Discovers all tests/fixture/*/input.tsx files and diffs against output.tsx.
// To regenerate output files after algorithm changes: UPDATE=1 cargo test
#[testing::fixture("tests/fixture/*/input.tsx")]
fn fixture(input: PathBuf) {
    let output = input.with_file_name("output.tsx");
    test_fixture(
        Syntax::Typescript(TsSyntax {
            tsx: true,
            ..Default::default()
        }),
        &|_| visit_mut_pass(ReactDataTestIdTransform::new(PluginOptions::default())),
        &input,
        &output,
        Default::default(),
    );
}

#[testing::fixture("tests/fixture-options/*/input.tsx")]
fn fixture_with_options(input: PathBuf) {
    let output = input.with_file_name("output.tsx");
    test_fixture(
        Syntax::Typescript(TsSyntax {
            tsx: true,
            ..Default::default()
        }),
        &|_| {
            visit_mut_pass(ReactDataTestIdTransform::new(PluginOptions {
                attributes: Some(vec!["data-testid".into(), "data-cy".into()]),
            }))
        },
        &input,
        &output,
        Default::default(),
    );
}

#[testing::fixture("tests/fixture-playwright/*/input.tsx")]
fn fixture_playwright(input: PathBuf) {
    let output = input.with_file_name("output.tsx");
    test_fixture(
        Syntax::Typescript(TsSyntax {
            tsx: true,
            ..Default::default()
        }),
        &|_| {
            visit_mut_pass(ReactDataTestIdTransform::new(PluginOptions {
                attributes: Some(vec!["data-test-id".into()]),
            }))
        },
        &input,
        &output,
        Default::default(),
    );
}

#[testing::fixture("tests/fixture-multi-attrs/*/input.tsx")]
fn fixture_multi_attrs(input: PathBuf) {
    let output = input.with_file_name("output.tsx");
    test_fixture(
        Syntax::Typescript(TsSyntax {
            tsx: true,
            ..Default::default()
        }),
        &|_| {
            visit_mut_pass(ReactDataTestIdTransform::new(PluginOptions {
                attributes: Some(vec![
                    "data-testid".into(),
                    "data-cy".into(),
                    "data-test-id".into(),
                    "data-pw".into(),
                ]),
            }))
        },
        &input,
        &output,
        Default::default(),
    );
}
