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
// Base name for the per-call-site counter that supplies an index when the callee turns
// out not to pass one (see counter_call_site).
const SEQ_PARAM_BASE: &str = "__testidSeq";
// Base name for the binding that receives an instance prefix from the parent's
// `data-testid` prop (instanceIds).
const PREFIX_PARAM_BASE: &str = "__testidPrefix";
// The prop a parent passes down to prefix a child instance's ids is the first configured
// attribute — the one the plugin already injects on the child element, so nothing extra
// has to be wired up. See prefix_prop().

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
    // Index a function that is only *referenced* as a map callback (`items.map(row)`)
    // by looking up its declaration in the same file. Defaults to true.
    pub resolve_callback_refs: Option<bool>,
    // Supply an index from a call-site counter when the callee passes none —
    // `R.map(fn, items)`. Defaults to true. See counter_call_site.
    pub index_counter: Option<bool>,
    // Prefix a component's ids with the `data-testid` it received from its parent, so
    // two instances of the same component do not repeat ids. Defaults to FALSE: it
    // changes every nested id from `Badge.span` to `List.Badge-0.span`, which is a
    // breaking change for existing selectors.
    pub instance_ids: Option<bool>,
}

// How this level's index is reached from inside the callback.
enum IndexSource {
    // A parameter the source declares: `(item, i) => …`. Always bound, so the id needs
    // no guard, and the compact `{"List.li-" + i}` form is used.
    Declared(Ident),
    // An identifier bound by a destructuring pattern in the index slot:
    // `(item, [i]) => …` or `(item, { index }) => …`. The slot is occupied so nothing
    // can be appended, but the pattern does bind something to reference. Whether that
    // binding holds an index is a guess, so the segment is guarded.
    Destructured(Ident),
    // An element of a rest parameter: `(...args) => …` reads `args[1]`. No signature
    // change is needed, but the element may be absent, so the segment is guarded.
    RestElement { rest: Ident, position: f64 },
    // A parameter the plugin appends, written back only if a testid referenced it.
    Appended(Ident),
    // An appended parameter whose *default* reads a call-site counter:
    // `(item, __testidIdx0 = __testidSeq0++) => …`. The default runs only when the
    // callee passed nothing, so the binding always holds a number and the segment
    // needs no guard. See counter_call_site for when this is available.
    Counted(Ident),
}

// One enclosing map-like callback's index, and how the id refers to it.
struct MapIndex {
    source: IndexSource,
    used: bool,
    // Forces a guard on a source that would otherwise be trusted. A function that is
    // merely *referenced* as a callback (`items.map(row)`) can also be called from
    // anywhere else with no index at all, so even a parameter it declares is a guess.
    force_guard: bool,
}

impl MapIndex {
    // Expression that evaluates to this level's index inside the callback body.
    fn expr(&self) -> Expr {
        match &self.source {
            // Clone the binding's Ident verbatim so sym *and* SyntaxContext match the
            // parameter. A fresh IdentName would carry an empty ctxt and not resolve.
            IndexSource::Declared(id)
            | IndexSource::Destructured(id)
            | IndexSource::Appended(id)
            | IndexSource::Counted(id) => Expr::Ident(id.clone()),
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

    // Whether the emitted segment must survive the index being absent at runtime. Two
    // sources are exempt: a parameter the source itself declares directly on the
    // callback literal, and one whose default supplies a counter — that one is bound by
    // construction, whatever the callee does.
    fn needs_guard(&self) -> bool {
        if self.force_guard {
            return true;
        }
        !matches!(
            self.source,
            IndexSource::Declared(_) | IndexSource::Counted(_)
        )
    }
}

// Which argument of a map-like call holds the callback.
struct MapCallSite {
    arg_index: usize,
}

// A mutable view of a parameter list, so the same rules apply whether the function is a
// callback literal, a declaration, or a class method.
enum ParamsMut<'a> {
    Arrow(&'a mut ArrowExpr),
    Fn(&'a mut Function),
}

impl ParamsMut<'_> {
    fn first_pat_mut(&mut self) -> Option<&mut Pat> {
        match self {
            ParamsMut::Arrow(arrow) => arrow.params.first_mut(),
            ParamsMut::Fn(function) => function.params.first_mut().map(|p| &mut p.pat),
        }
    }

    fn push(&mut self, pat: Pat) {
        match self {
            ParamsMut::Arrow(arrow) => arrow.params.push(pat),
            ParamsMut::Fn(function) => function.params.push(Param {
                span: DUMMY_SP,
                decorators: Vec::new(),
                pat,
            }),
        }
    }
}

// Parameters the plugin owes a function once its body has been visited. Both are added
// only when an id actually referenced them, so a component or callback whose JSX needed
// nothing keeps the signature its author wrote.
#[derive(Default)]
struct Writeback {
    // A props parameter for a component that declared none: `({ "data-testid": p }) => …`
    prefix_param: Option<Pat>,
    // A stand-in for the item parameter, so an index lands in the second slot even when
    // the function declares no parameters at all.
    placeholder_param: Option<Pat>,
    // An index parameter for a function referenced as a map callback.
    index_param: Option<Pat>,
}

impl Writeback {
    fn is_empty(&self) -> bool {
        self.prefix_param.is_none() && self.index_param.is_none()
    }

    // The prefix parameter goes first (it is the props argument); the index goes last.
    fn apply(self, params: &mut ParamsMut) {
        for pat in [self.prefix_param, self.placeholder_param, self.index_param]
            .into_iter()
            .flatten()
        {
            params.push(pat);
        }
    }
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

// Finds `await` / `yield` that belong to the *enclosing* function, so a call containing
// one is never wrapped in the counter arrow — `(c => await x)(0)` does not parse, and an
// arrow cannot yield. An await inside a nested function is that function's business, so
// traversal stops at every function boundary.
#[derive(Default)]
struct SuspendScan {
    found: bool,
}

impl Visit for SuspendScan {
    fn visit_await_expr(&mut self, _: &AwaitExpr) {
        self.found = true;
    }
    fn visit_yield_expr(&mut self, _: &YieldExpr) {
        self.found = true;
    }
    fn visit_arrow_expr(&mut self, _: &ArrowExpr) {}
    fn visit_function(&mut self, _: &Function) {}
}

// Collects the names of identifiers handed to a map-like call *by reference*
// (`items.map(renderItem)`), so the function they name can be indexed where it is
// declared. Matching is by name alone: the plugin runs before the resolver in some
// hosts, so SyntaxContext cannot be relied on to tell two same-named bindings apart.
// A false match costs nothing — the segment it produces is guarded.
struct CallbackRefScan<'a> {
    map_methods: &'a [String],
    collection_first: bool,
    names: HashSet<String>,
}

impl Visit for CallbackRefScan<'_> {
    fn visit_call_expr(&mut self, n: &CallExpr) {
        if let Callee::Expr(callee) = &n.callee {
            let named = |sym: &str| self.map_methods.iter().any(|m| m == sym);
            let is_map_name = match callee.as_ref() {
                Expr::Member(m) => matches!(&m.prop, MemberProp::Ident(i) if named(&i.sym)),
                Expr::Ident(i) => named(&i.sym),
                _ => false,
            };
            if is_map_name {
                // Same argument positions as map_call_site: `items.map(fn)` and, when
                // the collection-first shape is enabled, `_.map(items, fn)`.
                let mut positions = vec![0];
                if self.collection_first && n.args.len() >= 2 {
                    positions.push(1);
                }
                for position in positions {
                    if let Some(arg) = n.args.get(position) {
                        if arg.spread.is_none() {
                            if let Expr::Ident(id) = arg.expr.as_ref() {
                                self.names.insert(id.sym.to_string());
                            }
                        }
                    }
                }
            }
        }
        n.visit_children_with(self);
    }
}

// Where a component's instance prefix comes from, and what has to be written back into
// its parameter list if an id ends up using it.
enum PrefixBinding {
    // A props parameter that already binds an identifier: `(props) => …`, read as
    // `props["data-testid"]`. Nothing to write back.
    Props(Ident),
    // A binding in the props object pattern — either one the component already
    // destructured, or one the plugin adds (ComponentScope::prefix_insert) if an id
    // ends up referencing it.
    Destructured(Ident),
    // The component declares no parameter at all, so one is appended:
    // `({ "data-testid": __testidPrefix0 }) => …`.
    NewParam(Ident),
    // A class component reads this.props.
    ThisProps,
}

// The prefix in force for the component currently being visited (instanceIds only).
struct InstancePrefix {
    binding: PrefixBinding,
    used: bool,
}

impl InstancePrefix {
    // Expression that evaluates to the prefix the parent passed, or a falsy value when
    // it passed none. A member read is guarded with `&&` because a component function
    // called directly — a helper, a test — may get no props object at all.
    fn expr(&self, prop: &str) -> Expr {
        fn read(obj: Expr, prop: &str) -> Expr {
            let member = Expr::Member(MemberExpr {
                span: DUMMY_SP,
                obj: Box::new(obj.clone()),
                prop: MemberProp::Computed(ComputedPropName {
                    span: DUMMY_SP,
                    expr: Box::new(Expr::Lit(Lit::Str(Str {
                        span: DUMMY_SP,
                        value: prop.into(),
                        raw: None,
                    }))),
                }),
            });
            Expr::Bin(BinExpr {
                span: DUMMY_SP,
                op: BinaryOp::LogicalAnd,
                left: Box::new(obj),
                right: Box::new(member),
            })
        }
        match &self.binding {
            PrefixBinding::Props(id) => read(Expr::Ident(id.clone()), prop),
            PrefixBinding::Destructured(ident) | PrefixBinding::NewParam(ident) => {
                Expr::Ident(ident.clone())
            }
            PrefixBinding::ThisProps => read(
                Expr::Member(MemberExpr {
                    span: DUMMY_SP,
                    obj: Box::new(Expr::This(ThisExpr { span: DUMMY_SP })),
                    prop: MemberProp::Ident(IdentName {
                        span: DUMMY_SP,
                        sym: "props".into(),
                    }),
                }),
                prop,
            ),
        }
    }

