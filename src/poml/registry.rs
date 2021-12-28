use std::collections::HashMap;

use crate::shapes::Shape;

struct Fun {
    arity: usize,
    handler: Box<dyn Fn(Vec<f32>) -> Shape>,
}

#[derive(Default)]
pub struct Registry {
    functions: HashMap<&'static str, Fun>,
}

impl Registry {
    pub fn register<Args, S: ShapeConstructor<Args>>(mut self, name: &'static str, s: S) -> Self {
        self.functions.insert(
            name,
            Fun {
                arity: S::ARITY,
                handler: Box::new(move |mut stack| s.call(&mut stack)),
            },
        );
        self
    }

    pub fn exists(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }

    pub fn arity(&self, name: &str) -> Option<usize> {
        self.functions.get(name).map(|f| f.arity)
    }

    pub fn call(&self, name: &str, args: Vec<f32>) -> Option<Shape> {
        self.functions.get(name).map(|f| (f.handler)(args))
    }
}

pub trait ShapeConstructor<Args = ()>: 'static + Copy + Send + Sync {
    const ARITY: usize;
    fn call(&self, stack: &mut Vec<f32>) -> Shape;
}

macro_rules! tuple_impls {
    ( $c:expr ; $( $name:ident: $t:ty ),* ) => {
        impl<Fun> ShapeConstructor<($($t,)*)> for Fun
        where
            Fun: 'static + Copy + Send + Sync,
            Fun: Fn($($t),*) -> Shape,
        {
            const ARITY: usize = $c;
            #[allow(unused_variables)]
            fn call(&self, stack: &mut Vec<f32>) -> Shape {
                $( let $name = stack.pop().unwrap(); )*
                (self)($($name,)*)
            }
        }
    };
}

tuple_impls! { 0; }
tuple_impls! { 1; a:f32 }
tuple_impls! { 2; a:f32,b:f32 }
tuple_impls! { 3; a:f32,b:f32,c:f32 }
tuple_impls! { 4; a:f32,b:f32,c:f32,d:f32 }
