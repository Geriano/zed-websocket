use actix_web::web::Json;
use actix_web::{web, HttpRequest, HttpResponse};
use actix_ws::Message;
use futures::StreamExt;
use serde_json::json;

use crate::event::{Event, Payload};
use crate::handler::Handlers;
use crate::server::Server;

pub async fn broadcast(server: web::Data<Server>, request: Json<Payload>) -> HttpResponse {
    let event = Event {
        name: request.event.clone(),
        data: request.payload.clone(),
    };

    println!("Broadcasting event: {} {}", event.name, event.data);

    server.broadcast(event).await;

    HttpResponse::Ok().finish()
}

pub async fn listen(
    req: HttpRequest,
    body: web::Payload,
    routes: web::Data<Handlers>,
    server: web::Data<Server>,
) -> HttpResponse {
    let handshake = actix_ws::handle(&req, body);

    println!("Handshake: {:?}", handshake.is_ok());

    if let Err(e) = handshake {
        return HttpResponse::BadRequest().json(json!({
            "message": e.to_string(),
        }));
    }

    let (response, session, mut stream) = handshake.unwrap();

    println!("Incoming connection: {:?}", req.peer_addr());

    let mut client = server.register(session).await;

    println!("{} connected", client.id());

    actix::spawn(async move {
        while let Some(Ok(message)) = stream.next().await {
            match message {
                Message::Ping(message) => client.pong(message).await,
                Message::Close(_) => {
                    println!("Client {} Closed", client.id());
                    client.close().await;
                }
                Message::Text(message) => {
                    let payload = serde_json::from_str::<Payload>(&message);

                    if let Err(e) = payload {
                        client
                            .push(Event {
                                name: "error".to_string(),
                                data: json!({
                                    "message": e.to_string(),
                                }),
                            })
                            .await;

                        continue;
                    }

                    let payload = payload.unwrap();

                    match routes.get(payload.event) {
                        Some(handler) => {
                            // if handler.authentication() && !client.authenticated() {
                            //     client.push(Event {
                            //         name: "error".to_string(),
                            //         data: json!({
                            //             "message": "unauthenticated",
                            //         }),
                            //     }).await;

                            //     continue;
                            // }

                            handler.call(client.clone(), payload.payload).await;
                        }
                        None => {
                            let event = Event {
                                name: "error".to_string(),
                                data: json!({
                                    "message": "event not found",
                                }),
                            };

                            client.push(event).await;
                        }
                    }
                }
                _ => (),
            }
        }

        client.close().await;
    });

    println!("Connection established");

    response
}
