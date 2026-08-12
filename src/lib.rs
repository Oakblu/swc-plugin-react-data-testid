use std::collections::{HashMap, HashSet};
use std::path::Path;

use serde::Deserialize;
use swc_core::{
    common::{plugin::metadata::TransformPluginMetadataContextKind, SyntaxContext, DUMMY_SP},
    ecma::{
        ast::*,
        visit::{Visit, VisitMut, VisitMutWith, VisitWith},
    },
    plugin::{plugin_transform, proxies::TransformPluginProgramMetadata},
};

const DEFAULT_ATTRIBUTE: &str = "data-testid";

// Callee names treated as iteration by default. Covers Array.prototype.map/flatMap,
// lodash `_.map`, React.Children.map, jQuery.map. Override with the `mapMethods` option.
const MAP_METHODS: [&str; 2] = ["map", "flatMap"];

// Base names for parameters the plugin appends to a callback. Collisions are avoided
// by a per-file counter plus a scan of the callback (see fresh_ident).
const INDEX_PARAM_BASE: &str = "__testidIdx";
const ITEM_PARAM_BASE: &str = "__testidItem";

// Derives a PascalCase component name from a file path.
// Returns None for paths with no extension, multi-dot base names (e.g. "index.test"),
// or empty base names.
fn component_name_from_filename(filename: &str) -> Option<String> {
    let path = Path::new(filename);
    path.extension()?.to_str()?; // require a file extension
    let stem = path.file_stem()?.to_str()?;
    // Reject "index.test", "foo.spec", etc.
    if stem.contains('.') || stem.is_empty() {
        return None;
    }
    let mut chars = stem.chars();
    Some(chars.next()?.to_uppercase().collect::<String>() + chars.as_str())
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginOptions {
    pub attributes: Option<Vec<String>>,
    // Directory where per-file manifest JSON files are written during transform.
    // Each file produces one JSON: { "file": "...", "testids": [...] }
    // Merge with: node scripts/merge-manifest.mjs <manifestDir> <output.json>
    pub manifest_dir: Option<String>,
    // Append an index parameter to map callbacks that do not declare one, so every
    // iteration gets a distinct testid. Defaults to true. Set false to opt out (e.g. a
    // custom `map` whose callback is not called with an index).
    pub auto_index: Option<bool>,
    // Callee names treated as iteration. Defaults to ["map", "flatMap"]. Narrow it to
    // ["map"] to leave flatMap callbacks alone; widen it to add your own iterator.
    pub map_methods: Option<Vec<String>>,
    // Recognise the collection-first form `f(collection, callback)` — `_.map`,
    // `React.Children.map`, `$.map`. Defaults to true.
    pub collection_first_map: Option<bool>,
    // Give an element inside nested callbacks one id segment per level. Defaults to
    // true. Set false for the pre-0.2.0 innermost-only id, which repeats across outer
    // iterations — it makes ids collide, so it exists only for staged migration.
    pub nested_indexes: Option<bool>,
}

// How this level's index is reached from inside the callback.
enum IndexSource {
    // A parameter the source declares: `(item, i) => …`. Always bound, so the id needs
    // no guard, and the compact `{"List.li-" + i}` form is used.
    Declared(Ident),
    // An element of a rest parameter: `(...args) => …` reads `args[1]`. No signature
    // change is needed, but the element may be absent, so the segment is guarded.
    RestElement { rest: Ident, position: f64 },
    // A parameter the plugin appends, written back only if a testid referenced it.
    Appended(Ident),
}

// One enclosing map-like callback's index, and how the id refers to it.
struct MapIndex {
    source: IndexSource,
    used: bool,
}

impl MapIndex {
    // Expression that evaluates to this level's index inside the callback body.
    fn expr(&self) -> Expr {
        match &self.source {
            // Clone the binding's Ident verbatim so sym *and* SyntaxContext match the
            // parameter. A fresh IdentName would carry an empty ctxt and not resolve.
            IndexSource::Declared(id) | IndexSource::Appended(id) => Expr::Ident(id.clone()),
            IndexSource::RestElement { rest, position } => Expr::Member(MemberExpr {
                span: DUMMY_SP,
                obj: Box::new(Expr::Ident(rest.clone())),
                prop: MemberProp::Computed(ComputedPropName {
                    span: DUMMY_SP,
                    expr: Box::new(Expr::Lit(Lit::Num(Number {
                        span: DUMMY_SP,
                        value: *position,
                        raw: None,
                    }))),
                }),
            }),
        }
    }

    // Whether the emitted segment must survive the index being absent at runtime.
    // Only a parameter the source itself declares is taken on trust.
    fn needs_guard(&self) -> bool {
        !matches!(self.source, IndexSource::Declared(_))
    }
}

// Which argument of a map-like call holds the callback.
struct MapCallSite {
    arg_index: usize,
}

// Collects identifier names (and the first SyntaxContext seen) from a subtree.
#[derive(Default)]
struct IdentScan {
    syms: HashSet<String>,
    first_ctxt: Option<SyntaxContext>,
}

impl Visit for IdentScan {
    fn visit_ident(&mut self, n: &Ident) {
        self.first_ctxt.get_or_insert(n.ctxt);
        self.syms.insert(n.sym.to_string());
    }
}

pub struct ReactDataTestIdTransform {
    options: PluginOptions,
    // Outer key: component name. Inner key: element name. Value: occurrence count.
    component_counters: HashMap<String, HashMap<String, u32>>,
    current_component: Option<String>,
    // Tracks the enclosing named class declaration so visit_mut_class_method
    // can activate current_component only for the render() method.
    current_class: Option<String>,
    // Fallback name for anonymous default exports, derived from the source filename.
    filename_component_name: Option<String>,
    // Raw source filename stored for manifest output.
    pub source_filename: Option<String>,
    // All testid values generated during this transform (for manifest).
    pub generated_testids: Vec<String>,
    // Index bindings of the map-like callbacks currently being traversed, outermost
    // first. Used to generate dynamic testids like {"Comp.el-" + index} instead of
    // static strings. Stores the full Ident (not just the name) so the generated
    // reference carries the parameter's SyntaxContext. Without it the reference is
    // unresolved, and a minifier renames the parameter while leaving our reference
    // dangling -> ReferenceError.
    map_indices: Vec<MapIndex>,
    // Monotonic counter for generated parameter names, unique per file.
    next_generated_id: u32,
    // Owned default for `mapMethods`, so map_methods() can hand out a slice either way.
    default_map_methods: Vec<String>,
}

impl ReactDataTestIdTransform {
    pub fn new(options: PluginOptions) -> Self {
        Self::new_with_filename(options, None)
    }

    pub fn new_with_filename(options: PluginOptions, filename: Option<String>) -> Self {
        Self {
            filename_component_name: filename
                .as_deref()
                .and_then(component_name_from_filename),
            source_filename: filename,
            options,
            component_counters: HashMap::new(),
            current_component: None,
            current_class: None,
            generated_testids: Vec::new(),
            map_indices: Vec::new(),
            next_generated_id: 0,
            default_map_methods: MAP_METHODS.iter().map(|m| m.to_string()).collect(),
        }
    }

    fn attributes(&self) -> Vec<String> {
        self.options
            .attributes
            .as_ref()
            .filter(|v| !v.is_empty())
            .cloned()
            .unwrap_or_else(|| vec![DEFAULT_ATTRIBUTE.to_string()])
    }

    // Extracts the local name from a JSX element name.
    // Modal.Header → "Header", div → "div", namespaced → None.
    fn element_name(name: &JSXElementName) -> Option<String> {
        match name {
            JSXElementName::Ident(ident) => Some(ident.sym.to_string()),
            JSXElementName::JSXMemberExpr(member) => Some(member.prop.sym.to_string()),
            JSXElementName::JSXNamespacedName(_) => None,
        }
    }

    fn has_attribute(attrs: &[JSXAttrOrSpread], attr_name: &str) -> bool {
        attrs.iter().any(|a| match a {
            JSXAttrOrSpread::JSXAttr(attr) => match &attr.name {
                JSXAttrName::Ident(ident) => ident.sym.as_ref() == attr_name,
                _ => false,
            },
            _ => false,
        })
    }

    // Returns "element" for the first occurrence, "element2" for the second, etc.
    // Counters are scoped per component — call enter_component() to reset.
    fn unique_element_id(&mut self, component: &str, element: &str) -> String {
        let counters = self
            .component_counters
            .entry(component.to_string())
            .or_default();
        let count = counters.entry(element.to_string()).or_insert(0);
        *count += 1;
        if *count == 1 {
            element.to_string()
        } else {
            format!("{}{}", element, count)
        }
    }

    fn make_attr(attr_name: &str, value: &str) -> JSXAttrOrSpread {
        JSXAttrOrSpread::JSXAttr(JSXAttr {
            span: DUMMY_SP,
            name: JSXAttrName::Ident(IdentName {
                span: DUMMY_SP,
                sym: attr_name.into(),
            }),
            value: Some(JSXAttrValue::Str(Str {
                span: DUMMY_SP,
                value: value.into(),
                raw: None,
            })),
        })
    }

    // Generates attr_name={"prefix-" + i}, or {"prefix-" + i + "-" + j} when the element
    // sits inside nested map callbacks. Every enclosing index takes part, otherwise an
    // inner index alone would repeat for each outer iteration.
    //
    // An index the plugin *appended* is guarded, because whether the callee really
    // passes one is a static guess:
    //
    //   {"List.li" + (__testidIdx0 === void 0 ? "" : "-" + __testidIdx0)}
    //
    // A wrong guess therefore degrades to the plain static id `List.li` — the pre-0.2.0
    // behaviour — instead of producing `List.li-undefined` on every item. Indices the
    // source declares are not guarded: writing `(item, i) =>` asserts the binding is
    // real, and the unguarded form keeps the common output small.
    //
    // The test is strictly against `undefined`, not a truthiness or numeric check:
    // `_.map(someObject, cb)` passes a *string* key, and index `0` is falsy.
    fn make_dynamic_attr(
        attr_name: &str,
        prefix: &str,
        segments: &[(Expr, bool)],
    ) -> JSXAttrOrSpread {
        fn lit(value: String) -> Box<Expr> {
            Box::new(Expr::Lit(Lit::Str(Str {
                span: DUMMY_SP,
                value: value.into(),
                raw: None,
            })))
        }
        fn concat(left: Box<Expr>, right: Box<Expr>) -> Box<Expr> {
            Box::new(Expr::Bin(BinExpr {
                span: DUMMY_SP,
                op: BinaryOp::Add,
                left,
                right,
            }))
        }
        // Text pieces are merged as they are pushed, so a separator folds into the
        // literal before it: `"List.li-" + i`, not `"List.li" + "-" + i`.
        enum Part {
            Text(String),
            Node(Box<Expr>),
        }
        fn push_text(parts: &mut Vec<Part>, s: &str) {
            match parts.last_mut() {
                Some(Part::Text(t)) => t.push_str(s),
                _ => parts.push(Part::Text(s.to_string())),
            }
        }

        // The leading string literal makes every following `+` a string concatenation,
        // so numeric indices never sum.
        let mut parts = vec![Part::Text(prefix.to_string())];
        for (index_expr, guarded) in segments {
            let reference = || Box::new(index_expr.clone());
            if *guarded {
                parts.push(Part::Node(Box::new(Expr::Cond(CondExpr {
                    span: DUMMY_SP,
                    test: Box::new(Expr::Bin(BinExpr {
                        span: DUMMY_SP,
                        op: BinaryOp::EqEqEq,
                        left: reference(),
                        right: Box::new(Expr::Unary(UnaryExpr {
                            span: DUMMY_SP,
                            op: UnaryOp::Void,
                            arg: Box::new(Expr::Lit(Lit::Num(Number {
                                span: DUMMY_SP,
                                value: 0.0,
                                raw: None,
                            }))),
                        })),
                    })),
                    cons: lit(String::new()),
                    alt: concat(lit("-".to_string()), reference()),
                }))));
            } else {
                push_text(&mut parts, "-");
                parts.push(Part::Node(reference()));
            }
        }

        let node = |part| match part {
            Part::Text(t) => lit(t),
            Part::Node(n) => n,
        };
        let mut parts = parts.into_iter();
        // The first part is always the prefix text, so the fold starts from a literal.
        let mut expr = parts.next().map(node).unwrap_or_else(|| lit(String::new()));
        for part in parts {
            expr = concat(expr, node(part));
        }

        JSXAttrOrSpread::JSXAttr(JSXAttr {
            span: DUMMY_SP,
            name: JSXAttrName::Ident(IdentName {
                span: DUMMY_SP,
                sym: attr_name.into(),
            }),
            value: Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
                span: DUMMY_SP,
                expr: JSXExpr::Expr(expr),
            })),
        })
    }

    fn is_fn_literal(arg: &ExprOrSpread) -> bool {
        arg.spread.is_none() && matches!(arg.expr.as_ref(), Expr::Arrow(_) | Expr::Fn(_))
    }

    // Recognises any call to a map-like name that has a callback literal in argument 0
    // or 1, covering every iteration form in the wild:
    //   items.map(cb)           Array.prototype.map / flatMap, lodash chains
    //   items.map(cb, thisArg)  Array.prototype.map — the callback still gets an index
    //   _.map(items, cb)        lodash, React.Children.map, jQuery.map
    //   map(items, cb)          the same functions imported by name
    //   R.map(cb, items)        ramda — no index is passed
    //   map(cb)                 curried ramda / lodash-fp (no index), RxJS (index)
    //
    // Whether the callee really passes an index is NOT decided here, and cannot be: the
    // last two shapes are the same syntax for libraries that behave differently. That is
    // why the emitted segment is guarded (make_dynamic_attr) — a callee that passes no
    // index yields the plain static id. Detecting broadly and guarding the output finds
    // strictly more real indices than detecting narrowly ever could.
    fn map_call_site(&self, n: &CallExpr) -> Option<MapCallSite> {
        let Callee::Expr(callee) = &n.callee else {
            return None;
        };
        let named = |sym: &str| self.map_methods().iter().any(|m| m == sym);
        let is_map_name = match callee.as_ref() {
            Expr::Member(m) => matches!(&m.prop, MemberProp::Ident(i) if named(&i.sym)),
            Expr::Ident(i) => named(&i.sym),
            _ => false,
        };
        if !is_map_name {
            return None;
        }

        if n.args.first().is_some_and(Self::is_fn_literal) {
            return Some(MapCallSite { arg_index: 0 });
        }
        if self.collection_first_enabled()
            && n.args.len() >= 2
            && Self::is_fn_literal(&n.args[1])
        {
            return Some(MapCallSite { arg_index: 1 });
        }
        None
    }

    // Parameter patterns of a function literal, in source order.
    fn fn_params(callback: &Expr) -> Option<Vec<&Pat>> {
        match callback {
            Expr::Arrow(arrow) => Some(arrow.params.iter().collect()),
            Expr::Fn(fn_expr) => Some(fn_expr.function.params.iter().map(|p| &p.pat).collect()),
            _ => None,
        }
    }

    // The index parameter the source already declares, if it binds a plain identifier.
    // A default value is unwrapped: `(item, index = 0) => …` still binds `index`, and the
    // default only applies when the callee passes nothing — exactly the case where the
    // guarded segment emitted by make_dynamic_attr would have dropped out anyway.
    fn declared_index_ident(callback: &Expr) -> Option<Ident> {
        fn ident_of(pat: &Pat) -> Option<Ident> {
            match pat {
                Pat::Ident(binding) => Some(binding.id.clone()),
                Pat::Assign(assign) => ident_of(&assign.left),
                _ => None,
            }
        }
        ident_of(Self::fn_params(callback)?.get(1)?)
    }

    // A rest parameter collects the index instead of naming it: `(...args) => …` reads it
    // as `args[1]`, and `(item, ...rest) => …` as `rest[0]`. Nothing may be appended after
    // a rest element, but nothing needs to be — the index is already in scope.
    fn rest_index_source(callback: &Expr) -> Option<IndexSource> {
        let params = Self::fn_params(callback)?;
        // Only positions 0 and 1 can hold the index; a rest element later collects
        // arguments from position 2 onward, which is the collection, not the index.
        for (position, pat) in params.iter().enumerate().take(2) {
            if let Pat::Rest(rest) = pat {
                if let Pat::Ident(binding) = rest.arg.as_ref() {
                    return Some(IndexSource::RestElement {
                        rest: binding.id.clone(),
                        position: (1 - position) as f64,
                    });
                }
            }
        }
        None
    }

    // A callback can take an appended index parameter only when it declares fewer than
    // two parameters and none is a rest element — nothing may follow `...rest`.
    fn can_take_index_param(callback: &Expr) -> bool {
        match Self::fn_params(callback) {
            Some(params) => {
                params.len() < 2 && !params.iter().any(|p| matches!(p, Pat::Rest(_)))
            }
            None => false,
        }
    }

    // SyntaxContext of the callback's own parameter scope, so an appended binding is
    // hygienically indistinguishable from one written in the source. Falls back to the
    // empty context when the callback declares no identifier parameters to copy from;
    // binding and reference still agree, which is what hygiene and minifiers require.
    fn params_ctxt(callback: &Expr) -> SyntaxContext {
        let mut scan = IdentScan::default();
        if let Some(params) = Self::fn_params(callback) {
            for param in params {
                param.visit_with(&mut scan);
            }
        }
        scan.first_ctxt.unwrap_or_default()
    }

    // Generates an identifier that appears nowhere inside `callback`, so appending it
    // cannot shadow a binding the body relies on. The counter additionally keeps names
    // distinct across nested callbacks, where scanning an inner callback cannot yet see
    // an outer generated name — its references are only added later in the traversal.
    fn fresh_ident(&mut self, base: &str, callback: &Expr) -> Ident {
        let mut scan = IdentScan::default();
        callback.visit_with(&mut scan);
        let ctxt = Self::params_ctxt(callback);
        loop {
            let name = format!("{}{}", base, self.next_generated_id);
            self.next_generated_id += 1;
            if !scan.syms.contains(name.as_str()) {
                return Ident::new(name.into(), DUMMY_SP, ctxt);
            }
        }
    }

    // Appends the index parameter, adding a placeholder first parameter when the
    // callback declares none: `() => …` must become `(__testidItem2, __testidIdx1) => …`.
    // The placeholder carries the *higher* counter value because the index was allocated
    // before the callback was visited, while this placeholder is allocated now, after.
    // Do not "fix" those digits to read 0, 1 — see tests/fixture/map-no-index-param.
    fn append_index_param(&mut self, callback: &mut Expr, index: Ident) {
        if Self::fn_params(callback).is_some_and(|p| p.is_empty()) {
            let placeholder = self.fresh_ident(ITEM_PARAM_BASE, callback);
            Self::push_param(callback, placeholder);
        }
        Self::push_param(callback, index);
    }

    fn push_param(callback: &mut Expr, ident: Ident) {
        let pat = Pat::Ident(BindingIdent {
            id: ident,
            type_ann: None,
        });
        match callback {
            Expr::Arrow(arrow) => arrow.params.push(pat),
            Expr::Fn(fn_expr) => fn_expr.function.params.push(Param {
                span: DUMMY_SP,
                decorators: Vec::new(),
                pat,
            }),
            _ => {}
        }
    }

    fn auto_index_enabled(&self) -> bool {
        self.options.auto_index.unwrap_or(true)
    }

    fn collection_first_enabled(&self) -> bool {
        self.options.collection_first_map.unwrap_or(true)
    }

    fn nested_indexes_enabled(&self) -> bool {
        self.options.nested_indexes.unwrap_or(true)
    }

    fn map_methods(&self) -> &[String] {
        match self.options.map_methods.as_deref() {
            Some(methods) => methods,
            None => &self.default_map_methods,
        }
    }

    fn inject_attributes(&mut self, opening: &mut JSXOpeningElement) {
        let component = match self.current_component.clone() {
            Some(c) => c,
            None => return,
        };
        let element_name = match Self::element_name(&opening.name) {
            Some(n) => n,
            None => return,
        };

        let unique_id = self.unique_element_id(&component, &element_name);
        let test_id = format!("{}.{}", component, unique_id);
        let attributes = self.attributes();

        if self.options.manifest_dir.is_some() {
            self.generated_testids.push(test_id.clone());
        }

        for attr_name in attributes {
            if !Self::has_attribute(&opening.attrs, &attr_name) {
                // With nestedIndexes off, only the innermost callback contributes — the
                // pre-0.2.0 id, which repeats across outer iterations.
                let levels = if self.nested_indexes_enabled() {
                    0
                } else {
                    self.map_indices.len().saturating_sub(1)
                };
                let attr = if self.map_indices.is_empty() {
                    Self::make_attr(&attr_name, &test_id)
                } else {
                    // Only an index the source declares is taken on trust; anything the
                    // plugin inferred is guarded so a wrong guess drops the segment.
                    let segments: Vec<(Expr, bool)> = self.map_indices[levels..]
                        .iter()
                        .map(|m| (m.expr(), m.needs_guard()))
                        .collect();
                    // Every contributing index is now load-bearing, so a parameter the
                    // plugin appended must be written back.
                    for m in &mut self.map_indices[levels..] {
                        m.used = true;
                    }
                    Self::make_dynamic_attr(&attr_name, &test_id, &segments)
                };
                opening.attrs.insert(0, attr);
            }
        }
    }

    // Sets the active component and resets its counters. Returns the previous component
    // so the caller can restore it after traversal (handles nested components).
    fn enter_component(&mut self, name: String) -> Option<String> {
        self.component_counters.insert(name.clone(), HashMap::new());
        self.current_component.replace(name)
    }
}

