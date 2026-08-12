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
                ..Default::default()
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
                ..Default::default()
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
                ..Default::default()
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
                ..Default::default()
            }))
        },
        &input,
        &output,
        Default::default(),
    );
}

// autoIndex: false — no index parameter is appended to callbacks that declare none.
#[testing::fixture("tests/fixture-no-auto-index/*/input.tsx")]
fn fixture_no_auto_index(input: PathBuf) {
    let output = input.with_file_name("output.tsx");
    test_fixture(
        Syntax::Typescript(TsSyntax {
            tsx: true,
            ..Default::default()
        }),
        &|_| {
            visit_mut_pass(ReactDataTestIdTransform::new(PluginOptions {
                auto_index: Some(false),
                ..Default::default()
            }))
        },
        &input,
        &output,
        Default::default(),
    );
}

// Every opt-out at once — the 0.1.x id scheme, for a codebase migrating in stages.
// The defaults are what deliver unique ids; these flags only take that away.
#[testing::fixture("tests/fixture-legacy-ids/*/input.tsx")]
fn fixture_legacy_ids(input: PathBuf) {
    let output = input.with_file_name("output.tsx");
    test_fixture(
        Syntax::Typescript(TsSyntax {
            tsx: true,
            ..Default::default()
        }),
        &|_| {
            visit_mut_pass(ReactDataTestIdTransform::new(PluginOptions {
                auto_index: Some(false),
                collection_first_map: Some(false),
                nested_indexes: Some(false),
                map_methods: Some(vec!["map".to_string()]),
                resolve_callback_refs: Some(false),
                index_counter: Some(false),
                ..Default::default()
            }))
        },
        &input,
        &output,
        Default::default(),
    );
}

// nestedIndexes: false on its own, with autoIndex still appending. Guards the rule that
// a parameter is written back only when a testid actually referenced it — the outer
// callback of a nested pair contributes nothing here, so it must keep its signature.
#[testing::fixture("tests/fixture-flat-indexes/*/input.tsx")]
fn fixture_flat_indexes(input: PathBuf) {
    let output = input.with_file_name("output.tsx");
    test_fixture(
        Syntax::Typescript(TsSyntax {
            tsx: true,
            ..Default::default()
        }),
        &|_| {
            visit_mut_pass(ReactDataTestIdTransform::new(PluginOptions {
                nested_indexes: Some(false),
                ..Default::default()
            }))
        },
        &input,
        &output,
        Default::default(),
    );
}

