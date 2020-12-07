use std::{fmt::Debug, sync::Arc};

use super::{dynamic::Dynamic, interpreter::Interpreter};

#[derive(Clone)]
pub struct Method {
    name: String,
    inner: Arc<dyn Fn(&mut Interpreter, Dynamic, Vec<Dynamic>) -> Dynamic>,
}

impl Method {
    pub fn invoke(&self, i: &mut Interpreter, this: Dynamic, args: Vec<Dynamic>) -> Dynamic {
        (self.inner)(i, this, args)
    }

    pub fn new<T>(name: &str, f: T) -> Self
    where
        T: Fn(&mut Interpreter, Dynamic, Vec<Dynamic>) -> Dynamic + 'static,
    {
        Method {
            name: name.to_string(),
            inner: Arc::new(f),
        }
    }

    pub fn into_dynamic(&self, this: Dynamic) -> Dynamic {
        let f = self.inner.clone();
        Dynamic::RustFunction {
            name: self.name.clone(),
            scope: None,
            callback: Arc::new(move |i, args| Some((f)(i, this.clone(), args))),
        }
    }
}

impl Debug for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "method {}(...)", self.name)
    }
}