impl VisitMut for ReactDataTestIdTransform {
    // function MyComponent() { ... }
    fn visit_mut_fn_decl(&mut self, n: &mut FnDecl) {
        let prev = self.enter_component(n.ident.sym.to_string());
        n.visit_mut_children_with(self);
        self.current_component = prev;
    }

    // const MyComponent = () => { ... }
    // const MyComponent = function() { ... }
    fn visit_mut_var_declarator(&mut self, n: &mut VarDeclarator) {
        if let Pat::Ident(ident) = &n.name {
            if let Some(init) = &n.init {
                if matches!(init.as_ref(), Expr::Arrow(_) | Expr::Fn(_)) {
                    let prev = self.enter_component(ident.sym.to_string());
                    n.visit_mut_children_with(self);
                    self.current_component = prev;
                    return;
                }
            }
        }
        n.visit_mut_children_with(self);
    }

    // class MyComponent extends React.Component { render() { ... } }
    // Records the class name so visit_mut_class_method can activate it for render().
    fn visit_mut_class_decl(&mut self, n: &mut ClassDecl) {
        let prev = self.current_class.replace(n.ident.sym.to_string());
        n.visit_mut_children_with(self);
        self.current_class = prev;
    }

    // Only attributes JSX inside the render() method, matching the Babel plugin's behavior.
    fn visit_mut_class_method(&mut self, n: &mut ClassMethod) {
        if let PropName::Ident(ref key) = n.key {
            if key.sym.as_ref() == "render" {
                if let Some(class_name) = self.current_class.clone() {
                    let prev = self.enter_component(class_name);
                    n.visit_mut_children_with(self);
                    self.current_component = prev;
                    return;
                }
            }
        }
        n.visit_mut_children_with(self);
    }

