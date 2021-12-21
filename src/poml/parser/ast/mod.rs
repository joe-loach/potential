mod builder;
mod validation;

pub use builder::*;
pub use validation::validate;

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
node!(Value, SyntaxKind::Value);
node!(Label, SyntaxKind::Label);
node!(Name, SyntaxKind::Name);
node!(ParamList, SyntaxKind::ParamList);
node!(Param, SyntaxKind::Param);

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
        Self: Sized,
    {
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

    pub fn params(&self) -> Option<impl Iterator<Item = Param>> {
        let list = self.syntax().children().find_map(ParamList::cast)?;
        Some(list.params())
    }
}

impl Shape {
    pub fn label(&self) -> Label {
        self.syntax().children().find_map(Label::cast).unwrap()
    }

    pub fn name(&self) -> Option<Name> {
        self.syntax().children().find_map(Name::cast)
    }
}

impl ParamList {
    fn params(&self) -> impl Iterator<Item = Param> {
        self.syntax().children().filter_map(Param::cast)
    }
}

#[derive(Debug)]
pub enum ParamKind {
    Name(Name),
    Value(Value),
}

impl Param {
    pub fn kind(&self) -> ParamKind {
        let child = self.syntax().children().next().unwrap();
        ParamKind::cast(child).unwrap()
    }
}

impl AstNode for ParamKind {
    fn cast(node: SyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        Name::cast(node.clone())
            .map(ParamKind::Name)
            .or_else(|| Value::cast(node.clone()).map(ParamKind::Value))
    }

    fn syntax(&self) -> &SyntaxNode {
        match self {
            ParamKind::Name(it) => it.syntax(),
            ParamKind::Value(it) => it.syntax(),
        }
    }
}

impl Name {
    pub fn text(&self) -> Option<String> {
        self.syntax().green().children().find_map(|it| {
            it.as_token().and_then(|t| {
                if t.kind() == SyntaxKind::Ident.into() {
                    Some(t.text().to_string())
                } else {
                    None
                }
            })
        })
    }
}

impl Value {
    pub fn value(&self) -> Option<f32> {
        self.syntax().green().children().find_map(|it| {
            it.as_token().and_then(|t| {
                if t.kind() == SyntaxKind::Literal.into() {
                    t.text().parse::<f32>().ok()
                } else {
                    None
                }
            })
        })
    }
}
