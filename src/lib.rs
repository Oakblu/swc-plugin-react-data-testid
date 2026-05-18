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

fn parse_options(config_json: Option<String>) -> PluginOptions {
    config_json
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

#[plugin_transform]
pub fn process_transform(program: Program, metadata: TransformPluginProgramMetadata) -> Program {
    let options = parse_options(metadata.get_transform_plugin_config());
    let filename = metadata.get_context(&TransformPluginMetadataContextKind::Filename);
    let mut transform = ReactDataTestIdTransform::new_with_filename(options, filename);
    let mut program = program;
    program.visit_mut_with(&mut transform);
    program
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let t = ReactDataTestIdTransform::new(PluginOptions { attributes: None });
        assert_eq!(t.attributes(), vec!["data-testid".to_string()]);
    }

    #[test]
    fn attributes_falls_back_to_default_when_empty_vec() {
        let t = ReactDataTestIdTransform::new(PluginOptions { attributes: Some(vec![]) });
        assert_eq!(t.attributes(), vec!["data-testid".to_string()]);
    }

    #[test]
    fn attributes_returns_custom_list() {
        let t = ReactDataTestIdTransform::new(PluginOptions {
            attributes: Some(vec!["data-cy".to_string(), "data-pw".to_string()]),
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
}
