use std::collections::HashSet;

use super::*;

pub fn validate(root: &SyntaxNode) -> Vec<String> {
    let mut errors = Vec::new();
    let mut vars = HashSet::new();
    for node in root.descendants() {
        if let Some(it) = Stmt::cast(node.clone()) {
            validate_stmt(it, &mut errors);
        } else if let Some(it) = Label::cast(node.clone()) {
            validate_label(it, &mut vars, &mut errors);
        }
    }
    errors
}

fn validate_stmt(stmt: Stmt, e: &mut Vec<String>) {
    match stmt.kind() {
        StmtKind::Shape(_) => {}
        StmtKind::Object(object) => {
            let params: Vec<Param> = object
                .params()
                .map(Iterator::collect)
                .unwrap_or_else(Vec::new);
            // Objects must have 4 parameters
            if params.len() != 4 {
                error(e, "objects must be a tuple of [value, x, y, shape]");
            }
            // "type check"
            // should be: [value, value, value, name]
            let expected = vec![
                SyntaxKind::Value,
                SyntaxKind::Value,
                SyntaxKind::Value,
                SyntaxKind::Name,
            ];
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
