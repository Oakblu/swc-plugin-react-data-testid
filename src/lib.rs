use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;
use swc_core::{
    common::{plugin::metadata::TransformPluginMetadataContextKind, DUMMY_SP},
    ecma::{
        ast::*,
        visit::{VisitMut, VisitMutWith},
    },
    plugin::{plugin_transform, proxies::TransformPluginProgramMetadata},
};

const DEFAULT_ATTRIBUTE: &str = "data-testid";

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
}

impl ReactDataTestIdTransform {
    pub fn new(options: PluginOptions) -> Self {
        Self::new_with_filename(options, None)
    }

    pub fn new_with_filename(options: PluginOptions, filename: Option<String>) -> Self {
        Self {
            options,
            component_counters: HashMap::new(),
            current_component: None,
            current_class: None,
            filename_component_name: filename
                .as_deref()
                .and_then(component_name_from_filename),
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

        for attr_name in attributes {
            if !Self::has_attribute(&opening.attrs, &attr_name) {
                opening.attrs.insert(0, Self::make_attr(&attr_name, &test_id));
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

    fn visit_mut_jsx_element(&mut self, n: &mut JSXElement) {
        self.inject_attributes(&mut n.opening);
        n.visit_mut_children_with(self);
    }
}

#[plugin_transform]
pub fn process_transform(program: Program, metadata: TransformPluginProgramMetadata) -> Program {
    let options: PluginOptions = metadata
        .get_transform_plugin_config()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();

    let filename = metadata.get_context(&TransformPluginMetadataContextKind::Filename);
    let mut transform = ReactDataTestIdTransform::new_with_filename(options, filename);
    let mut program = program;
    program.visit_mut_with(&mut transform);
    program
}