// instanceIds: true — the one option that is off by default, because it rewrites every
// nested id from `Badge.span` to `List.Badge.span`. A component prefixes its ids with
// the data-testid its parent passed down, so two instances no longer collide.
#[testing::fixture("tests/fixture-instance-ids/*/input.tsx")]
fn fixture_instance_ids(input: PathBuf) {
    let output = input.with_file_name("output.tsx");
    test_fixture(
        Syntax::Typescript(TsSyntax {
            tsx: true,
            ..Default::default()
        }),
        &|_| {
            visit_mut_pass(ReactDataTestIdTransform::new(PluginOptions {
                instance_ids: Some(true),
                ..Default::default()
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
    fn map_no_index_param() {
        // callbacks that declare no index get one appended, so ids are unique per item.
        // An appended index is guarded: whether the callee passes one is a static guess,
        // and a wrong guess must degrade to the static id, never to "-undefined".
        check("tests/fixture/map-no-index-param", true, &[
            r#"(item, __testidIdx0)"#,
            r#"data-testid={"List.li" + (__testidIdx0 === void 0 ? "" : "-" + __testidIdx0)}"#,
            // a parameterless callback gains a placeholder so the index stays 2nd
            r#"(__testidItem2, __testidIdx1)"#,
            r#"data-testid={"Stars.span" + (__testidIdx1 === void 0 ? "" : "-" + __testidIdx1)}"#,
            // destructured item parameter is preserved
            r#"({ id }, __testidIdx3)"#,
            r#"data-testid={"Rows.tr" + (__testidIdx3 === void 0 ? "" : "-" + __testidIdx3)}"#,
            // function expressions too
            r#"function(item, __testidIdx4)"#,
            r#"data-testid={"Legacy.li" + (__testidIdx4 === void 0 ? "" : "-" + __testidIdx4)}"#,
        ]);
        let output =
            std::fs::read_to_string("tests/fixture/map-no-index-param/output.tsx").unwrap();
        assert!(
            !output.contains("undefined"),
            "the guard must test against `void 0`, which no minifier can shadow"
        );
    }

    #[test]
    fn map_flatmap() {
        // flatMap passes (item, index) exactly like map
        check("tests/fixture/map-flatmap", true, &[
            // a declared index is not guarded — the source asserts the binding is real
            r#"data-testid={"Groups.li-" + i}"#,
            r#"(group, __testidIdx0)"#,
            r#"data-testid={"Tags.span" + (__testidIdx0 === void 0 ? "" : "-" + __testidIdx0)}"#,
        ]);
    }

    #[test]
    fn auto_index_false_governs_appending_only() {
        // Pins the exact scope of the opt-out, which the migration notes describe: it
        // suppresses an *appended* index and nothing else. A declared index is still
        // used, including in the shapes 0.2.0 newly detects — so `autoIndex: false` is
        // not a general "give me 0.1.x ids back" switch.
        check("tests/fixture-no-auto-index/opt-out", true, &[
            r#"items.map((item)=><li data-testid="List.li">"#,   // restored to static
            r#"data-testid={"Declared.li-" + index}"#,           // declared: still dynamic
            r#"data-testid={"FlatMap.li-" + i}"#,                // newly detected, no opt-out
            r#"data-testid={"Lodash.li-" + i}"#,                 // newly detected, no opt-out
        ]);
        let output =
            std::fs::read_to_string("tests/fixture-no-auto-index/opt-out/output.tsx").unwrap();
        assert!(
            !output.contains("__testidIdx"),
            "no index parameter may be appended while autoIndex is false"
        );
    }

    #[test]
    fn every_opt_out_at_once_reproduces_the_legacy_scheme() {
        // The staged-migration escape hatch: with all four flags set, ids are exactly
        // what 0.1.x produced. This is the ONLY configuration that gives that, and it is
        // opt-in — the defaults infer as much as possible.
        check("tests/fixture-legacy-ids/all-off", true, &[
            r#"items.map((item)=><li data-testid="List.li">"#,     // autoIndex: false
            r#"flatMap((group, i)=><li data-testid="Flat.li">"#,   // mapMethods: ["map"]
            r#"_.map(items, (item, i)=><li data-testid="Lodash.li">"#, // collectionFirstMap
            r#"data-testid={"Table.td-" + c}"#,                    // nestedIndexes: false
        ]);
        let output = std::fs::read_to_string("tests/fixture-legacy-ids/all-off/output.tsx").unwrap();
        assert!(
            !output.contains("__testidIdx"),
            "no parameter may be appended with autoIndex disabled"
        );
        assert!(
            !output.contains(r#""Table.td-" + r"#),
            "with nestedIndexes off only the innermost callback contributes"
        );
    }

    #[test]
    fn nested_indexes_off_still_appends_only_what_is_used() {
        // The flag combination that exercises the write-back rule: appending is ON, but
        // only the innermost callback contributes a segment. An outer callback that
        // renders no JSX of its own must keep its original signature.
        check("tests/fixture-flat-indexes/inner-only", true, &[
            // innermost index only — no outer segment
            r#"data-testid={"Table.td" + (__testidIdx1 === void 0 ? "" : "-" + __testidIdx1)}"#,
            // the <tr> is inside one callback, so it does use the outer index
            r#"rows.map((cells, __testidIdx0)"#,
            // …but here nothing references the outer index, so no parameter is added
            r#"{rows.map((cells)=>cells.map((cell, __testidIdx4)"#,
        ]);
        let output =
            std::fs::read_to_string("tests/fixture-flat-indexes/inner-only/output.tsx").unwrap();
        assert!(
            !output.contains(r#""Table.td" + (__testidIdx0"#),
            "with nestedIndexes off, an outer index must not reach an inner element's id"
        );
    }

    #[test]
    fn dynamic_ids_with_multiple_attributes() {
        // Each attribute gets its own copy of the index expression, all referencing the
        // same binding — and the parameter is appended once, not once per attribute.
        check("tests/fixture-options/dynamic-ids-multi-attr", false, &[
            r#"data-cy={"List.li" + (__testidIdx0 === void 0 ? "" : "-" + __testidIdx0)} data-testid={"List.li" + (__testidIdx0 === void 0 ? "" : "-" + __testidIdx0)}"#,
            r#"data-cy={"Declared.li-" + i} data-testid={"Declared.li-" + i}"#,
            // an existing attribute is preserved; only the missing one is added
            r#"<li data-testid={"Partial.li-" + i} data-cy="pinned">"#,
            // nested: both attributes carry both segments
            r#"data-testid={"Table.td" + (__testidIdx1 === void 0 ? "" : "-" + __testidIdx1) + (__testidIdx2 === void 0 ? "" : "-" + __testidIdx2)}"#,
        ]);
        let output =
            std::fs::read_to_string("tests/fixture-options/dynamic-ids-multi-attr/output.tsx")
                .unwrap();
        assert_eq!(
            output.matches("(item, __testidIdx0)").count(),
            1,
            "the index parameter must be appended once, not once per attribute"
        );
    }

    #[test]
    fn filename_derived_name_reaches_dynamic_ids() {
        // The prefix for an anonymous default export comes from the filename; it has to
        // survive into the dynamic form, not just the static one.
        check("tests/fixture-filename/anonymous-export-map", true, &[
            r#"data-testid="Star-list.ul""#,
            r#"data-testid={"Star-list.li" + (__testidIdx0 === void 0 ? "" : "-" + __testidIdx0)}"#,
        ]);
    }

    #[test]
    fn class_component_map() {
        // Index inference works inside render(), and NOT outside it: a non-render method
        // has no active component, so its callback must keep its original signature.
        check("tests/fixture/class-component-map", true, &[
            r#"data-testid={"ItemList.li" + (__testidIdx0 === void 0 ? "" : "-" + __testidIdx0)}"#,
            // a declared outer index composes with an appended inner one
            r#"data-testid={"Grid.td-" + r + (__testidIdx2 === void 0 ? "" : "-" + __testidIdx2)}"#,
            // renderRow is not render(): untouched, no parameter appended
            r#"return row.split("").map((ch)=><span>{ch}</span>);"#,
        ]);
    }

    #[test]
    fn map_index_param_shapes() {
        // a second parameter that still binds one identifier is usable as the index
        check("tests/fixture/map-index-param-shapes", true, &[
            r#"(item, index = 0)"#,
            r#"data-testid={"Defaulted.li-" + index}"#,   // default value unwrapped
            r#"data-testid={"Annotated.li-" + index}"#,   // type annotation ignored
            r#"data-testid={"WithArray.li-" + index}"#,   // 3rd parameter is untouched
            // a destructuring pattern in the index slot: the first identifier it binds
            // is referenced, guarded, and the occupied slot means nothing is appended
            r#"items.map((item, [i])=><li data-testid={"Destructured.li" + (i === void 0 ? "" : "-" + i)}>"#,
            // a default on the first parameter leaves the index slot free; a parameter
            // without a default may legally follow one that has it
            r#"items.map((item = "?", __testidIdx0)"#,
        ]);
    }

    #[test]
    fn map_lodash_style() {
        // collection-first helpers: the callback is argument 1
        check("tests/fixture/map-lodash-style", true, &[
            r#"data-testid={"Lodash.li-" + i}"#,
            r#"_.map(items, (item, __testidIdx0)"#,
            r#"map(items, (item, __testidIdx1)"#,
            r#"Children.map(children, (child, __testidIdx2)"#,
        ]);
    }

    #[test]
    fn map_nested() {
        // one index segment per enclosing callback, or the inner index would repeat
        // for every outer iteration
        check("tests/fixture/map-nested", true, &[
            r#"data-testid={"Table.tr" + (__testidIdx0 === void 0 ? "" : "-" + __testidIdx0)}"#,
            r#"data-testid={"Table.td" + (__testidIdx0 === void 0 ? "" : "-" + __testidIdx0) + (__testidIdx1 === void 0 ? "" : "-" + __testidIdx1)}"#,
            // a declared outer index stays unguarded next to an appended inner one
            r#"data-testid={"Mixed.section-" + groupIndex}"#,
            r#"data-testid={"Mixed.p-" + groupIndex + (__testidIdx2 === void 0 ? "" : "-" + __testidIdx2)}"#,
        ]);
    }

    #[test]
    fn map_this_arg() {
        // no parameter is appended next to a thisArg, but a declared index still counts
        check("tests/fixture/map-this-arg", true, &[
            r#"function(item, index)"#,
            r#"data-testid={"List.li-" + index}"#,
        ]);
        let output = std::fs::read_to_string("tests/fixture/map-this-arg/output.tsx").unwrap();
        assert!(
            !output.contains("__testidIdx"),
            "a thisArg argument must not gain an appended index parameter"
        );
    }

    #[test]
    fn map_no_inject_cases() {
        // shapes where nothing dynamic can be emitted at all: no parameter list to read,
        // no attribute to write, or no identifier bound to reference
        check("tests/fixture/map-no-inject-cases", false, &[
            r#"items.map(renderItem)"#,                       // imported: no declaration to index
            r#".map((item)=>item.trim())"#,                   // no JSX → signature untouched
            r#"items.map((item)=><li data-testid="custom">"#, // attr already present
        ]);
        let output =
            std::fs::read_to_string("tests/fixture/map-no-inject-cases/output.tsx").unwrap();
        assert!(
            !output.contains("__testidIdx"),
            "no index parameter may be appended in any of these shapes"
        );
    }

    #[test]
    fn map_uncertain_callees() {
        // The syntax cannot say whether these callees pass an index, so all are inferred
        // and every segment is guarded. Narrowing detection instead would guarantee
        // duplicate ids for the ones that DO pass an index (RxJS, .map(cb, thisArg)).
        check("tests/fixture/map-uncertain-callees", true, &[
            // ramda order — the callee may pass no index, so the appended parameter
            // defaults to a call-site counter and the segment needs no guard
            r#"((__testidSeq0)=>R.map((item, __testidIdx1 = __testidSeq0++)"#,
            r#"data-testid={"Ramda.li-" + __testidIdx1}"#,
            // bare map(fn) — RxJS passes an index here, curried ramda does not. No
            // collection argument, so no counter: the function this call returns may be
            // stored and reused, and a counter would drift instead of tracking items.
            r#"map((item, __testidIdx2)=><li data-testid={"Curried.li" + (__testidIdx2 === void 0 ? "" : "-" + __testidIdx2)}>"#,
            // .map(cb, thisArg) is Array.prototype.map, so the real index wins and the
            // counter's default never runs — but the shape cannot say so statically
            r#"function(item, __testidIdx4 = __testidSeq3++)"#,
            // a rest parameter already holds the index — no signature change needed
            r#"data-testid={"Rest.li" + (args[1] === void 0 ? "" : "-" + args[1])}"#,
            r#"items.map((...args)=>"#,
            r#"data-testid={"NamedThenRest.li" + (rest[0] === void 0 ? "" : "-" + rest[0])}"#,
            r#"items.map((item, ...rest)=>"#,
            // a destructuring pattern in the index slot binds something to reference
            r#"items.map((item, [i])=><li data-testid={"Destructured.li" + (i === void 0 ? "" : "-" + i)}>"#,
            r#"items.map((item, { index })=><li data-testid={"NamedInPattern.li" + (index === void 0 ? "" : "-" + index)}>"#,
        ]);
        let output =
            std::fs::read_to_string("tests/fixture/map-uncertain-callees/output.tsx").unwrap();
        assert!(
            !output.contains("undefined"),
            "an inferred segment is either guarded with `void 0` or bound by a counter"
        );
    }

    #[test]
    fn map_callback_refs() {
        // A callback passed by reference has no parameter list at the call site, so the
        // index is added where the function is declared — before or after the call, and
        // hoisted or not, which is why the names are collected in a pass of their own.
        check("tests/fixture/map-callback-refs", true, &[
            // declared after the call site that reaches it
            r#"const renderItem = (item: string, __testidIdx0)"#,
            r#"data-testid={"renderItem.li" + (__testidIdx0 === void 0 ? "" : "-" + __testidIdx0)}"#,
            // an index the function already declares is used as-is — but still guarded,
            // because a named function can also be called with no index at all
            r#"data-testid={"renderRow.tr" + (position === void 0 ? "" : "-" + position)}"#,
            // the collection-first shape passes the reference as argument 1
            r#"function renderTag(tag: string, __testidIdx1)"#,
            // no parameters at all: a placeholder keeps the index in the second slot
            r#"const renderStar = (__testidItem3, __testidIdx2)"#,
            // the referenced function's own map nests underneath it
            r#"data-testid={"renderGroup.b" + (groupIndex === void 0 ? "" : "-" + groupIndex) + (__testidIdx4 === void 0 ? "" : "-" + __testidIdx4)}"#,
            // a component used as a callback is a callback
            r#"data-testid={"Cell.td" + (__testidIdx5 === void 0 ? "" : "-" + __testidIdx5)}"#,
        ]);
        let output = std::fs::read_to_string("tests/fixture/map-callback-refs/output.tsx").unwrap();
        assert!(
            output.contains(r#"const plainHelper = (value: string)=><em data-testid="plainHelper.em">"#),
            "a function never used as a callback must keep its signature and its static id"
        );
        assert!(
            !output.contains("undefined"),
            "every segment here is a guess about a call site elsewhere, so all are guarded"
        );
    }

    #[test]
    fn map_index_counter() {
        // Where the callee may pass no index, the appended parameter defaults to a
        // counter bound just outside the call. The default runs per invocation and only
        // when nothing was passed, so the binding always holds a number — which is what
        // licenses the unguarded segment.
        check("tests/fixture/map-index-counter", true, &[
            r#"((__testidSeq0)=>R.map((item, __testidIdx1 = __testidSeq0++)"#,
            r#"data-testid={"Ramda.li-" + __testidIdx1}"#,
            // an index the source declares is left exactly as written
            r#"R.map((item, i)=><li data-testid={"Declared.li-" + i}>"#,
            // nested calls get one counter each, and one segment per level
            r#"data-testid={"Table.td-" + __testidIdx7 + "-" + __testidIdx9}"#,
        ]);
        let output = std::fs::read_to_string("tests/fixture/map-index-counter/output.tsx").unwrap();
        // A curried map(fn) must NOT get a counter: that call evaluates once and the
        // function it returns may be stored, so the counter would climb past the item
        // positions instead of tracking them.
        assert!(
            output.contains(r#"R.map((item, __testidIdx2)=><li data-testid={"Curried.li" + (__testidIdx2 === void 0 ? "" : "-" + __testidIdx2)}>"#),
            "a call with no collection argument keeps the guarded index and gets no counter"
        );
        // An arrow cannot await, so a call with an awaited argument cannot be wrapped.
        assert!(
            output.contains(r#"data-testid={"Awaited.li" + (__testidIdx10 === void 0 ? "" : "-" + __testidIdx10)}"#),
            "a call containing await must fall back to the guarded index, not be wrapped"
        );
        assert!(
            !output.contains("__testidSeq") || !output.contains("await load()))("),
            "the counter arrow must never be wrapped around an awaited call"
        );
        // A callback with no JSX referenced nothing, so neither parameter is written back.
        assert!(
            output.contains(r#"R.map((item)=>item.trim(), items).length"#),
            "a callback whose body has no testid keeps its original signature"
        );
    }

    #[test]
    fn instance_ids() {
        // instanceIds: true — a component prefixes its ids with the one it was handed,
        // so two instances of the same component stop colliding. The fallback is the
        // component's own name, i.e. exactly the id it had before.
        // (the input deliberately contains data-testid already: one component
        // destructures the prop itself, another pins an id on an element)
        check("tests/fixture-instance-ids/components", false, &[
            // the prop is added to the pattern the component already had, first, so a
            // rest element stays last
            r#"const Badge = ({ "data-testid": __testidPrefix0, label }"#,
            r#"data-testid={(__testidPrefix0 || "Badge") + ".span"}"#,
            // two instances get different prefixes from their parent
            r#"<Badge data-testid={(__testidPrefix1 || "Toolbar") + ".Badge"} label="a"/>"#,
            r#"<Badge data-testid={(__testidPrefix1 || "Toolbar") + ".Badge2"} label="b"/>"#,
            // an identifier props parameter is read through a `&&`, since a component
            // function called as a plain helper may get no props at all
            r#"data-testid={(props && props["data-testid"] || "Card") + ".article"}"#,
            // a component that declared no parameter gets one, defaulted so that calling
            // it with no argument still cannot throw
            r#"const Field = ({ "data-testid": __testidPrefix8 } = {})"#,
            // a component that already destructures the prop is read, not rewritten
            r#"const Titled = ({ "data-testid": testId }"#,
            r#"data-testid={(testId || "Titled") + ".h2"}"#,
            // prefix and map index compose
            r#"data-testid={(__testidPrefix3 || "Rows") + ".Badge" + (__testidIdx4 === void 0 ? "" : "-" + __testidIdx4)}"#,
            // a class reads this.props
            r#"data-testid={(this.props && this.props["data-testid"] || "Panel") + ".section"}"#,
            // …but not from inside a function expression, where `this` is something else
            r#"data-testid={"Legacy.li" + (__testidIdx5 === void 0 ? "" : "-" + __testidIdx5)}"#,
            // the prefix is inserted before a rest element, never after it
            r#"const Input = ({ "data-testid": __testidPrefix7, label, ...rest }"#,
            // a default on the props parameter is transparent — the property goes into
            // the pattern underneath it, not around it
            r#"const Defaulted = ({ "data-testid": __testidPrefix9, label } = {})"#,
            // a function reached by reference takes an INDEX, never a prefix: its first
            // argument is the item, so a prefix read there would read a property of it
            r#"const renderItem = (item: string, __testidIdx11)"#,
            r#"data-testid={"renderItem.li" + (__testidIdx11 === void 0 ? "" : "-" + __testidIdx11)}"#,
        ]);
        let output =
            std::fs::read_to_string("tests/fixture-instance-ids/components/output.tsx").unwrap();
        assert!(
            output.contains(r#"const Pinned = ()=><div data-testid="pinned"/>;"#),
            "a component whose only element already has the attribute gains no parameter"
        );
        // An array pattern in the props slot is not a React component: no prefix exists
        // to read, so those ids stay exactly as they are without the option.
        assert!(
            output.contains(r#"const NotAComponent = ([first])=><p data-testid="NotAComponent.p">"#),
            "a non-props first parameter must be left alone and keep static ids"
        );
        // The fallback is the component name, so an unrendered-by-us component keeps the
        // ids it had before the option was turned on.
        assert!(
            !output.contains(r#"|| "") "#),
            "the fallback must be the component name, never an empty prefix"
        );
    }

    // Nothing above is opt-in except instanceIds: with every opt-out set, the ids are
    // exactly 0.1.x's, including for the two shapes newly reached by the counter and by
    // callback-reference resolution.
    #[test]
    fn legacy_ids_cover_the_new_inference() {
        check("tests/fixture-legacy-ids/all-off", true, &[
            // indexCounter: false — no counter arrow around a ramda-shaped call
            r#"R.map((item)=><li data-testid="Ramda.li">"#,
            // resolveCallbackRefs: false — the referenced function is not looked up
            r#"const renderItem = (item: string)=><li data-testid="renderItem.li">"#,
        ]);
        let output = std::fs::read_to_string("tests/fixture-legacy-ids/all-off/output.tsx").unwrap();
        assert!(
            !output.contains("__testidSeq"),
            "no counter may be introduced with indexCounter disabled"
        );
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