    // `this` inside a function expression is not the class instance, so a class
    // component's prefix cannot be read from there.
    fn reads_this(&self) -> bool {
        matches!(self.binding, PrefixBinding::ThisProps)
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
    // Names handed to a map-like call by reference (`items.map(row)`), collected in one
    // pass over the program before anything is rewritten — the declaration may appear
    // after the call site, or be hoisted above it.
    referenced_callbacks: HashSet<String>,
    // The instance prefix of the component being visited (instanceIds only).
    instance_prefix: Option<InstancePrefix>,
    // Depth of function expressions entered inside the current component. A function
    // expression rebinds `this`, so a class component's prefix is unreadable inside one.
    fn_expr_depth: u32,
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
            referenced_callbacks: HashSet::new(),
            instance_prefix: None,
            fn_expr_depth: 0,
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
    //
    // `prefix_expr` is the instance prefix (instanceIds): a leading expression that
    // already evaluates to a string in both of its branches, so it can head the
    // concatenation without breaking the "+ is always string concat" invariant.
    fn make_dynamic_attr(
        attr_name: &str,
        prefix_expr: Option<Expr>,
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

        // The leading string makes every following `+` a string concatenation, so
        // numeric indices never sum.
        let mut parts = match prefix_expr {
            Some(expr) => vec![Part::Node(Box::new(expr)), Part::Text(prefix.to_string())],
            None => vec![Part::Text(prefix.to_string())],
        };
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
        // The first part is always the prefix — a literal, or the instance-prefix
        // expression, which is a string in both branches.
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

    // Whether this call site can be given a counter to number its items with, for the
    // case where the callee turns out to pass no index at all — `R.map(fn, items)`.
    //
    // Two conditions, and both are about *soundness*, not about guessing the library:
    //
    // 1. The callback is argument 0 and the call has further arguments. That is the
    //    ramda/lodash-fp order — and, importantly, it is NOT the curried shape
    //    `map(fn)`, where the call evaluates once and the function it returns may be
    //    stored and reused. A counter created there would keep counting across every
    //    later use and drift away from the item positions. With a collection argument
    //    present, the call iterates now, and the counter lives exactly as long as the
    //    iteration. (The collection-first shape `_.map(items, fn)` needs no counter:
    //    every callee that takes it passes an index or a key.)
    //
    // 2. Nothing in the call awaits or yields, since the counter binding is introduced
    //    by an arrow wrapped around the call, and an arrow can do neither.
    //
    // The counter is what makes the segment *unguarded*: whatever the callee does, the
    // parameter's default leaves it bound to a number.
    fn counter_call_site(&self, n: &CallExpr, site: &MapCallSite) -> bool {
        if !self.index_counter_enabled() || !self.auto_index_enabled() || site.arg_index != 0 {
            return false;
        }
        if n.args.len() < 2 {
            return false;
        }
        let mut scan = SuspendScan::default();
        n.visit_with(&mut scan);
        !scan.found
    }

    // `((counter) => call)(0)`
    fn wrap_in_counter(call: CallExpr, counter: Ident) -> CallExpr {
        let arrow = Expr::Arrow(ArrowExpr {
            span: DUMMY_SP,
            ctxt: SyntaxContext::default(),
            params: vec![Self::binding_pat(counter)],
            body: Box::new(BlockStmtOrExpr::Expr(Box::new(Expr::Call(call)))),
            is_async: false,
            is_generator: false,
            type_params: None,
            return_type: None,
        });
        CallExpr {
            span: DUMMY_SP,
            ctxt: SyntaxContext::default(),
            // Parenthesised explicitly: an arrow in callee position needs it, and the
            // plugin's output is not passed through SWC's fixer pass.
            callee: Callee::Expr(Box::new(Expr::Paren(ParenExpr {
                span: DUMMY_SP,
                expr: Box::new(arrow),
            }))),
            args: vec![ExprOrSpread {
                spread: None,
                expr: Box::new(Expr::Lit(Lit::Num(Number {
                    span: DUMMY_SP,
                    value: 0.0,
                    raw: None,
                }))),
            }],
            type_args: None,
        }
    }

    // A placeholder to swap a call out of the tree with; never reaches the output.
    fn empty_call() -> CallExpr {
        CallExpr {
            span: DUMMY_SP,
            ctxt: SyntaxContext::default(),
            callee: Callee::Expr(Box::new(Expr::Invalid(Invalid { span: DUMMY_SP }))),
            args: Vec::new(),
            type_args: None,
        }
    }

    // Parameter patterns of a function literal, in source order.
    fn fn_params(callback: &Expr) -> Option<Vec<&Pat>> {
        match callback {
            Expr::Arrow(arrow) => Some(arrow.params.iter().collect()),
            Expr::Fn(fn_expr) => Some(fn_expr.function.params.iter().map(|p| &p.pat).collect()),
            _ => None,
        }
    }

    // Same, for a function that is not an expression — a declaration or a method. Every
    // parameter rule below applies equally to a function only *referenced* as a callback.
    fn params_of_fn(function: &Function) -> Vec<&Pat> {
        function.params.iter().map(|p| &p.pat).collect()
    }

    // Unwraps whatever single identifier a parameter binds directly. A default value is
    // transparent: `(item, index = 0) => …` still binds `index`, and the default only
    // applies when the callee passes nothing — exactly the case where the guarded
    // segment emitted by make_dynamic_attr would have dropped out anyway.
    fn param_ident(pat: &Pat) -> Option<Ident> {
        match pat {
            Pat::Ident(binding) => Some(binding.id.clone()),
            Pat::Assign(assign) => Self::param_ident(&assign.left),
            _ => None,
        }
    }

    // The index parameter the source already declares, if it binds a plain identifier.
    fn declared_index_ident_from(params: &[&Pat]) -> Option<Ident> {
        Self::param_ident(params.get(1)?)
    }

    // The index slot holds a destructuring pattern: `(item, [i]) => …`. Nothing can be
    // appended — the slot is taken — but the pattern does bind identifiers, and the
    // first one is what the source pulled out of that argument. Referencing it is a
    // guess about the callee, so the segment is guarded and a wrong guess degrades to
    // the static id. A pattern that binds nothing at all (`(item, []) => …`) yields None.
    fn destructured_index_ident_from(params: &[&Pat]) -> Option<Ident> {
        // Only binding positions are considered. A default *value* is an arbitrary
        // expression that may itself contain functions, and identifiers in there are
        // not bound by this parameter.
        fn first_binding(pat: &Pat) -> Option<Ident> {
            match pat {
                Pat::Ident(binding) => Some(binding.id.clone()),
                Pat::Assign(assign) => first_binding(&assign.left),
                Pat::Rest(rest) => first_binding(&rest.arg),
                Pat::Array(array) => array.elems.iter().flatten().find_map(first_binding),
                Pat::Object(object) => object.props.iter().find_map(|prop| match prop {
                    ObjectPatProp::KeyValue(kv) => first_binding(&kv.value),
                    ObjectPatProp::Assign(assign) => Some(assign.key.id.clone()),
                    ObjectPatProp::Rest(rest) => first_binding(&rest.arg),
                }),
                _ => None,
            }
        }
        let pat = params.get(1)?;
        // A plain identifier is the Declared case and must not be re-routed here.
        if Self::param_ident(pat).is_some() {
            return None;
        }
        first_binding(pat)
    }

    // A rest parameter collects the index instead of naming it: `(...args) => …` reads it
    // as `args[1]`, and `(item, ...rest) => …` as `rest[0]`. Nothing may be appended after
    // a rest element, but nothing needs to be — the index is already in scope.
    fn rest_index_source_from(params: &[&Pat]) -> Option<IndexSource> {
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
    fn can_take_index_param_from(params: &[&Pat]) -> bool {
        params.len() < 2 && !params.iter().any(|p| matches!(p, Pat::Rest(_)))
    }

    // SyntaxContext of the callback's own parameter scope, so an appended binding is
    // hygienically indistinguishable from one written in the source. Falls back to the
    // empty context when the callback declares no identifier parameters to copy from;
    // binding and reference still agree, which is what hygiene and minifiers require.
    fn params_ctxt(callback: &Expr) -> SyntaxContext {
        match Self::fn_params(callback) {
            Some(params) => Self::params_ctxt_from(&params),
            None => SyntaxContext::default(),
        }
    }

    fn params_ctxt_from(params: &[&Pat]) -> SyntaxContext {
        let mut scan = IdentScan::default();
        for param in params {
            param.visit_with(&mut scan);
        }
        scan.first_ctxt.unwrap_or_default()
    }

    // Generates an identifier that appears nowhere inside `callback`, so appending it
    // cannot shadow a binding the body relies on. The counter additionally keeps names
    // distinct across nested callbacks, where scanning an inner callback cannot yet see
    // an outer generated name — its references are only added later in the traversal.
    fn fresh_ident(&mut self, base: &str, callback: &Expr) -> Ident {
        self.fresh_ident_in(base, callback, Self::params_ctxt(callback))
    }

    // Same, over an arbitrary node: the binding a counter introduces lives *outside* the
    // callback, so its name has to avoid every identifier in the whole call expression.
    fn fresh_ident_in<N>(&mut self, base: &str, node: &N, ctxt: SyntaxContext) -> Ident
    where
        N: VisitWith<IdentScan>,
    {
        let mut scan = IdentScan::default();
        node.visit_with(&mut scan);
        loop {
            let name = format!("{}{}", base, self.next_generated_id);
            self.next_generated_id += 1;
            if !scan.syms.contains(name.as_str()) {
                return Ident::new(name.into(), DUMMY_SP, ctxt);
            }
        }
    }

    fn append_index_param(&mut self, callback: &mut Expr, index: Ident) {
        self.append_index_pat(callback, Self::binding_pat(index));
    }

    // The counter form: `(item, __testidIdx0 = __testidSeq0++) => …`. A default is
    // evaluated per call and only when the argument is `undefined`, which makes it an
    // exact fit for "use the callee's index if there is one, otherwise number the items
    // myself". It also keeps the callback's `Function.length` unchanged, where a plain
    // appended parameter would raise it.
    fn append_counted_index_param(&mut self, callback: &mut Expr, index: Ident, counter: Ident) {
        let pat = Pat::Assign(AssignPat {
            span: DUMMY_SP,
            left: Box::new(Self::binding_pat(index)),
            right: Box::new(Expr::Update(UpdateExpr {
                span: DUMMY_SP,
                op: UpdateOp::PlusPlus,
                prefix: false,
                arg: Box::new(Expr::Ident(counter)),
            })),
        });
        self.append_index_pat(callback, pat);
    }

    // Appends the index parameter, adding a placeholder first parameter when the
    // callback declares none: `() => …` must become `(__testidItem2, __testidIdx1) => …`.
    // The placeholder carries the *higher* counter value because the index was allocated
    // before the callback was visited, while this placeholder is allocated now, after.
    // Do not "fix" those digits to read 0, 1 — see tests/fixture/map-no-index-param.
    fn append_index_pat(&mut self, callback: &mut Expr, index: Pat) {
        if Self::fn_params(callback).is_some_and(|p| p.is_empty()) {
            let placeholder = self.fresh_ident(ITEM_PARAM_BASE, callback);
            Self::push_param(callback, placeholder);
        }
        Self::push_pat_param(callback, index);
    }

    fn binding_pat(ident: Ident) -> Pat {
        Pat::Ident(BindingIdent {
            id: ident,
            type_ann: None,
        })
    }

    fn push_param(callback: &mut Expr, ident: Ident) {
        Self::push_pat_param(callback, Self::binding_pat(ident));
    }

    fn push_pat_param(callback: &mut Expr, pat: Pat) {
        match callback {
            Expr::Arrow(arrow) => arrow.params.push(pat),
            Expr::Fn(fn_expr) => Self::push_fn_param(&mut fn_expr.function, pat),
            _ => {}
        }
    }

    fn push_fn_param(function: &mut Function, pat: Pat) {
        function.params.push(Param {
            span: DUMMY_SP,
            decorators: Vec::new(),
            pat,
        });
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

    fn resolve_callback_refs_enabled(&self) -> bool {
        self.options.resolve_callback_refs.unwrap_or(true)
    }

    fn index_counter_enabled(&self) -> bool {
        self.options.index_counter.unwrap_or(true)
    }

    // The one option that is off by default: it rewrites `Badge.span` to
    // `List.Badge-0.span`, so every existing selector for a nested component changes.
    fn instance_ids_enabled(&self) -> bool {
        self.options.instance_ids.unwrap_or(false)
    }

    // The prop a parent uses to hand an instance prefix to a child. The plugin already
    // injects the configured attributes on the child element, so the first of them is
    // the prefix that arrived — no separate prop, and no wiring for the user.
    fn prefix_prop(&self) -> String {
        self.attributes()
            .first()
            .cloned()
            .unwrap_or_else(|| DEFAULT_ATTRIBUTE.to_string())
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

        // With instanceIds, the component part of the id is whatever prefix the parent
        // passed down, falling back to the component's own name when it rendered at the
        // top level (or was rendered by something the plugin did not touch). `||` is the
        // right test here where an index needs `=== void 0`: a prefix is a string, and
        // the only falsy string — "" — is exactly the case that must fall back too.
        let prefix_expr = self.instance_prefix.as_ref().and_then(|prefix| {
            // `this` is not the class instance inside a function expression.
            if prefix.reads_this() && self.fn_expr_depth > 0 {
                return None;
            }
            Some(Expr::Bin(BinExpr {
                span: DUMMY_SP,
                op: BinaryOp::LogicalOr,
                left: Box::new(prefix.expr(&self.prefix_prop())),
                right: Box::new(Expr::Lit(Lit::Str(Str {
                    span: DUMMY_SP,
                    value: component.as_str().into(),
                    raw: None,
                }))),
            }))
        });
        // Only mark the prefix used once we know an attribute will actually reference it.
        let uses_prefix = prefix_expr.is_some()
            && attributes
                .iter()
                .any(|attr_name| !Self::has_attribute(&opening.attrs, attr_name));
        if uses_prefix {
            if let Some(prefix) = self.instance_prefix.as_mut() {
                prefix.used = true;
            }
        }
        // The static part that follows the prefix: ".span" when a prefix expression
        // supplies the component name, "Comp.span" when it does not.
        let static_part = match prefix_expr {
            Some(_) => format!(".{}", unique_id),
            None => test_id.clone(),
        };

        for attr_name in attributes {
            if !Self::has_attribute(&opening.attrs, &attr_name) {
                // With nestedIndexes off, only the innermost callback contributes — the
                // pre-0.2.0 id, which repeats across outer iterations.
                let levels = if self.nested_indexes_enabled() {
                    0
                } else {
                    self.map_indices.len().saturating_sub(1)
                };
                let attr = if self.map_indices.is_empty() && prefix_expr.is_none() {
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
                    Self::make_dynamic_attr(
                        &attr_name,
                        prefix_expr.clone(),
                        &static_part,
                        &segments,
                    )
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

    // Everything a component entry point does around its body: activate the component,
    // work out where an instance prefix would come from, and — for a function that is
    // handed to a map-like call by reference — put an index in scope so its JSX is
    // unique per item.
    //
    // Every generated identifier is allocated here, while the whole node is still
    // readable, and only *written back* at exit if an id referenced it. That is the same
    // rule the inline-callback path follows: a function whose JSX needed nothing keeps
    // the signature its author wrote.
    fn enter_component_scope<N>(&mut self, name: String, params: &[&Pat], node: &N) -> ComponentScope
    where
        N: VisitWith<IdentScan>,
    {
        let is_callback =
            self.resolve_callback_refs_enabled() && self.referenced_callbacks.contains(&name);
        let prev_component = self.enter_component(name);
        let prev_fn_expr_depth = std::mem::replace(&mut self.fn_expr_depth, 0);

        // A function reached through `items.map(row)` receives the item as its first
        // argument, not props — so it takes an index, and never an instance prefix.
        let mut scope = ComponentScope {
            prev_component,
            prev_prefix: None,
            prev_fn_expr_depth,
            pushed_index: false,
            prefix_insert: None,
            prefix_param: None,
            placeholder_param: None,
            index_param: None,
        };

        let prefix = if is_callback {
            None
        } else {
            self.plan_instance_prefix(params, node, &mut scope)
        };
        scope.prev_prefix = std::mem::replace(&mut self.instance_prefix, prefix);

        if is_callback {
            if let Some(source) = self.plan_callback_index(params, node, &mut scope) {
                // Even a parameter this function declares is a guess: unlike a callback
                // literal, a named function can also be called from somewhere that
                // passes no index at all.
                self.map_indices.push(MapIndex {
                    source,
                    used: false,
                    force_guard: true,
                });
                scope.pushed_index = true;
            }
        }
        scope
    }

    // A class component's prefix comes from this.props, so there are no parameters to
    // plan for, and none of the callback-reference handling applies to render().
    fn enter_class_component_scope(&mut self, name: String) -> ComponentScope {
        let prev_component = self.enter_component(name);
        let prev_fn_expr_depth = std::mem::replace(&mut self.fn_expr_depth, 0);
        let prefix = self.instance_ids_enabled().then(|| InstancePrefix {
            binding: PrefixBinding::ThisProps,
            used: false,
        });
        ComponentScope {
            prev_component,
            prev_prefix: std::mem::replace(&mut self.instance_prefix, prefix),
            prev_fn_expr_depth,
            pushed_index: false,
            prefix_insert: None,
            prefix_param: None,
            placeholder_param: None,
            index_param: None,
        }
    }

    fn params_mut_of(expr: &mut Expr) -> Option<ParamsMut<'_>> {
        match expr {
            Expr::Arrow(arrow) => Some(ParamsMut::Arrow(arrow)),
            Expr::Fn(fn_expr) => Some(ParamsMut::Fn(&mut fn_expr.function)),
            _ => None,
        }
    }

    // Restores the enclosing component and writes back whatever the body actually used.
    fn exit_component_scope(&mut self, scope: ComponentScope, params: &mut ParamsMut) {
        let mut writeback = Writeback::default();

        if scope.pushed_index {
            let entry = self
                .map_indices
                .pop()
                .expect("component map index stack popped by a nested visit");
            if entry.used {
                writeback.index_param = scope.index_param;
                writeback.placeholder_param = scope.placeholder_param;
            }
        }

        let prefix = std::mem::replace(&mut self.instance_prefix, scope.prev_prefix);
        if let Some(prefix) = prefix {
            if prefix.used {
                writeback.prefix_param = scope.prefix_param;
                if let Some((prop, ident)) = scope.prefix_insert {
                    Self::insert_prefix_prop(params, &prop, ident);
                }
            }
        }

        if !writeback.is_empty() {
            writeback.apply(params);
        }
        self.fn_expr_depth = scope.prev_fn_expr_depth;
        self.current_component = scope.prev_component;
    }

    // Decides where this component reads the prefix its parent passed down, allocating
    // (but not yet inserting) any binding that has to be introduced.
    fn plan_instance_prefix<N>(
        &mut self,
        params: &[&Pat],
        node: &N,
        scope: &mut ComponentScope,
    ) -> Option<InstancePrefix>
    where
        N: VisitWith<IdentScan>,
    {
        if !self.instance_ids_enabled() {
            return None;
        }
        let prop = self.prefix_prop();
        let ctxt = Self::params_ctxt_from(params);
        let binding = match params.first() {
            // No props parameter at all — one is appended if an id ends up using it.
            None => {
                let ident = self.fresh_ident_in(PREFIX_PARAM_BASE, node, ctxt);
                scope.prefix_param = Some(Self::props_param(&prop, ident.clone()));
                PrefixBinding::NewParam(ident)
            }
            Some(pat) => {
                // `({ items } = {}) => …` — a default on the props parameter is
                // transparent, the pattern underneath is what binds.
                let pat = match pat {
                    Pat::Assign(assign) => assign.left.as_ref(),
                    other => other,
                };
                match pat {
                    Pat::Ident(binding) => PrefixBinding::Props(binding.id.clone()),
                    Pat::Object(object) => match Self::destructured_prop_ident(object, &prop) {
                        // The component already destructures the prop itself; use that.
                        Some(ident) => PrefixBinding::Destructured(ident),
                        None => {
                            let ident = self.fresh_ident_in(PREFIX_PARAM_BASE, node, ctxt);
                            scope.prefix_insert = Some((prop, ident.clone()));
                            PrefixBinding::Destructured(ident)
                        }
                    },
                    // An array pattern in the props slot is not a React component.
                    _ => return None,
                }
            }
        };
        Some(InstancePrefix {
            binding,
            used: false,
        })
    }

    // The identifier an object pattern already binds for `prop`, if any — so a component
    // that destructures `{"data-testid": testId}` itself is read, not rewritten.
    fn destructured_prop_ident(object: &ObjectPat, prop: &str) -> Option<Ident> {
        object.props.iter().find_map(|p| match p {
            ObjectPatProp::KeyValue(kv) => {
                let matches = match &kv.key {
                    PropName::Str(s) => s.value.as_str() == Some(prop),
                    PropName::Ident(i) => i.sym.as_ref() == prop,
                    _ => false,
                };
                matches.then(|| Self::param_ident(&kv.value)).flatten()
            }
            ObjectPatProp::Assign(assign) => (assign.key.id.sym.as_ref() == prop).then(|| assign.key.id.clone()),
            ObjectPatProp::Rest(_) => None,
        })
    }

    // `{ "data-testid": __testidPrefix0 } = {}` — as a whole parameter. The default is
    // not decoration: destructuring `undefined` throws, and a component that declared no
    // parameter at all can legitimately be called with no argument (a helper, a test).
    // It also leaves Function.length at 0, where a plain parameter would raise it.
    fn props_param(prop: &str, ident: Ident) -> Pat {
        Pat::Assign(AssignPat {
            span: DUMMY_SP,
            left: Box::new(Pat::Object(ObjectPat {
                span: DUMMY_SP,
                props: vec![Self::prefix_prop_pat(prop, ident)],
                optional: false,
                type_ann: None,
            })),
            right: Box::new(Expr::Object(ObjectLit {
                span: DUMMY_SP,
                props: Vec::new(),
            })),
        })
    }

    // …or as one property added to the pattern the component already had. It goes first
    // because a rest element must stay last.
    fn insert_prefix_prop(params: &mut ParamsMut, prop: &str, ident: Ident) {
        let Some(pat) = params.first_pat_mut() else {
            return;
        };
        let pat = match pat {
            Pat::Assign(assign) => assign.left.as_mut(),
            other => other,
        };
        if let Pat::Object(object) = pat {
            object.props.insert(0, Self::prefix_prop_pat(prop, ident));
        }
    }

    fn prefix_prop_pat(prop: &str, ident: Ident) -> ObjectPatProp {
        ObjectPatProp::KeyValue(KeyValuePatProp {
            key: PropName::Str(Str {
                span: DUMMY_SP,
                value: prop.into(),
                raw: None,
            }),
            value: Box::new(Self::binding_pat(ident)),
        })
    }

    // The index a function referenced as a map callback will use, in the same order of
    // confidence as an inline callback. No call-site counter is available here — the
    // call sites are elsewhere, possibly several — so every source is guarded.
    fn plan_callback_index<N>(
        &mut self,
        params: &[&Pat],
        node: &N,
        scope: &mut ComponentScope,
    ) -> Option<IndexSource>
    where
        N: VisitWith<IdentScan>,
    {
        if let Some(ident) = Self::declared_index_ident_from(params) {
            return Some(IndexSource::Declared(ident));
        }
        if let Some(source) = Self::rest_index_source_from(params) {
            return Some(source);
        }
        if let Some(ident) = Self::destructured_index_ident_from(params) {
            return Some(IndexSource::Destructured(ident));
        }
        if !self.auto_index_enabled() || !Self::can_take_index_param_from(params) {
            return None;
        }
        let ctxt = Self::params_ctxt_from(params);
        let index = self.fresh_ident_in(INDEX_PARAM_BASE, node, ctxt);
        if params.is_empty() {
            let placeholder = self.fresh_ident_in(ITEM_PARAM_BASE, node, ctxt);
            scope.placeholder_param = Some(Self::binding_pat(placeholder));
        }
        scope.index_param = Some(Self::binding_pat(index.clone()));
        Some(IndexSource::Appended(index))
    }
}

// State a component entry point saves while its body is visited, plus every identifier
// the plugin may have to write back into its parameter list afterwards.
struct ComponentScope {
    prev_component: Option<String>,
    prev_prefix: Option<InstancePrefix>,
    prev_fn_expr_depth: u32,
    pushed_index: bool,
    // (prop name, binding) to add to an existing props pattern.
    prefix_insert: Option<(String, Ident)>,
    prefix_param: Option<Pat>,
    placeholder_param: Option<Pat>,
    index_param: Option<Pat>,
}

impl VisitMut for ReactDataTestIdTransform {
    // One read-only pass first: a function handed to a map-like call by reference
    // (`items.map(row)`) has to be indexed where it is *declared*, and that declaration
    // may appear after the call site, or be hoisted above it.
    fn visit_mut_program(&mut self, n: &mut Program) {
        if self.resolve_callback_refs_enabled() {
            let mut scan = CallbackRefScan {
                map_methods: self.map_methods(),
                collection_first: self.collection_first_enabled(),
                names: HashSet::new(),
            };
            n.visit_with(&mut scan);
            self.referenced_callbacks = scan.names;
        }
        n.visit_mut_children_with(self);
    }

    // A function expression rebinds `this`, so a class component's `this.props` is not
    // readable inside one. Tracked as a depth, not a flag, because callbacks nest.
    fn visit_mut_fn_expr(&mut self, n: &mut FnExpr) {
        self.fn_expr_depth += 1;
        n.visit_mut_children_with(self);
        self.fn_expr_depth -= 1;
    }

    // function MyComponent() { ... }
    fn visit_mut_fn_decl(&mut self, n: &mut FnDecl) {
        let scope = {
            let params = Self::params_of_fn(&n.function);
            self.enter_component_scope(n.ident.sym.to_string(), &params, &*n)
        };
        n.visit_mut_children_with(self);
        self.exit_component_scope(scope, &mut ParamsMut::Fn(&mut n.function));
    }

    // const MyComponent = () => { ... }
    // const MyComponent = function() { ... }
    fn visit_mut_var_declarator(&mut self, n: &mut VarDeclarator) {
        if let Pat::Ident(ident) = &n.name {
            if let Some(init) = &n.init {
                if matches!(init.as_ref(), Expr::Arrow(_) | Expr::Fn(_)) {
                    let name = ident.sym.to_string();
                    let scope = {
                        let params = Self::fn_params(init).unwrap_or_default();
                        self.enter_component_scope(name, &params, &**init)
                    };
                    n.visit_mut_children_with(self);
                    if let Some(mut params) =
                        n.init.as_deref_mut().and_then(Self::params_mut_of)
                    {
                        self.exit_component_scope(scope, &mut params);
                    }
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
                    // A class reads its instance prefix from this.props, not from a
                    // parameter, so there is nothing to write back to render()'s own
                    // signature.
                    let scope = self.enter_class_component_scope(class_name);
                    n.visit_mut_children_with(self);
                    self.exit_component_scope(scope, &mut ParamsMut::Fn(&mut n.function));
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
                let scope = {
                    let params = Self::fn_params(&n.expr).unwrap_or_default();
                    self.enter_component_scope(name, &params, &*n.expr)
                };
                n.visit_mut_children_with(self);
                if let Some(mut params) = Self::params_mut_of(&mut n.expr) {
                    self.exit_component_scope(scope, &mut params);
                }
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
                let scope = {
                    let params = Self::params_of_fn(&fn_expr.function);
                    self.enter_component_scope(name, &params, &fn_expr.function)
                };
                n.visit_mut_children_with(self);
                if let DefaultDecl::Fn(fn_expr) = &mut n.decl {
                    self.exit_component_scope(scope, &mut ParamsMut::Fn(&mut fn_expr.function));
                }
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
        // then one a rest parameter collects, then one a destructuring pattern in the
        // index slot binds, then one appended as a last resort.
        let callback = &n.args[site.arg_index].expr;
        let params = Self::fn_params(callback).unwrap_or_default();
        // A counter is only worth wiring up where the callee might pass no index at all
        // and the appended parameter would otherwise idle.
        let counter = self
            .counter_call_site(n, &site)
            .then(|| self.fresh_ident_in(SEQ_PARAM_BASE, &*n, SyntaxContext::default()));
        let source = Self::declared_index_ident_from(&params)
            .map(IndexSource::Declared)
            .or_else(|| Self::rest_index_source_from(&params))
            .or_else(|| Self::destructured_index_ident_from(&params).map(IndexSource::Destructured))
            .or_else(|| {
                (self.auto_index_enabled() && Self::can_take_index_param_from(&params)).then(|| {
                    let index = self.fresh_ident(INDEX_PARAM_BASE, callback);
                    match counter {
                        Some(_) => IndexSource::Counted(index),
                        None => IndexSource::Appended(index),
                    }
                })
            });
        let mut index = source.map(|source| MapIndex {
            source,
            used: false,
            force_guard: false,
        });

        // The index binding exists ONLY inside the callback. Visiting the callee or the
        // remaining arguments with it in scope emits a reference to a variable that does
        // not exist there — e.g. [<Header/>, ...xs].map((x, i) => ...) would give <Header>
        // a testid of {"C.Header-" + i}, throwing ReferenceError at render. Children are
        // visited in their natural order (callee, args, type_args) so element numbering
        // is unchanged.
        // Set only if the counter is actually referenced, so a call site whose callback
        // never used one is left exactly as it was written.
        let mut wrap_in_counter = None;
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
                IndexSource::Appended(ident) | IndexSource::Counted(ident) => Some(ident.clone()),
                _ => None,
            };
            let counted = matches!(entry.source, IndexSource::Counted(_));
            self.map_indices.push(entry);
            arg.visit_mut_with(self);
            let entry = self
                .map_indices
                .pop()
                .expect("map index stack popped by a nested visit");
            // A callback whose JSX (if any) never used the index keeps its original
            // signature — no stray parameter in the output.
            if let (Some(ident), true) = (appended, entry.used) {
                match (counted, counter.clone()) {
                    (true, Some(seq)) => {
                        self.append_counted_index_param(&mut arg.expr, ident, seq.clone());
                        wrap_in_counter = Some(seq);
                    }
                    _ => self.append_index_param(&mut arg.expr, ident),
                }
            }
        }
        n.type_args.visit_mut_with(self);

        // The counter has to outlive each callback invocation but be reset every time
        // the map call itself runs, so it is bound by an arrow wrapped around the call:
        //   ((__testidSeq0) => R.map((item, __testidIdx0 = __testidSeq0++) => …, xs))(0)
        if let Some(seq) = wrap_in_counter {
            let inner = std::mem::replace(n, Self::empty_call());
            *n = Self::wrap_in_counter(inner, seq);
        }
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

    // Same contract for the options added in 0.3.0: a field name that does not match is
    // not a compile error, it is an option that silently does nothing.
    #[test]
    fn parse_options_deserializes_the_new_options() {
        let opts = parse_options(Some(
            r#"{"resolveCallbackRefs":false,"indexCounter":false,"instanceIds":true}"#.to_string(),
        ));
        assert_eq!(opts.resolve_callback_refs, Some(false));
        assert_eq!(opts.index_counter, Some(false));
        assert_eq!(opts.instance_ids, Some(true));

        let t = ReactDataTestIdTransform::new(opts);
        assert!(!t.resolve_callback_refs_enabled());
        assert!(!t.index_counter_enabled());
        assert!(t.instance_ids_enabled());
    }

    // instanceIds is the one option that has to be switched ON, because it rewrites ids
    // that already work: `Badge.span` becomes `List.Badge.span`. Every other option is
    // an opt-out from inference that is already correct by default.
    #[test]
    fn instance_ids_is_the_only_option_that_defaults_to_off() {
        let d = ReactDataTestIdTransform::new(PluginOptions::default());
        assert!(d.resolve_callback_refs_enabled());
        assert!(d.index_counter_enabled());
        assert!(!d.instance_ids_enabled());
    }

    // The prefix prop is the attribute the plugin already injects on the child, so a
    // project that renamed the attribute gets the prefix under the new name too.
    #[test]
    fn prefix_prop_follows_the_configured_attribute() {
        assert_eq!(
            ReactDataTestIdTransform::new(PluginOptions::default()).prefix_prop(),
            "data-testid"
        );
        assert_eq!(
            ReactDataTestIdTransform::new(PluginOptions {
                attributes: Some(vec!["data-test-id".into(), "data-cy".into()]),
                ..Default::default()
            })
            .prefix_prop(),
            "data-test-id"
        );
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

    fn params_of(callback: &Expr) -> Vec<&Pat> {
        ReactDataTestIdTransform::fn_params(callback).unwrap_or_default()
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
            ReactDataTestIdTransform::declared_index_ident_from(&params_of(&cb)).map(|i| i.sym.to_string()),
            Some("idx".to_string())
        );
    }

    #[test]
    fn declared_index_ident_returns_none_when_only_one_param() {
        let cb = arrow(vec![param("item")]);
        assert!(ReactDataTestIdTransform::declared_index_ident_from(&params_of(&cb)).is_none());
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
        assert!(ReactDataTestIdTransform::declared_index_ident_from(&params_of(&cb)).is_none());
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
            ReactDataTestIdTransform::declared_index_ident_from(&params_of(&cb)).map(|i| i.sym.to_string()),
            Some("index".to_string())
        );
    }

    #[test]
    fn can_take_index_param_accepts_zero_or_one_param() {
        assert!(ReactDataTestIdTransform::can_take_index_param_from(&params_of(&arrow(vec![]))));
        assert!(ReactDataTestIdTransform::can_take_index_param_from(&params_of(&arrow(vec![param("item")]))));
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
        assert!(!ReactDataTestIdTransform::can_take_index_param_from(&params_of(&cb)));
    }

    #[test]
    fn can_take_index_param_rejects_callback_that_already_has_two() {
        let cb = arrow(vec![param("item"), param("idx")]);
        assert!(!ReactDataTestIdTransform::can_take_index_param_from(&params_of(&cb)));
    }

    // --- destructured_index_ident_from ---

    fn array_pat(elems: Vec<Pat>) -> Pat {
        Pat::Array(ArrayPat {
            span: DUMMY_SP,
            elems: elems.into_iter().map(Some).collect(),
            optional: false,
            type_ann: None,
        })
    }

    #[test]
    fn destructured_index_reads_the_first_binding_of_an_array_pattern() {
        // `(item, [i]) => …` — the slot is occupied so nothing can be appended, but the
        // pattern does bind something the source pulled out of that argument.
        let cb = arrow(vec![param("item"), array_pat(vec![param("i")])]);
        assert_eq!(
            ReactDataTestIdTransform::destructured_index_ident_from(&params_of(&cb))
                .map(|i| i.sym.to_string()),
            Some("i".to_string())
        );
    }

    #[test]
    fn destructured_index_reads_a_shorthand_object_pattern() {
        // `(item, { index }) => …`
        let cb = arrow(vec![
            param("item"),
            Pat::Object(ObjectPat {
                span: DUMMY_SP,
                props: vec![ObjectPatProp::Assign(AssignPatProp {
                    span: DUMMY_SP,
                    key: BindingIdent {
                        id: IdentName { span: DUMMY_SP, sym: "index".into() }.into(),
                        type_ann: None,
                    },
                    value: None,
                })],
                optional: false,
                type_ann: None,
            }),
        ]);
        assert_eq!(
            ReactDataTestIdTransform::destructured_index_ident_from(&params_of(&cb))
                .map(|i| i.sym.to_string()),
            Some("index".to_string())
        );
    }

    #[test]
    fn destructured_index_ignores_a_plain_identifier_slot() {
        // That is the Declared case, which is trusted and unguarded; routing it here
        // would guard an index the source asserted was real.
        let cb = arrow(vec![param("item"), param("i")]);
        assert!(
            ReactDataTestIdTransform::destructured_index_ident_from(&params_of(&cb)).is_none()
        );
    }

    #[test]
    fn destructured_index_returns_none_when_the_pattern_binds_nothing() {
        let cb = arrow(vec![param("item"), array_pat(vec![])]);
        assert!(
            ReactDataTestIdTransform::destructured_index_ident_from(&params_of(&cb)).is_none()
        );
    }

    // Order matters at the call site: a rest element at position 1 is read as `rest[0]`,
    // and must never be referenced as if it were the index itself — `"-" + rest` would
    // stringify the whole array.
    #[test]
    fn a_rest_element_is_not_treated_as_a_destructured_index() {
        let cb = arrow(vec![
            param("item"),
            Pat::Rest(RestPat {
                span: DUMMY_SP,
                dot3_token: DUMMY_SP,
                arg: Box::new(param("rest")),
                type_ann: None,
            }),
        ]);
        let params = params_of(&cb);
        assert!(
            matches!(
                ReactDataTestIdTransform::rest_index_source_from(&params),
                Some(IndexSource::RestElement { .. })
            ),
            "the rest source must match first, or the destructured path would take it"
        );
    }

    // --- counter_call_site ---

    fn counter_for(options: PluginOptions, call: &CallExpr) -> bool {
        let t = ReactDataTestIdTransform::new(options);
        match t.map_call_site(call) {
            Some(site) => t.counter_call_site(call, &site),
            None => false,
        }
    }

    #[test]
    fn counter_only_where_a_collection_argument_follows_the_callback() {
        // R.map(fn, items) — the call iterates now, so the counter lives exactly as long
        // as the iteration does.
        let ramda = call(
            Some("R"),
            "map",
            vec![arrow(vec![param("item")]), ident_expr("items")],
        );
        assert!(counter_for(PluginOptions::default(), &ramda));

        // map(fn) — curried. This call evaluates once and the function it returns may be
        // stored and reused; a counter there would climb past the item positions.
        let curried = call(None, "map", vec![arrow(vec![param("item")])]);
        assert!(!counter_for(PluginOptions::default(), &curried));

        // _.map(items, fn) — every callee that takes this shape passes an index or a key.
        let lodash = call(
            Some("_"),
            "map",
            vec![ident_expr("items"), arrow(vec![param("item")])],
        );
        assert!(!counter_for(PluginOptions::default(), &lodash));
    }

    #[test]
    fn counter_is_suppressed_by_either_opt_out() {
        let ramda = || {
            call(
                Some("R"),
                "map",
                vec![arrow(vec![param("item")]), ident_expr("items")],
            )
        };
        assert!(!counter_for(
            PluginOptions { index_counter: Some(false), ..Default::default() },
            &ramda()
        ));
        // autoIndex governs appending, and the counter rides on an appended parameter.
        assert!(!counter_for(
            PluginOptions { auto_index: Some(false), ..Default::default() },
            &ramda()
        ));
    }

    // The counter binding is introduced by an arrow wrapped around the call, and an
    // arrow can neither await nor yield.
    #[test]
    fn counter_is_refused_when_the_call_awaits() {
        let awaited = call(
            Some("R"),
            "map",
            vec![
                arrow(vec![param("item")]),
                Box::new(Expr::Await(AwaitExpr {
                    span: DUMMY_SP,
                    arg: ident_expr("load"),
                })),
            ],
        );
        assert!(!counter_for(PluginOptions::default(), &awaited));
    }

    // …but an await inside the callback is that callback's own business: an async arrow
    // keeps working when the *call* around it is wrapped.
    #[test]
    fn counter_is_allowed_when_only_the_callback_awaits() {
        let mut cb = arrow(vec![param("item")]);
        if let Expr::Arrow(arrow) = cb.as_mut() {
            arrow.is_async = true;
            arrow.body = Box::new(BlockStmtOrExpr::Expr(Box::new(Expr::Await(AwaitExpr {
                span: DUMMY_SP,
                arg: ident_expr("item"),
            }))));
        }
        let c = call(Some("R"), "map", vec![cb, ident_expr("items")]);
        assert!(counter_for(PluginOptions::default(), &c));
    }

    // --- guarding ---

    #[test]
    fn a_counted_index_is_not_guarded_but_everything_else_inferred_is() {
        let ident: Ident = IdentName { span: DUMMY_SP, sym: "i".into() }.into();
        let index = |source| MapIndex { source, used: false, force_guard: false };
        assert!(!index(IndexSource::Declared(ident.clone())).needs_guard());
        assert!(!index(IndexSource::Counted(ident.clone())).needs_guard());
        assert!(index(IndexSource::Appended(ident.clone())).needs_guard());
        assert!(index(IndexSource::Destructured(ident.clone())).needs_guard());
        assert!(index(IndexSource::RestElement {
            rest: ident.clone(),
            position: 1.0
        })
        .needs_guard());
        // A function only *referenced* as a callback can also be called with no index at
        // all, so even a parameter it declares is a guess.
        assert!(MapIndex {
            source: IndexSource::Declared(ident),
            used: false,
            force_guard: true
        }
        .needs_guard());
    }

    // --- callback references ---

    #[test]
    fn callback_ref_scan_collects_both_argument_positions() {
        let mut scan = CallbackRefScan {
            map_methods: &["map".to_string()],
            collection_first: true,
            names: HashSet::new(),
        };
        // items.map(renderItem)
        call(Some("items"), "map", vec![ident_expr("renderItem")]).visit_with(&mut scan);
        // _.map(items, renderTag)
        call(
            Some("_"),
            "map",
            vec![ident_expr("items"), ident_expr("renderTag")],
        )
        .visit_with(&mut scan);
        // items.filter(keep) — not an iteration name
        call(Some("items"), "filter", vec![ident_expr("keep")]).visit_with(&mut scan);

        assert!(scan.names.contains("renderItem"));
        assert!(scan.names.contains("renderTag"));
        assert!(!scan.names.contains("keep"));
    }

    // --- instance prefix ---

    #[test]
    fn destructured_prop_ident_finds_a_string_keyed_binding() {
        // `({ "data-testid": testId }) => …` — the component already destructures the
        // prop, so it is read rather than rewritten.
        let object = ObjectPat {
            span: DUMMY_SP,
            props: vec![ObjectPatProp::KeyValue(KeyValuePatProp {
                key: PropName::Str(Str {
                    span: DUMMY_SP,
                    value: "data-testid".into(),
                    raw: None,
                }),
                value: Box::new(param("testId")),
            })],
            optional: false,
            type_ann: None,
        };
        assert_eq!(
            ReactDataTestIdTransform::destructured_prop_ident(&object, "data-testid")
                .map(|i| i.sym.to_string()),
            Some("testId".to_string())
        );
        assert!(
            ReactDataTestIdTransform::destructured_prop_ident(&object, "data-cy").is_none(),
            "a different attribute name must not match"
        );
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
            None,
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
            None,
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
            force_guard: false,
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
