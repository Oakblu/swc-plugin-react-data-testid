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
                manifest_dir: None,
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
                manifest_dir: None,
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
                manifest_dir: None,
            }))
        },
        &input,
        &output,
        Default::default(),
    );
}

// Tests that an explicitly empty attributes array falls back to the default ["data-testid"].
#[testing::fixture("tests/fixture-empty-attrs/*/input.tsx")]
fn fixture_empty_attrs(input: PathBuf) {
    let output = input.with_file_name("output.tsx");
    test_fixture(
        Syntax::Typescript(TsSyntax {
            tsx: true,
            ..Default::default()
        }),
        &|_| {
            visit_mut_pass(ReactDataTestIdTransform::new(PluginOptions {
                attributes: Some(vec![]),
                manifest_dir: None,
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

// ─── Correctness assertions for the 7 new edge-case fixtures ──────────────────
// test_fixture (above) guarantees the plugin output is stable across code changes.
// The tests below guarantee the INITIAL snapshot was correct — i.e. the plugin
// actually injected the expected data-testid values and didn't silently no-op.
//
// Logic: if output.tsx passes test_fixture AND contains the expected IDs, then
// the plugin must have produced correct output.
#[cfg(test)]
mod correctness {
    fn check(fixture_dir: &str, assert_input_clean: bool, expected_ids: &[&str]) {
        let out_path = format!("{fixture_dir}/output.tsx");
        let output = std::fs::read_to_string(&out_path)
            .unwrap_or_else(|_| panic!("output.tsx not found at {out_path}"));

        if assert_input_clean {
            let in_path = format!("{fixture_dir}/input.tsx");
            let input = std::fs::read_to_string(&in_path).unwrap();
            assert!(
                !input.contains("data-testid"),
                "precondition failed: input at {in_path} already has data-testid"
            );
        }

        for id in expected_ids {
            assert!(output.contains(id), "output {out_path} is missing: {id}");
        }
    }

    #[test]
    fn async_component() {
        // async FnDecl and async arrow — plugin must not skip due to async flag
        check("tests/fixture/async-component", true, &[
            r#"data-testid="AsyncPage.main""#,
            r#"data-testid="AsyncPage.h1""#,
            r#"data-testid="AsyncPage.p""#,
            r#"data-testid="AsyncArrow.div""#,
            r#"data-testid="AsyncWithEarlyReturn.span""#,
            r#"data-testid="AsyncWithEarlyReturn.article""#,
        ]);
    }

    #[test]
    fn try_catch_jsx() {
        // JSX in both try and catch branches must be tagged
        check("tests/fixture/try-catch-jsx", true, &[
            r#"data-testid="RiskyComponent.div""#,
            r#"data-testid="RiskyComponent.span""#,
            r#"data-testid="WithFinally.section""#,
            r#"data-testid="WithFinally.aside""#,
        ]);
    }

    #[test]
    fn many_same_elements() {
        // counter must reach 5 without off-by-one; independent per element type
        check("tests/fixture/many-same-elements", true, &[
            r#"data-testid="ItemList.li""#,
            r#"data-testid="ItemList.li2""#,
            r#"data-testid="ItemList.li3""#,
            r#"data-testid="ItemList.li4""#,
            r#"data-testid="ItemList.li5""#,
            r#"data-testid="Article.p""#,
            r#"data-testid="Article.p5""#,
            r#"data-testid="Article.h2""#,
            r#"data-testid="Article.h22""#, // second <h2>: "h2" + "2" = "h22"
        ]);
    }

    #[test]
    fn iife_jsx() {
        // IIFE arrow is a CallExpr callee — no new scope, shares parent counter
        check("tests/fixture/iife-jsx", true, &[
            r#"data-testid="Dashboard.div""#,
            r#"data-testid="Dashboard.AdminPanel""#,
            r#"data-testid="Dashboard.ModPanel""#,
            r#"data-testid="Dashboard.UserPanel""#,
            r#"data-testid="Wizard.section""#,
            r#"data-testid="Wizard.span""#,
        ]);
    }

    #[test]
    fn jsx_in_default_params() {
        // enter_component() fires before visit_mut_children_with, so param
        // default JSX is visited while current_component is already set
        check("tests/fixture/jsx-in-default-params", true, &[
            r#"data-testid="WithFallback.Spinner""#,   // from the param default!
            r#"data-testid="WithFallback.main""#,
            r#"data-testid="WithMultipleDefaults.DefaultHeader""#, // param default
            r#"data-testid="WithMultipleDefaults.DefaultFooter""#, // param default
            r#"data-testid="WithMultipleDefaults.div""#,
            r#"data-testid="WithMultipleDefaults.article""#,
        ]);
    }

    #[test]
    fn generator_component() {
        // FnDecl has no is_generator guard — generators are a known false positive
        check("tests/fixture/generator-component", true, &[
            r#"data-testid="ListGenerator.li""#,
            r#"data-testid="ListGenerator.li2""#,
            r#"data-testid="ListGenerator.li3""#,
            r#"data-testid="TableRows.tr""#,
            r#"data-testid="TableRows.td""#,
        ]);
    }

    #[test]
    fn partial_existing_attrs() {
        // input already has data-cy on one element and data-testid on another;
        // each attribute is injected independently — only the missing one is added
        check("tests/fixture-options/partial-existing-attrs", false, &[
            // fresh <form>: both attrs injected
            r#"data-testid="Form.form""#,
            r#"data-cy="Form.form""#,
            // <input> had data-cy="form-input" → only data-testid injected; original cy kept
            r#"data-testid="Form.input""#,
            r#"data-cy="form-input""#,
            // <button> had data-testid="form-submit" → only data-cy injected; original testid kept
            r#"data-cy="Form.button""#,
            r#"data-testid="form-submit""#,
            // fresh <label>: both attrs injected
            r#"data-testid="Form.label""#,
            r#"data-cy="Form.label""#,
        ]);
    }
}