    // export default () => { ... }
    fn visit_mut_export_default_expr(&mut self, n: &mut ExportDefaultExpr) {
        if matches!(n.expr.as_ref(), Expr::Arrow(_) | Expr::Fn(_)) {
            if let Some(name) = self.filename_component_name.clone() {
                let prev = self.enter_component(name);
                n.visit_mut_children_with(self);
                self.current_component = prev;
                return;
            }
        }
        n.visit_mut_children_with(self);
    }

    // export default function() { ... }
    // export default function Named() { ... }
    fn visit_mut_export_default_decl(&mut self, n: &mut ExportDefaultDecl) {
        if let DefaultDecl::Fn(fn_expr) = &n.decl {
            let name = fn_expr
                .ident
                .as_ref()
                .map(|id| id.sym.to_string())
                .or_else(|| self.filename_component_name.clone());
            if let Some(name) = name {
                let prev = self.enter_component(name);
                n.visit_mut_children_with(self);
                self.current_component = prev;
                return;
            }
        }
        n.visit_mut_children_with(self);
    }

    // Detects map-like calls and makes the callback's index available to
    // inject_attributes, so JSX inside gets {"Comp.el-" + index} instead of one static
    // id repeated for every item. When the callback declares no index parameter, one is
    // appended — but only if a testid ends up referencing it.
    fn visit_mut_call_expr(&mut self, n: &mut CallExpr) {
        let Some(site) = self.map_call_site(n) else {
            n.visit_mut_children_with(self);
            return;
        };

        // Prefer an index already in scope, in order of confidence: one the source names,
        // then one a rest parameter collects, then one appended as a last resort.
        let callback = &n.args[site.arg_index].expr;
        let source = Self::declared_index_ident(callback)
            .map(IndexSource::Declared)
            .or_else(|| Self::rest_index_source(callback))
            .or_else(|| {
                (self.auto_index_enabled() && Self::can_take_index_param(callback))
                    .then(|| IndexSource::Appended(self.fresh_ident(INDEX_PARAM_BASE, callback)))
            });
        let mut index = source.map(|source| MapIndex { source, used: false });

        // The index binding exists ONLY inside the callback. Visiting the callee or the
        // remaining arguments with it in scope emits a reference to a variable that does
        // not exist there — e.g. [<Header/>, ...xs].map((x, i) => ...) would give <Header>
        // a testid of {"C.Header-" + i}, throwing ReferenceError at render. Children are
        // visited in their natural order (callee, args, type_args) so element numbering
        // is unchanged.
        n.callee.visit_mut_with(self);
        for (idx, arg) in n.args.iter_mut().enumerate() {
            if idx != site.arg_index {
                arg.visit_mut_with(self);
                continue;
            }
            let Some(entry) = index.take() else {
                arg.visit_mut_with(self);
                continue;
            };
            let appended = match &entry.source {
                IndexSource::Appended(ident) => Some(ident.clone()),
                _ => None,
            };
            self.map_indices.push(entry);
            arg.visit_mut_with(self);
            let entry = self
                .map_indices
                .pop()
                .expect("map index stack popped by a nested visit");
            // A callback whose JSX (if any) never used the index keeps its original
            // signature — no stray parameter in the output.
            if let (Some(ident), true) = (appended, entry.used) {
                self.append_index_param(&mut arg.expr, ident);
            }
        }
        n.type_args.visit_mut_with(self);
    }

