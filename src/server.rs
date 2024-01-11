use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use actix_ws::Session;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::client::Client;
use crate::event::Event;

#[derive(Clone)]
pub struct Server {
    sessions: Arc<Mutex<HashMap<Uuid, Session>>>,
    subscribes: Arc<Mutex<HashMap<Uuid, HashSet<String>>>>,
}

impl Server {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            subscribes: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn sessions(&self) -> HashMap<Uuid, Session> {
        let sessions = self.sessions.lock().await;

        sessions.clone()
    }

    pub async fn clients(&self) -> Vec<Client> {
        self.sessions()
            .await
            .iter()
            .map(|(id, session)| Client {
                id: *id,
                server: self.clone(),
                session: session.clone(),
            })
            .collect()
    }

    pub async fn register(&self, session: Session) -> Client {
        let id = Uuid::new_v4();
        let server = self.clone();
        self.sessions.lock().await.insert(id, session.clone());
        self.subscribes.lock().await.insert(id, HashSet::new());

        Client {
            id,
            server,
            session,
        }
    }

    pub async fn unregister(&self, id: Uuid) {
        match self.sessions.lock().await.get(&id).cloned() {
            Some(session) => {
                let sessions = Arc::clone(&self.sessions);
                let subscribes = Arc::clone(&self.subscribes);

                actix::spawn(async move {
                    sessions.lock().await.remove(&id);
                    subscribes.lock().await.remove(&id);
                    session.close(None).await.ok();

                    println!("{} disconnected", id);
                });
            }
            None => (),
        }
    }

    pub async fn subscribes(&self, id: Uuid) -> HashSet<String> {
        self.subscribes.lock().await.get(&id).unwrap().clone()
    }

    pub async fn subscribe<T: ToString>(&self, id: Uuid, event: T) {
        self.subscribes
            .lock()
            .await
            .get_mut(&id)
            .unwrap()
            .insert(event.to_string());
    }

    pub async fn unsubscribe<T: ToString>(&self, id: Uuid, event: T) {
        self.subscribes
            .lock()
            .await
            .get_mut(&id)
            .unwrap()
            .remove(&event.to_string());
    }

    pub async fn subscribed<T: ToString>(&self, id: Uuid, event: T) -> bool {
        self.subscribes
            .lock()
            .await
            .get_mut(&id)
            .unwrap()
            .contains(&event.to_string())
    }

    pub async fn send<T: ToString>(&self, id: Uuid, message: T) {
        match self.sessions.lock().await.get(&id).cloned() {
            Some(mut session) => match session.text(message.to_string()).await {
                Ok(_) => (),
                Err(_) => {
                    self.unregister(id).await;
                }
            },
            None => (),
        }
    }

    pub async fn broadcast(&self, event: Event) {
        let clients = self.clients().await;

        for client in clients {
            if client.subscribed(&event.name).await {
                client.push(event.clone()).await;
            }
        }
    }

    #[allow(dead_code)]
    pub async fn others(&self, client: Client, event: Event) {
        let clients = self.clients().await;

        for other in clients {
            if other.id() != client.id() && other.subscribed(&event.name).await {
                other.push(event.clone()).await;
            }
        }
    }
}
