mod builder;

pub use builder::*;

use crate::poml::{SyntaxKind, SyntaxNode};

pub trait AstNode {
    fn cast(node: SyntaxNode) -> Option<Self>
    where
        Self: Sized;

    fn syntax(&self) -> &SyntaxNode;
}

macro_rules! node {
    ($name:ident, $kind:path) => {
        #[derive(Debug)]
        pub struct $name(SyntaxNode);

        impl AstNode for $name {
            fn cast(node: SyntaxNode) -> Option<Self> {
                match node.kind() {
                    $kind => Some(Self(node)),
                    _ => None,
                }
            }

            fn syntax(&self) -> &SyntaxNode {
                &self.0
            }
        }
    };
}

node!(Root, SyntaxKind::Root);
node!(Shape, SyntaxKind::Shape);
node!(Object, SyntaxKind::Object);
node!(Label, SyntaxKind::Label);

impl Root {
    pub fn stmts(&self) -> impl Iterator<Item = Stmt> {
        self.syntax().children().filter_map(Stmt::cast)
    }
}

#[derive(Debug)]
pub struct Stmt(SyntaxNode);

pub enum StmtKind {
    Shape(Shape),
    Object(Object),
}

impl AstNode for Stmt {
    fn cast(node: SyntaxNode) -> Option<Self>
    where
        Self: Sized {
        if Shape::cast(node.clone()).is_some() || Object::cast(node.clone()).is_some() {
            Some(Stmt(node))
        } else {
            None
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.0
    }
}

impl Stmt {
    pub fn kind(&self) -> StmtKind {
        Shape::cast(self.0.clone())
            .map(StmtKind::Shape)
            .or_else(|| Object::cast(self.0.clone()).map(StmtKind::Object))
            .unwrap()
    }

    pub fn label(&self) -> Option<Label> {
        self.syntax().children().find_map(Label::cast)
    }
}