    fn visit_mut_jsx_element(&mut self, n: &mut JSXElement) {
        self.inject_attributes(&mut n.opening);
        n.visit_mut_children_with(self);
    }
}

// Deserializes the host's plugin config. A failure here discards EVERY option, not just
// the bad one — `{"autoIndex": "false"}` (a string where a bool belongs) is valid JSON, so
// it reaches us and then fails typed deserialization. Swallowing that leaves the user with
// a config that appears accepted and does nothing, which is the silent no-op this repo has
// been bitten by before. Report it, then fall back to defaults.
fn parse_options(config_json: Option<String>) -> PluginOptions {
    let Some(raw) = config_json else {
        return PluginOptions::default();
    };
    match serde_json::from_str(&raw) {
        Ok(options) => options,
        Err(e) => {
            report_error(&format!(
                "could not read the plugin config, so ALL options were ignored \
                 (including any you set). Parse error: {e}. Config received: {raw}",
            ));
            PluginOptions::default()
        }
    }
}

// Reports a non-fatal plugin diagnostic. Inside the SWC plugin runner the host installs
// a HANDLER; in native `cargo test` builds it is absent, so fall back to stderr rather
// than panicking (HANDLER.with panics when unset).
// Reports a plugin diagnostic that the user will actually see.
//
// This deliberately uses `err`, not `warn`: @swc/core discards plugin warnings entirely
// (verified — HANDLER.is_set() is true and warn() is called, but nothing reaches stdout
// or stderr), so a warning here would be indistinguishable from the silent failure this
// is meant to replace. `err` surfaces the full message and aborts the build.
//
// In native `cargo test` builds no HANDLER is installed, so fall back to stderr instead
// of panicking (HANDLER.with panics when unset).
fn report_error(msg: &str) {
    use swc_core::plugin::errors::HANDLER;
    if HANDLER.is_set() {
        HANDLER.with(|handler| handler.err(msg));
    } else {
        eprintln!("swc-plugin-react-data-testid: {}", msg);
    }
}

// Writes a per-file manifest JSON into `dir`. Each source file gets its own file so
// parallel transforms don't conflict. Merge them with scripts/merge-manifest.mjs.
//
// ⚠️  KNOWN BROKEN: SWC's plugin runner grants the Wasm module no preopened directories,
// so every std::fs call fails with WASI errno 76 (ENOTCAPABLE, "Capabilities
// insufficient") and the process cwd is "/". No value of `manifestDir` can work — this
// is architectural, not a bad path. A real fix cannot use std::fs; it needs SWC's
// plugin-comments/metadata channel or a host-side post-pass that collects testids
// outside the sandbox. Until then we at least surface the failure instead of swallowing
// it, so users are not left wondering why no manifest appeared.
fn write_manifest(dir: &str, source_file: Option<&str>, testids: &[String]) {
    use std::collections::HashSet;
    let file = source_file.unwrap_or("unknown");
    let mut seen = HashSet::new();
    let unique: Vec<&String> = testids.iter().filter(|t| seen.insert(t.as_str())).collect();
    let json = format!(
        "{{\"file\":{},\"testids\":[{}]}}",
        serde_json::to_string(file).unwrap_or_else(|_| "\"unknown\"".to_string()),
        unique
            .iter()
            .map(|t| serde_json::to_string(t).unwrap_or_default())
            .collect::<Vec<_>>()
            .join(",")
    );
    // Derive a safe filename: replace path separators and colons.
    let safe_name = file.replace(['/', '\\', ':'], "_");
    let out_path = format!("{}/{}.json", dir, safe_name);
    if let Err(err) = std::fs::create_dir_all(dir).and_then(|()| std::fs::write(&out_path, json)) {
        report_error(&format!(
            "`manifestDir` is not supported and the manifest was NOT written \
             (failed writing {out_path}: {err}). SWC runs Wasm plugins inside a WASI \
             sandbox with no preopened directories, so the plugin cannot write to any \
             path on disk — no value of `manifestDir` can work. Remove the `manifestDir` \
             option to unblock your build. See the Limitations section of the readme."
        ));
    }
}

#[plugin_transform]
pub fn process_transform(program: Program, metadata: TransformPluginProgramMetadata) -> Program {
    let options = parse_options(metadata.get_transform_plugin_config());
    let filename = metadata.get_context(&TransformPluginMetadataContextKind::Filename);
    let mut transform = ReactDataTestIdTransform::new_with_filename(options, filename);
    let mut program = program;
    program.visit_mut_with(&mut transform);
    if let Some(ref dir) = transform.options.manifest_dir {
        write_manifest(dir, transform.source_filename.as_deref(), &transform.generated_testids);
    }
    program
}

#[cfg(test)]
mod tests {
    use super::*;
    use swc_core::common::{Globals, Mark, SyntaxContext, GLOBALS};

