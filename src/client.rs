use std::collections::HashSet;

use actix_ws::Session;
use serde::Serialize;
use uuid::Uuid;

use crate::{event::Event, server::Server};

#[derive(Clone)]
pub struct Client {
    pub(crate) id: Uuid,
    pub(crate) server: Server,
    pub(crate) session: Session,
}

impl Client {
    pub fn id(&self) -> Uuid {
        self.id
    }

    pub async fn subscribes(&self) -> HashSet<String> {
        self.server.subscribes(self.id).await
    }

    pub async fn subscribe<T: ToString>(&self, event: T) {
        self.server.subscribe(self.id, event).await;
    }

    pub async fn unsubscribe<T: ToString>(&self, event: T) {
        self.server.unsubscribe(self.id, event).await;
    }

    pub async fn subscribed<T: ToString>(&self, event: T) -> bool {
        self.server.subscribed(self.id, event).await
    }

    pub(crate) async fn send<T: ToString>(&self, message: T) {
        self.server.send(self.id(), message).await;
    }

    pub(crate) async fn json<S: Serialize>(&self, message: S) {
        self.send(serde_json::to_string(&message).unwrap()).await;
    }

    pub async fn push(&self, event: Event) {
        self.json(event).await;
    }

    pub async fn broadcast(&self, event: Event) {
        self.server.broadcast(event).await;
    }

    pub(crate) async fn pong<T: AsRef<[u8]>>(&mut self, message: T) {
        match self.session.pong(message.as_ref()).await {
            Ok(_) => (),
            Err(_) => {
                self.server.unregister(self.id).await;
            }
        }
    }

    pub(crate) async fn close(&self) {
        self.server.unregister(self.id).await;
    }
}
