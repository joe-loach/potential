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

    pub fn params(&self) -> Option<Params> {
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
    fn params(&self) -> Params {
        Params::new(self.syntax().children().filter_map(Param::cast))
    }
}

pub struct Params {
    iter: Box<dyn Iterator<Item = Param>>,
}

impl Params {
    fn new(iter: impl Iterator<Item = Param> + 'static) -> Self {
        let iter = Box::new(iter);
        Self { iter }
    }

    pub fn values(self) -> impl Iterator<Item = Value> {
        self.iter.filter_map(|p| p.kind().try_into_value().ok())
    }

    pub fn next_value(&mut self) -> Option<Value> {
        self.iter
            .next()
            .map(|p| p.kind())
            .and_then(|p| p.try_into_value().ok())
    }

    pub fn next_name(&mut self) -> Option<Name> {
        self.iter
            .next()
            .map(|p| p.kind())
            .and_then(|p| p.try_into_name().ok())
    }
}

impl Iterator for Params {
    type Item = Param;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

#[derive(Debug)]
pub enum ParamKind {
    Name(Name),
    Value(Value),
}

impl ParamKind {
    pub fn as_name(&self) -> Option<&Name> {
        if let Self::Name(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_value(&self) -> Option<&Value> {
        if let Self::Value(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn try_into_name(self) -> Result<Name, Self> {
        if let Self::Name(v) = self {
            Ok(v)
        } else {
            Err(self)
        }
    }

    pub fn try_into_value(self) -> Result<Value, Self> {
        if let Self::Value(v) = self {
            Ok(v)
        } else {
            Err(self)
        }
    }
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

impl Label {
    pub fn text(&self) -> Option<String> {
        self.syntax()
            .children()
            .find_map(Name::cast)
            .map(|name| name.text())
    }
}

impl Name {
    pub fn text(&self) -> String {
        self.syntax()
            .green()
            .children()
            .find_map(|it| {
                it.as_token().and_then(|t| {
                    if t.kind() == SyntaxKind::Ident.into() {
                        Some(t.text().to_string())
                    } else {
                        None
                    }
                })
            })
            .unwrap()
    }
}

impl Value {
    pub fn value(&self) -> f32 {
        // cannot fail as the lexer will always produce parsable literals
        self.syntax()
            .green()
            .children()
            .find_map(|it| {
                it.as_token().and_then(|t| {
                    if t.kind() == SyntaxKind::Literal.into() {
                        t.text().parse::<f32>().ok()
                    } else {
                        None
                    }
                })
            })
            .unwrap()
    }
}