    #[test]
    fn element_name_returns_none_for_namespaced() {
        let name = JSXElementName::JSXNamespacedName(JSXNamespacedName {
            span: DUMMY_SP,
            ns: IdentName { span: DUMMY_SP, sym: "svg".into() },
            name: IdentName { span: DUMMY_SP, sym: "image".into() },
        });
        assert_eq!(ReactDataTestIdTransform::element_name(&name), None);
    }

    #[test]
    fn has_attribute_returns_false_for_namespaced_attr_name() {
        let attr = JSXAttrOrSpread::JSXAttr(JSXAttr {
            span: DUMMY_SP,
            name: JSXAttrName::JSXNamespacedName(JSXNamespacedName {
                span: DUMMY_SP,
                ns: IdentName { span: DUMMY_SP, sym: "xml".into() },
                name: IdentName { span: DUMMY_SP, sym: "lang".into() },
            }),
            value: None,
        });
        assert!(!ReactDataTestIdTransform::has_attribute(&[attr], "lang"));
    }

    #[test]
    fn inject_attributes_skips_namespaced_element() {
        let mut transform = ReactDataTestIdTransform::new(PluginOptions::default());
        transform.current_component = Some("Test".to_string());
        let mut opening = JSXOpeningElement {
            span: DUMMY_SP,
            name: JSXElementName::JSXNamespacedName(JSXNamespacedName {
                span: DUMMY_SP,
                ns: IdentName { span: DUMMY_SP, sym: "svg".into() },
                name: IdentName { span: DUMMY_SP, sym: "image".into() },
            }),
            attrs: vec![],
            self_closing: true,
            type_args: None,
        };
        transform.inject_attributes(&mut opening);
        assert!(opening.attrs.is_empty());
    }

    #[test]
    fn parse_options_returns_default_for_none() {
        let opts = parse_options(None);
        assert!(opts.attributes.is_none());
    }

    #[test]
    fn parse_options_returns_default_for_invalid_json() {
        let opts = parse_options(Some("not json at all".to_string()));
        assert!(opts.attributes.is_none());
    }

    // The dangerous shape: valid JSON that fails typed deserialization. Serde aborts the
    // whole struct, so a single mistyped field silently discards the correct ones too —
    // hence the reported error. Falling back to defaults keeps the build's ids sane.
    #[test]
    fn parse_options_falls_back_when_a_field_has_the_wrong_type() {
        let opts = parse_options(Some(
            r#"{"autoIndex":"false","attributes":["data-cy"]}"#.to_string(),
        ));
        assert_eq!(opts.auto_index, None);
        assert!(
            opts.attributes.is_none(),
            "one bad field discards the rest — which is exactly why it must be reported"
        );
        assert!(ReactDataTestIdTransform::new(opts).auto_index_enabled());
    }

    #[test]
    fn parse_options_deserializes_auto_index() {
        // camelCase in JSON maps to the snake_case field
        assert_eq!(parse_options(Some(r#"{"autoIndex":false}"#.to_string())).auto_index, Some(false));
        assert_eq!(parse_options(Some("{}".to_string())).auto_index, None);
    }

    // Every option reaches the plugin only as JSON from the host config. A field name
    // that does not match is not a compile error — it is a silently ignored option, so
    // each one is pinned here with the exact JSON a user would write.
    #[test]
    fn parse_options_deserializes_every_inference_option() {
        let opts = parse_options(Some(
            r#"{"mapMethods":["map"],"collectionFirstMap":false,"nestedIndexes":false}"#
                .to_string(),
        ));
        assert_eq!(opts.map_methods, Some(vec!["map".to_string()]));
        assert_eq!(opts.collection_first_map, Some(false));
        assert_eq!(opts.nested_indexes, Some(false));

        // …and each one actually reaches the behaviour it names.
        let t = ReactDataTestIdTransform::new(opts);
        assert!(!t.collection_first_enabled());
        assert!(!t.nested_indexes_enabled());
        assert_eq!(t.map_methods(), ["map".to_string()]);
    }

    #[test]
    fn parse_options_deserializes_attributes() {
        let opts = parse_options(Some(r#"{"attributes":["data-cy","data-test"]}"#.to_string()));
        assert_eq!(
            opts.attributes,
            Some(vec!["data-cy".to_string(), "data-test".to_string()])
        );
    }

    // --- unique_element_id ---

    #[test]
    fn unique_element_id_first_occurrence_no_suffix() {
        let mut t = ReactDataTestIdTransform::new(PluginOptions::default());
        assert_eq!(t.unique_element_id("Comp", "div"), "div");
    }

    #[test]
    fn unique_element_id_second_occurrence_suffix_2() {
        let mut t = ReactDataTestIdTransform::new(PluginOptions::default());
        t.unique_element_id("Comp", "div");
        assert_eq!(t.unique_element_id("Comp", "div"), "div2");
    }

    #[test]
    fn unique_element_id_third_occurrence_suffix_3() {
        let mut t = ReactDataTestIdTransform::new(PluginOptions::default());
        t.unique_element_id("Comp", "div");
        t.unique_element_id("Comp", "div");
        assert_eq!(t.unique_element_id("Comp", "div"), "div3");
    }

    #[test]
    fn unique_element_id_counters_isolated_per_component() {
        let mut t = ReactDataTestIdTransform::new(PluginOptions::default());
        assert_eq!(t.unique_element_id("Foo", "div"), "div");
        assert_eq!(t.unique_element_id("Bar", "div"), "div");
        assert_eq!(t.unique_element_id("Foo", "div"), "div2");
        assert_eq!(t.unique_element_id("Bar", "div"), "div2");
    }

    #[test]
    fn unique_element_id_different_element_types_independent() {
        let mut t = ReactDataTestIdTransform::new(PluginOptions::default());
        assert_eq!(t.unique_element_id("Comp", "div"), "div");
        assert_eq!(t.unique_element_id("Comp", "span"), "span");
        assert_eq!(t.unique_element_id("Comp", "div"), "div2");
        assert_eq!(t.unique_element_id("Comp", "span"), "span2");
    }

    // --- enter_component ---

    #[test]
    fn enter_component_returns_previous_component() {
        let mut t = ReactDataTestIdTransform::new(PluginOptions::default());
        assert_eq!(t.enter_component("First".to_string()), None);
        assert_eq!(
            t.enter_component("Second".to_string()),
            Some("First".to_string())
        );
    }

    #[test]
    fn enter_component_resets_counters_for_that_component() {
        let mut t = ReactDataTestIdTransform::new(PluginOptions::default());
        t.unique_element_id("Comp", "div");
        t.unique_element_id("Comp", "div");
        t.enter_component("Comp".to_string());
        assert_eq!(t.unique_element_id("Comp", "div"), "div");
    }

    // --- component_name_from_filename ---

    #[test]
    fn component_name_from_filename_uppercases_first_char() {
        assert_eq!(
            component_name_from_filename("/src/button.tsx"),
            Some("Button".to_string())
        );
    }

    #[test]
    fn component_name_from_filename_already_uppercase_unchanged() {
        assert_eq!(
            component_name_from_filename("/src/MyComponent.tsx"),
            Some("MyComponent".to_string())
        );
    }

    #[test]
    fn component_name_from_filename_no_extension_returns_none() {
        assert_eq!(component_name_from_filename("button"), None);
    }

    #[test]
    fn component_name_from_filename_multi_dot_stem_returns_none() {
        assert_eq!(component_name_from_filename("index.test.tsx"), None);
    }

    #[test]
    fn component_name_from_filename_empty_stem_returns_none() {
        assert_eq!(component_name_from_filename(".hidden"), None);
    }

    // --- attributes() ---

    #[test]
    fn attributes_falls_back_to_default_when_none() {
        let t = ReactDataTestIdTransform::new(PluginOptions { attributes: None, ..Default::default() });
        assert_eq!(t.attributes(), vec!["data-testid".to_string()]);
    }

    #[test]
    fn attributes_falls_back_to_default_when_empty_vec() {
        let t = ReactDataTestIdTransform::new(PluginOptions { attributes: Some(vec![]), ..Default::default() });
        assert_eq!(t.attributes(), vec!["data-testid".to_string()]);
    }

    #[test]
    fn attributes_returns_custom_list() {
        let t = ReactDataTestIdTransform::new(PluginOptions {
            attributes: Some(vec!["data-cy".to_string(), "data-pw".to_string()]),
            ..Default::default()
        });
        assert_eq!(
            t.attributes(),
            vec!["data-cy".to_string(), "data-pw".to_string()]
        );
    }

    // --- has_attribute ---

    #[test]
    fn has_attribute_returns_true_when_attr_present() {
        let attr = JSXAttrOrSpread::JSXAttr(JSXAttr {
            span: DUMMY_SP,
            name: JSXAttrName::Ident(IdentName { span: DUMMY_SP, sym: "data-testid".into() }),
            value: None,
        });
        assert!(ReactDataTestIdTransform::has_attribute(&[attr], "data-testid"));
    }

    #[test]
    fn has_attribute_returns_false_for_different_attr() {
        let attr = JSXAttrOrSpread::JSXAttr(JSXAttr {
            span: DUMMY_SP,
            name: JSXAttrName::Ident(IdentName { span: DUMMY_SP, sym: "className".into() }),
            value: None,
        });
        assert!(!ReactDataTestIdTransform::has_attribute(&[attr], "data-testid"));
    }

    // --- inject_attributes ---

    #[test]
    fn inject_attributes_skips_when_no_current_component() {
        let mut t = ReactDataTestIdTransform::new(PluginOptions::default());
        let mut opening = JSXOpeningElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(IdentName { span: DUMMY_SP, sym: "div".into() }.into()),
            attrs: vec![],
            self_closing: true,
            type_args: None,
        };
        t.inject_attributes(&mut opening);
        assert!(opening.attrs.is_empty());
    }

    #[test]
    fn inject_attributes_inserts_at_position_zero() {
        let mut t = ReactDataTestIdTransform::new(PluginOptions::default());
        t.current_component = Some("Comp".to_string());
        let existing = JSXAttrOrSpread::JSXAttr(JSXAttr {
            span: DUMMY_SP,
            name: JSXAttrName::Ident(IdentName { span: DUMMY_SP, sym: "className".into() }),
            value: None,
        });
        let mut opening = JSXOpeningElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(IdentName { span: DUMMY_SP, sym: "div".into() }.into()),
            attrs: vec![existing],
            self_closing: true,
            type_args: None,
        };
        t.inject_attributes(&mut opening);
        assert_eq!(opening.attrs.len(), 2);
        if let JSXAttrOrSpread::JSXAttr(attr) = &opening.attrs[0] {
            if let JSXAttrName::Ident(name) = &attr.name {
                assert_eq!(name.sym.as_ref(), "data-testid");
            } else {
                panic!("expected Ident attr name");
            }
        } else {
            panic!("expected JSXAttr at index 0");
        }
    }

    #[test]
    fn inject_attributes_skips_element_already_having_attr() {
        let mut t = ReactDataTestIdTransform::new(PluginOptions::default());
        t.current_component = Some("Comp".to_string());
        let existing = JSXAttrOrSpread::JSXAttr(JSXAttr {
            span: DUMMY_SP,
            name: JSXAttrName::Ident(IdentName { span: DUMMY_SP, sym: "data-testid".into() }),
            value: None,
        });
        let mut opening = JSXOpeningElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(IdentName { span: DUMMY_SP, sym: "div".into() }.into()),
            attrs: vec![existing],
            self_closing: true,
            type_args: None,
        };
        t.inject_attributes(&mut opening);
        assert_eq!(opening.attrs.len(), 1, "should not add a second data-testid");
    }

    // --- manifest collection ---

    #[test]
    fn manifest_collects_generated_testids() {
        let mut t = ReactDataTestIdTransform::new(PluginOptions {
            manifest_dir: Some("/tmp/test-manifest".to_string()),
            ..Default::default()
        });
        t.current_component = Some("Form".to_string());
        let make_opening = |tag: &str| JSXOpeningElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(IdentName { span: DUMMY_SP, sym: tag.into() }.into()),
            attrs: vec![],
            self_closing: true,
            type_args: None,
        };
        t.inject_attributes(&mut make_opening("input"));
        t.inject_attributes(&mut make_opening("input"));
        t.inject_attributes(&mut make_opening("button"));
        assert_eq!(
            t.generated_testids,
            vec!["Form.input", "Form.input2", "Form.button"]
        );
    }

    #[test]
    fn manifest_not_collected_when_manifest_dir_none() {
        let mut t = ReactDataTestIdTransform::new(PluginOptions::default());
        t.current_component = Some("Form".to_string());
        let mut opening = JSXOpeningElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(IdentName { span: DUMMY_SP, sym: "div".into() }.into()),
            attrs: vec![],
            self_closing: true,
            type_args: None,
        };
        t.inject_attributes(&mut opening);
        assert!(t.generated_testids.is_empty());
    }

