use std::pin::Pin;

use futures::Future;
use regex::Regex;
use serde_json::Value;

use crate::client::Client;

pub trait Handler {
    fn event(&self) -> Regex;
    fn authentication(&self) -> bool;
    fn call(&self, client: Client, payload: Value) -> Pin<Box<dyn Future<Output = ()>>>;
}

pub struct Handlers {
    routes: Vec<Box<dyn Handler>>,
}

impl Handlers {
    pub fn new() -> Self {
        Handlers { routes: Vec::new() }
    }

    pub fn register<H: Handler + Send + 'static>(mut self, handler: H) -> Self {
        self.routes.push(Box::new(handler));
        self
    }

    pub fn get<T: ToString>(&self, event: T) -> Option<&Box<dyn Handler>> {
        self.routes
            .iter()
            .find(|handler| handler.event().is_match(&event.to_string()))
    }
}
