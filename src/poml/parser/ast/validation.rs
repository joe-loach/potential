use std::collections::HashSet;

use crate::poml::Registry;
use crate::poml::SyntaxKind as Sk;
use crate::poml::SyntaxNode;

use super::*;

pub fn validate(root: &SyntaxNode, registry: &Registry) -> Vec<String> {
    let mut errors = Vec::new();
    let mut vars = HashSet::new();
    for node in root.descendants() {
        if let Some(it) = Stmt::cast(node.clone()) {
            validate_stmt(it, registry, &mut errors);
        } else if let Some(it) = Label::cast(node.clone()) {
            validate_label(it, &mut vars, &mut errors);
        }
    }
    errors
}

fn validate_stmt(stmt: Stmt, registry: &Registry, e: &mut Vec<String>) {
    let params: Vec<Param> = stmt
        .params()
        .map(Iterator::collect)
        .unwrap_or_else(Vec::new);
    match stmt.kind() {
        StmtKind::Shape(shape) => {
            // Shape name must be registered
            if let Some(name) = shape.name().map(|name| name.text()) {
                if !registry.exists(&name) {
                    error(e, format!("shape `{}` is not recognised", name));
                } else {
                    // Shape must have the same arity
                    let arity = registry.arity(&name).unwrap();
                    if arity != params.len() {
                        error(
                            e,
                            format!("expected to find {} args, found {}", arity, params.len()),
                        )
                    }
                }
            }
            // Shapes must only have "value" parameters
            for param in params {
                let found = param.kind().syntax().kind();
                if found != Sk::Value {
                    error(
                        e,
                        format!(
                            "shapes only have `Value` parameters, found a `{:?}` instead",
                            found
                        ),
                    );
                }
            }
        }
        StmtKind::Object(_) => {
            // Objects must have 4 parameters
            if params.len() != 4 {
                error(e, "objects must be a tuple of [value, x, y, shape]");
            }
            // "type check"
            // should be: [value, value, value, name]
            let expected = [Sk::Value, Sk::Value, Sk::Value, Sk::Name];
            for (param, exp) in params.iter().zip(expected) {
                let found = param.kind().syntax().kind();
                if found != exp {
                    error(
                        e,
                        format!(
                            "expected to find a `{:?}`, got a `{:?}` instead",
                            exp, found
                        ),
                    );
                }
            }
        }
    }
}

fn validate_label(label: Label, state: &mut HashSet<String>, e: &mut Vec<String>) {
    if let Some(text) = label.text() {
        let exists = !state.insert(text.clone());
        if exists {
            error(e, format!("label `{}` has multiple definitions", text))
        }
    }
}

fn error(e: &mut Vec<String>, msg: impl Into<String>) {
    e.push(msg.into());
}