    // --- map call detection ---

    fn ident_expr(sym: &str) -> Box<Expr> {
        Box::new(Expr::Ident(
            IdentName { span: DUMMY_SP, sym: sym.into() }.into(),
        ))
    }

    fn arrow(params: Vec<Pat>) -> Box<Expr> {
        Box::new(Expr::Arrow(ArrowExpr {
            span: DUMMY_SP,
            ctxt: Default::default(),
            params,
            body: Box::new(BlockStmtOrExpr::Expr(Box::new(Expr::Lit(Lit::Null(Null {
                span: DUMMY_SP,
            }))))),
            is_async: false,
            is_generator: false,
            type_params: None,
            return_type: None,
        }))
    }

    fn param(sym: &str) -> Pat {
        Pat::Ident(BindingIdent {
            id: IdentName { span: DUMMY_SP, sym: sym.into() }.into(),
            type_ann: None,
        })
    }

    // `obj.method(args...)`, or `method(args...)` when obj is None.
    fn call(obj: Option<&str>, method: &str, args: Vec<Box<Expr>>) -> CallExpr {
        let callee = match obj {
            Some(obj) => Box::new(Expr::Member(MemberExpr {
                span: DUMMY_SP,
                obj: ident_expr(obj),
                prop: MemberProp::Ident(IdentName { span: DUMMY_SP, sym: method.into() }),
            })),
            None => ident_expr(method),
        };
        CallExpr {
            span: DUMMY_SP,
            ctxt: Default::default(),
            callee: Callee::Expr(callee),
            args: args
                .into_iter()
                .map(|expr| ExprOrSpread { spread: None, expr })
                .collect(),
            type_args: None,
        }
    }

    fn site(call: &CallExpr) -> Option<usize> {
        ReactDataTestIdTransform::new(PluginOptions::default())
            .map_call_site(call)
            .map(|s| s.arg_index)
    }

    fn site_with(options: PluginOptions, call: &CallExpr) -> Option<usize> {
        ReactDataTestIdTransform::new(options)
            .map_call_site(call)
            .map(|s| s.arg_index)
    }

    #[test]
    fn map_call_site_method_callback_first() {
        // items.map(cb)
        let c = call(Some("items"), "map", vec![arrow(vec![param("item")])]);
        assert_eq!(site(&c), Some(0));
    }

    #[test]
    fn map_call_site_detects_flat_map() {
        // items.flatMap(cb)
        let c = call(Some("items"), "flatMap", vec![arrow(vec![param("item")])]);
        assert_eq!(site(&c), Some(0));
    }

    #[test]
    fn map_call_site_lodash_style_callback_second() {
        // _.map(items, cb)
        let c = call(Some("_"), "map", vec![ident_expr("items"), arrow(vec![param("item")])]);
        assert_eq!(site(&c), Some(1));
    }

    #[test]
    fn map_call_site_bare_identifier_callback_second() {
        // map(items, cb) — named import of lodash's map
        let c = call(None, "map", vec![ident_expr("items"), arrow(vec![param("item")])]);
        assert_eq!(site(&c), Some(1));
    }

