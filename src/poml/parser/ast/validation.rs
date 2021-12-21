use crate::poml::SyntaxKind as Sk;
use crate::poml::SyntaxNode;

use super::*;

pub fn validate(root: &SyntaxNode) -> Vec<String> {
    let mut errors = Vec::new();
    for node in root.descendants() {
        if let Some(it) = Stmt::cast(node.clone()) {
            validate_stmt(it, &mut errors);
        }
    }
    errors
}

fn validate_stmt(stmt: Stmt, e: &mut Vec<String>) {
    let params: Vec<Param> = stmt
        .params()
        .map(Iterator::collect)
        .unwrap_or_else(Vec::new);
    match stmt.kind() {
        StmtKind::Shape(_) => {}
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

fn error(e: &mut Vec<String>, msg: impl Into<String>) {
    e.push(msg.into());
}
