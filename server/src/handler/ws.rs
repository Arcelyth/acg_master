use actix_web::{HttpRequest, Responder, web};
use actix_ws::{Message, Session};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ClientMsg {
    Join,
    Message(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ServerMsg {
    JoinSucc,
    Response(String),
}

#[derive(Clone)]
pub struct MultiState {
    waiting: Arc<Mutex<Vec<Player>>>,
    rooms: Arc<Mutex<HashMap<String, (Player, Player)>>>,
    user_room: Arc<Mutex<HashMap<String, String>>>,
}

#[derive(Clone)]
pub struct Player {
    pub id: String,
    pub session: Session,
}

pub async fn ws(
    req: HttpRequest,
    body: web::Payload,
    data: web::Data<MultiState>,
) -> actix_web::Result<impl Responder> {
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, body)?;

    let state = data.get_ref().clone();

    actix_web::rt::spawn(async move {
        let mut current_user_id: Option<String> = None;
        while let Some(Ok(msg)) = msg_stream.recv().await {
            match msg {
                Message::Ping(bytes) => {
                    if session.pong(&bytes).await.is_err() {
                        return;
                    }
                }

                Message::Text(msg) => {
                    let Ok(client_msg) = serde_json::from_str::<ClientMsg>(&msg) else {
                        continue;
                    };
                    println!("receive: {:?}", client_msg);
                    match client_msg {
                        ClientMsg::Join => {
                            let user_id = format!("user-{}", Uuid::new_v4());
                            current_user_id = Some(user_id.clone());
                            let mut player = Player {
                                id: user_id.clone(),
                                session: session.clone(),
                            };

                            // TODO:  handle error
                            let mut waiting = state.waiting.lock().unwrap();
                            if let Some(mut another) = waiting.pop() {
                                let room_id = Uuid::new_v4().to_string();
                                // TODO:  handle error
                                let mut rooms = state.rooms.lock().unwrap();
                                rooms.insert(room_id.clone(), (another.clone(), player.clone()));
                                // TODO:  handle error
                                let mut user_room = state.user_room.lock().unwrap();
                                user_room.insert(another.id.clone(), room_id.clone());
                                user_room.insert(user_id.clone(), room_id.clone());
                                // inform both players
                                let _ = another
                                    .session
                                    .text(serde_json::to_string(&ServerMsg::JoinSucc).unwrap())
                                    .await;

                                let _ = player
                                    .session
                                    .text(serde_json::to_string(&ServerMsg::JoinSucc).unwrap())
                                    .await;
                            } else {
                                waiting.push(player);
                            }
                        }
                        ClientMsg::Message(m) => {
                            if let Some(uid) = &current_user_id {
                                let room_id = state.user_room.lock().unwrap().get(uid).cloned();
                                if let Some(rid) = room_id {
                                    // TODO:  handle error
                                    let rooms = state.rooms.lock().unwrap();
                                    if let Some((p1, p2)) = rooms.get(&rid) {
                                        let target = if p1.id == *uid { p2 } else { p1 };
                                        let _ = target
                                            .session
                                            .clone()
                                            .text(
                                                serde_json::to_string(&ServerMsg::Response(m))
                                                    .unwrap(),
                                            )
                                            .await;
                                    }
                                }
                            }
                        }
                    };
                }
                _ => break,
            }
        }

        // clean up
        if let Some(uid) = current_user_id {
            println!("Cleaning up user: {}", uid);
            state.waiting.lock().unwrap().retain(|p| p.id != uid);
        }
        // TODO: clean up room and user_room
        let _ = session.close(None).await;
    });

    Ok(response)
}