    // A bare map(cb) is RxJS's operator (which passes an index) or curried ramda /
    // lodash-fp (which do not). The syntax is identical, so detection cannot tell them
    // apart — it recognises all of them and the guarded segment absorbs the difference.
    #[test]
    fn map_call_site_detects_a_callback_first_bare_call() {
        // map(cb, items) — ramda order, or a curried operator
        let c = call(None, "map", vec![arrow(vec![param("item")]), ident_expr("items")]);
        assert_eq!(site(&c), Some(0));
    }

    #[test]
    fn map_call_site_detects_a_callback_alongside_a_second_argument() {
        // R.map(cb, items) / items.map(cb, thisArg)
        let c = call(Some("R"), "map", vec![arrow(vec![param("item")]), ident_expr("items")]);
        assert_eq!(site(&c), Some(0));
    }

    #[test]
    fn map_call_site_returns_none_for_non_map_call() {
        let c = call(Some("items"), "filter", vec![arrow(vec![param("item")])]);
        assert_eq!(site(&c), None);
    }

    #[test]
    fn map_call_site_returns_none_without_a_function_literal() {
        // items.map(renderItem)
        let c = call(Some("items"), "map", vec![ident_expr("renderItem")]);
        assert_eq!(site(&c), None);
    }

    // --- opt-out options: each narrows detection, none is needed for correctness ---

    #[test]
    fn map_methods_option_narrows_which_names_are_detected() {
        let flat = call(Some("items"), "flatMap", vec![arrow(vec![param("item")])]);
        let plain = call(Some("items"), "map", vec![arrow(vec![param("item")])]);
        let only_map = || PluginOptions {
            map_methods: Some(vec!["map".to_string()]),
            ..Default::default()
        };
        assert_eq!(site_with(only_map(), &flat), None, "flatMap must be excluded");
        assert_eq!(site_with(only_map(), &plain), Some(0), "map must still be detected");

        // and it can widen, for a project-local iterator
        let custom = call(Some("items"), "mapIndexed", vec![arrow(vec![param("item")])]);
        assert_eq!(site(&custom), None);
        assert_eq!(
            site_with(
                PluginOptions {
                    map_methods: Some(vec!["mapIndexed".to_string()]),
                    ..Default::default()
                },
                &custom
            ),
            Some(0)
        );
    }

    #[test]
    fn collection_first_map_option_disables_the_argument_one_form() {
        let c = call(Some("_"), "map", vec![ident_expr("items"), arrow(vec![param("item")])]);
        assert_eq!(site(&c), Some(1));
        assert_eq!(
            site_with(
                PluginOptions {
                    collection_first_map: Some(false),
                    ..Default::default()
                },
                &c
            ),
            None
        );
    }

    #[test]
    fn empty_map_methods_disables_detection_entirely() {
        let c = call(Some("items"), "map", vec![arrow(vec![param("item")])]);
        assert_eq!(
            site_with(
                PluginOptions {
                    map_methods: Some(Vec::new()),
                    ..Default::default()
                },
                &c
            ),
            None
        );
    }

    // --- declared_index_ident / can_take_index_param ---

    #[test]
    fn declared_index_ident_returns_second_param() {
        let cb = arrow(vec![param("item"), param("idx")]);
        assert_eq!(
            ReactDataTestIdTransform::declared_index_ident(&cb).map(|i| i.sym.to_string()),
            Some("idx".to_string())
        );
    }

    #[test]
    fn declared_index_ident_returns_none_when_only_one_param() {
        let cb = arrow(vec![param("item")]);
        assert!(ReactDataTestIdTransform::declared_index_ident(&cb).is_none());
    }

    #[test]
    fn declared_index_ident_returns_none_for_destructured_second_param() {
        let cb = arrow(vec![
            param("item"),
            Pat::Array(ArrayPat {
                span: DUMMY_SP,
                elems: vec![],
                optional: false,
                type_ann: None,
            }),
        ]);
        assert!(ReactDataTestIdTransform::declared_index_ident(&cb).is_none());
    }

    #[test]
    fn declared_index_ident_unwraps_a_default_value() {
        // `(item, index = 0) => …` still binds `index`; the default only applies when
        // the callee passes nothing, so referencing it is always safe.
        let cb = arrow(vec![
            param("item"),
            Pat::Assign(AssignPat {
                span: DUMMY_SP,
                left: Box::new(param("index")),
                right: Box::new(Expr::Lit(Lit::Num(Number {
                    span: DUMMY_SP,
                    value: 0.0,
                    raw: None,
                }))),
            }),
        ]);
        assert_eq!(
            ReactDataTestIdTransform::declared_index_ident(&cb).map(|i| i.sym.to_string()),
            Some("index".to_string())
        );
    }

    #[test]
    fn can_take_index_param_accepts_zero_or_one_param() {
        assert!(ReactDataTestIdTransform::can_take_index_param(&arrow(vec![])));
        assert!(ReactDataTestIdTransform::can_take_index_param(&arrow(vec![param("item")])));
    }

    #[test]
    fn can_take_index_param_rejects_rest_param() {
        // Nothing may follow `...rest` in a parameter list.
        let cb = arrow(vec![Pat::Rest(RestPat {
            span: DUMMY_SP,
            dot3_token: DUMMY_SP,
            arg: Box::new(param("args")),
            type_ann: None,
        })]);
        assert!(!ReactDataTestIdTransform::can_take_index_param(&cb));
    }

    #[test]
    fn can_take_index_param_rejects_callback_that_already_has_two() {
        let cb = arrow(vec![param("item"), param("idx")]);
        assert!(!ReactDataTestIdTransform::can_take_index_param(&cb));
    }

    // --- fresh_ident ---

    #[test]
    fn fresh_ident_avoids_names_used_inside_the_callback() {
        let mut t = ReactDataTestIdTransform::new(PluginOptions::default());
        // A callback that already references __testidIdx0 must not have that name
        // appended, or the appended parameter would shadow the outer binding.
        let cb = arrow(vec![param("__testidIdx0")]);
        assert_eq!(
            t.fresh_ident(INDEX_PARAM_BASE, &cb).sym.to_string(),
            "__testidIdx1"
        );
    }

    #[test]
    fn fresh_ident_copies_the_parameter_scope_ctxt() {
        let ctxt = GLOBALS.set(&Globals::new(), || {
            SyntaxContext::empty().apply_mark(Mark::new())
        });
        let mut item: Ident = IdentName { span: DUMMY_SP, sym: "item".into() }.into();
        item.ctxt = ctxt;
        let cb = arrow(vec![Pat::Ident(BindingIdent { id: item, type_ann: None })]);

        let mut t = ReactDataTestIdTransform::new(PluginOptions::default());
        assert_eq!(t.fresh_ident(INDEX_PARAM_BASE, &cb).ctxt, ctxt);
    }

    #[test]
    fn fresh_ident_names_are_unique_across_calls() {
        let mut t = ReactDataTestIdTransform::new(PluginOptions::default());
        let cb = arrow(vec![param("item")]);
        let a = t.fresh_ident(INDEX_PARAM_BASE, &cb).sym.to_string();
        let b = t.fresh_ident(INDEX_PARAM_BASE, &cb).sym.to_string();
        assert_ne!(a, b);
    }

    // --- append_index_param ---

    #[test]
    fn append_index_param_adds_placeholder_when_callback_has_no_params() {
        let mut t = ReactDataTestIdTransform::new(PluginOptions::default());
        let mut cb = *arrow(vec![]);
        let index: Ident = IdentName { span: DUMMY_SP, sym: "__testidIdx0".into() }.into();
        t.append_index_param(&mut cb, index);
        let params = ReactDataTestIdTransform::fn_params(&cb).unwrap();
        assert_eq!(params.len(), 2, "() => … must become (placeholder, index) => …");
        assert!(matches!(params[1], Pat::Ident(b) if b.id.sym.as_ref() == "__testidIdx0"));
    }

