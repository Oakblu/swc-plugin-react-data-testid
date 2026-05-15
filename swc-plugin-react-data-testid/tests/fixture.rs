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

// Reads filename.txt from the fixture directory and passes it to the transform
// so we can test the filename-based fallback for anonymous default exports.
#[testing::fixture("tests/fixture-filename/*/input.tsx")]
fn fixture_with_filename(input: PathBuf) {
    let output = input.with_file_name("output.tsx");
    let filename = std::fs::read_to_string(input.with_file_name("filename.txt"))
        .ok()
        .map(|s| s.trim().to_string());
    test_fixture(
        Syntax::Typescript(TsSyntax {
            tsx: true,
            ..Default::default()
        }),
        &|_| {
            visit_mut_pass(ReactDataTestIdTransform::new_with_filename(
                PluginOptions::default(),
                filename.clone(),
            ))
        },
        &input,
        &output,
        Default::default(),
    );
}
