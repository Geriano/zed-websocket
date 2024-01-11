pub mod client;
pub mod event;
pub mod handler;
pub mod server;
pub mod service;

use actix_web::web::Data;
pub use client::Client;
pub use derives::listener;
pub use event::Event;
pub use futures::Future;
pub use handler::Handlers;
pub use regex::Regex;
pub use server::Server;
pub use service::{broadcast, listen};

pub type Websocket = Data<Server>;