    #[test]
    fn append_index_param_keeps_existing_param() {
        let mut t = ReactDataTestIdTransform::new(PluginOptions::default());
        let mut cb = *arrow(vec![param("item")]);
        let index: Ident = IdentName { span: DUMMY_SP, sym: "__testidIdx0".into() }.into();
        t.append_index_param(&mut cb, index);
        let params = ReactDataTestIdTransform::fn_params(&cb).unwrap();
        assert_eq!(params.len(), 2);
        assert!(matches!(params[0], Pat::Ident(b) if b.id.sym.as_ref() == "item"));
    }

    #[test]
    fn auto_index_can_be_disabled() {
        let t = ReactDataTestIdTransform::new(PluginOptions {
            auto_index: Some(false),
            ..Default::default()
        });
        assert!(!t.auto_index_enabled());

        // Every option defaults to the maximally-inferring behaviour: the plugin must
        // deliver unique ids out of the box, with flags available only to turn that off.
        let d = ReactDataTestIdTransform::new(PluginOptions::default());
        assert!(d.auto_index_enabled());
        assert!(d.collection_first_enabled());
        assert!(d.nested_indexes_enabled());
        assert_eq!(d.map_methods(), ["map".to_string(), "flatMap".to_string()]);
    }

    // Regression guard: the generated index reference must reuse the parameter's own
    // Ident (same sym AND SyntaxContext). If a fresh Ident is synthesized instead, the
    // reference does not resolve to the binding, and a renaming minifier rewrites the
    // parameter but not the reference — producing `"Comp.el-" + i` where `i` no longer
    // exists. That fails only in a minified production build, so no fixture catches it.
    #[test]
    fn make_dynamic_attr_preserves_index_ident_ctxt() {
        let mut index_ident: Ident =
            IdentName { span: DUMMY_SP, sym: "i".into() }.into();
        // Mark::new() allocates from thread-local hygiene state, which requires GLOBALS.
        index_ident.ctxt = GLOBALS.set(&Globals::new(), || {
            SyntaxContext::empty().apply_mark(Mark::new())
        });
        assert_ne!(
            index_ident.ctxt,
            SyntaxContext::empty(),
            "test setup must use a non-empty ctxt or it proves nothing"
        );

        let attr = ReactDataTestIdTransform::make_dynamic_attr(
            "data-testid",
            "List.li",
            &[(Expr::Ident(index_ident.clone()), false)],
        );

        let JSXAttrOrSpread::JSXAttr(attr) = attr else {
            panic!("expected a JSXAttr");
        };
        let Some(JSXAttrValue::JSXExprContainer(c)) = attr.value else {
            panic!("expected an expression container");
        };
        let JSXExpr::Expr(expr) = c.expr else {
            panic!("expected an expression");
        };
        let Expr::Bin(bin) = *expr else {
            panic!("expected a binary expression");
        };
        let Expr::Ident(emitted) = *bin.right else {
            panic!("expected an identifier on the right-hand side");
        };

        assert_eq!(emitted.sym, index_ident.sym);
        assert_eq!(
            emitted.ctxt, index_ident.ctxt,
            "generated reference must carry the parameter's SyntaxContext"
        );
    }

    // An appended index is a static guess about the callee. If the guess is wrong the id
    // must fall back to the plain static id, not "List.li-undefined" on every item — so
    // the segment is emitted as a conditional, and both of its references must carry the
    // parameter's own SyntaxContext or a minifier will break one of them.
    #[test]
    fn make_dynamic_attr_guards_an_injected_index() {
        let mut index_ident: Ident = IdentName {
            span: DUMMY_SP,
            sym: "__testidIdx0".into(),
        }
        .into();
        index_ident.ctxt =
            GLOBALS.set(&Globals::new(), || SyntaxContext::empty().apply_mark(Mark::new()));

        let attr = ReactDataTestIdTransform::make_dynamic_attr(
            "data-testid",
            "List.li",
            &[(Expr::Ident(index_ident.clone()), true)],
        );

        let JSXAttrOrSpread::JSXAttr(attr) = attr else {
            panic!("expected a JSXAttr");
        };
        let Some(JSXAttrValue::JSXExprContainer(c)) = attr.value else {
            panic!("expected an expression container");
        };
        let JSXExpr::Expr(expr) = c.expr else {
            panic!("expected an expression");
        };
        // "List.li" + (…)  — the leading literal keeps every `+` a string concatenation
        let Expr::Bin(bin) = *expr else {
            panic!("expected a binary expression");
        };
        assert!(
            matches!(bin.left.as_ref(), Expr::Lit(Lit::Str(_))),
            "the prefix must stay a plain literal, so a dropped segment yields the static id"
        );
        let Expr::Cond(cond) = *bin.right else {
            panic!("an injected index must be emitted as a guarded conditional");
        };

        // test: __testidIdx0 === void 0
        let Expr::Bin(test) = *cond.test else {
            panic!("expected a comparison");
        };
        assert_eq!(test.op, BinaryOp::EqEqEq, "a loose or truthiness test would drop index 0");
        let Expr::Ident(tested) = *test.left else {
            panic!("expected the index on the left of the comparison");
        };
        assert_eq!(tested.ctxt, index_ident.ctxt, "the guard must reference the binding");
        assert!(
            matches!(test.right.as_ref(), Expr::Unary(u) if u.op == UnaryOp::Void),
            "compare against `void 0`, which cannot be shadowed, not the `undefined` global"
        );

        // consequent: "" — the id degrades to exactly the static id
        assert!(
            matches!(cond.cons.as_ref(), Expr::Lit(Lit::Str(s)) if s.value.is_empty()),
            "a missing index must contribute nothing at all"
        );

        // alternative: "-" + __testidIdx0
        let Expr::Bin(alt) = *cond.alt else {
            panic!("expected a concatenation in the alternative");
        };
        let Expr::Ident(used) = *alt.right else {
            panic!("expected the index identifier");
        };
        assert_eq!(used.sym, index_ident.sym);
        assert_eq!(
            used.ctxt, index_ident.ctxt,
            "generated reference must carry the parameter's SyntaxContext"
        );
    }

    // --- inject_attributes with an active map index ---

    #[test]
    fn inject_attributes_uses_dynamic_attr_when_in_map_context() {
        let mut t = ReactDataTestIdTransform::new(PluginOptions::default());
        t.current_component = Some("List".to_string());
        t.map_indices.push(MapIndex {
            source: IndexSource::Declared(
                IdentName { span: DUMMY_SP, sym: "index".into() }.into(),
            ),
            used: false,
        });
        let mut opening = JSXOpeningElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(IdentName { span: DUMMY_SP, sym: "li".into() }.into()),
            attrs: vec![],
            self_closing: false,
            type_args: None,
        };
        t.inject_attributes(&mut opening);
        assert_eq!(opening.attrs.len(), 1);
        if let JSXAttrOrSpread::JSXAttr(attr) = &opening.attrs[0] {
            assert!(
                matches!(&attr.value, Some(JSXAttrValue::JSXExprContainer(_))),
                "expected JSXExprContainer for dynamic testid"
            );
        } else {
            panic!("expected JSXAttr");
        }
    }

    #[test]
    fn inject_attributes_uses_static_attr_when_not_in_map_context() {
        let mut t = ReactDataTestIdTransform::new(PluginOptions::default());
        t.current_component = Some("List".to_string());
        let mut opening = JSXOpeningElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(IdentName { span: DUMMY_SP, sym: "li".into() }.into()),
            attrs: vec![],
            self_closing: false,
            type_args: None,
        };
        t.inject_attributes(&mut opening);
        assert_eq!(opening.attrs.len(), 1);
        if let JSXAttrOrSpread::JSXAttr(attr) = &opening.attrs[0] {
            assert!(
                matches!(&attr.value, Some(JSXAttrValue::Str(_))),
                "expected static Str for normal testid"
            );
        } else {
            panic!("expected JSXAttr");
        }
    }
}
