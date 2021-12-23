use std::collections::HashMap;

use crate::Sdf;

struct Fun {
    arity: usize,
    handler: Box<dyn Fn(Vec<f32>) -> Box<dyn Sdf>>,
}

#[derive(Default)]
pub struct Registry {
    functions: HashMap<&'static str, Fun>,
}

impl Registry {
    pub fn register<Args, S: Function<Args>>(mut self, name: &'static str, s: S) -> Self {
        self.functions.insert(
            name,
            Fun {
                arity: S::ARITY,
                handler: Box::new(move |mut stack| Box::new(s.call(&mut stack))),
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

    pub fn call(&self, name: &str, args: Vec<f32>) -> Option<Box<dyn Sdf>> {
        self.functions.get(name).map(|f| (f.handler)(args))
    }
}

pub trait Function<Args = ()>: 'static + Copy + Send + Sync {
    type Return: Sdf + 'static;
    const ARITY: usize;
    fn call(&self, stack: &mut Vec<f32>) -> Self::Return;
}

macro_rules! tuple_impls {
    ( $c:expr ; $( $name:ident: $t:ty ),* ) => {
        impl<Fun, Res> Function<($($t,)*)> for Fun
        where
            Fun: 'static + Copy + Send + Sync,
            Fun: Fn($($t),*) -> Res,
            Res: Sdf + 'static,
        {
            type Return = Res;
            const ARITY: usize = $c;
            #[allow(unused_variables)]
            fn call(&self, stack: &mut Vec<f32>) -> Self::Return {
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
