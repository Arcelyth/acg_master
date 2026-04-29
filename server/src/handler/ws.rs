use actix_web::{HttpRequest, Responder, web};
use actix_ws::{Message, Session};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::handler::bangumi::*;

#[derive(Clone)]
pub struct MultiState {
    waiting: Arc<Mutex<Vec<Player>>>,
    rooms: Arc<Mutex<HashMap<String, Room>>>,
    user_room: Arc<Mutex<HashMap<String, String>>>,
}

impl MultiState {
    pub fn new() -> Self {
        Self {
            waiting: Arc::new(Mutex::new(vec![])),
            rooms: Arc::new(Mutex::new(HashMap::new())),
            user_room: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ClientMsg {
    Join(String), // name
    Message(String),
    Guess(BangumiSubject),
    Reset,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WsGuessResponse {
    pub guess: BangumiSubject,
    pub comparison: CompareResult,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ServerMsg {
    JoinSucc(String, String),
    Response(String),
    GuessResp(WsGuessResponse),
    OGuessResp(BangumiSubjectHide), // another guy's resp
    Over(bool, (BangumiSubject, CompareResult)),
    Reset,
    ResetOk,
    Leave(BangumiSubject, CompareResult),
}

#[derive(Clone)]
pub struct Room {
    pub p1: Player,
    pub p2: Player,
    pub answer: BangumiSubject,
    pub reset: (bool, bool),
}

#[derive(Clone)]
pub struct Player {
    pub id: String,
    pub name: String,
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
                    match client_msg {
                        ClientMsg::Join(name) => {
                            let user_id = format!("user-{}", Uuid::new_v4());
                            current_user_id = Some(user_id.clone());
                            let player = Player {
                                id: user_id.clone(),
                                name: name.clone(),
                                session: session.clone(),
                            };

                            let mut waiting = state.waiting.lock().unwrap();
                            if let Some(another) = waiting.pop() {
                                drop(waiting);
                                if let Some(answer) = fetch_random_anime(1960, 2026).await {
                                    let room_id = Uuid::new_v4().to_string();

                                    {
                                        let mut rooms = state.rooms.lock().unwrap();
                                        println!(
                                            "Generate answer: {} \n {}",
                                            answer.name, answer.name_cn
                                        );
                                        rooms.insert(
                                            room_id.clone(),
                                            Room {
                                                p1: another.clone(),
                                                p2: player.clone(),
                                                answer: answer.clone(),
                                                reset: (false, false),
                                            },
                                        );
                                    }

                                    {
                                        let mut user_room = state.user_room.lock().unwrap();
                                        user_room.insert(another.id.clone(), room_id.clone());
                                        user_room.insert(user_id.clone(), room_id.clone());
                                    }

                                    let _ = another
                                        .session
                                        .clone()
                                        .text(
                                            serde_json::to_string(&ServerMsg::JoinSucc(
                                                name.clone(),
                                                another.name.clone(),
                                            ))
                                            .unwrap(),
                                        )
                                        .await;

                                    let _ = player
                                        .session
                                        .clone()
                                        .text(
                                            serde_json::to_string(&ServerMsg::JoinSucc(
                                                another.name.clone(),
                                                name,
                                            ))
                                            .unwrap(),
                                        )
                                        .await;
                                }
                            } else {
                                waiting.push(player);
                                println!("Someone start waiting...");
                            }
                        }
                        ClientMsg::Message(m) => {
                            if let Some(uid) = &current_user_id {
                                let rid = state.user_room.lock().unwrap().get(uid).cloned();
                                if let Some(rid) = rid {
                                    let rooms = state.rooms.lock().unwrap();
                                    if let Some(room) = rooms.get(&rid) {
                                        let target = if room.p1.id == *uid {
                                            &room.p2
                                        } else {
                                            &room.p1
                                        };
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
                        ClientMsg::Guess(guess) => {
                            if let Some(uid) = &current_user_id {
                                let rid = state.user_room.lock().unwrap().get(uid).cloned();
                                if let Some(rid) = rid {
                                    let (
                                        is_correct,
                                        comparison,
                                        answer,
                                        p1_id,
                                        p1_sess,
                                        p2_sess,
                                        right_comp,
                                        comp_hide,
                                    ) = {
                                        let rooms = state.rooms.lock().unwrap();
                                        if let Some(room) = rooms.get(&rid) {
                                            let is_correct = is_guess_right(&guess, &room.answer);
                                            let comparison = compare_anime(&guess, &room.answer);
                                            let right_comp =
                                                compare_anime(&room.answer, &room.answer);
                                            let comp_hide = get_hide_subject(&room.answer, &guess);
                                            (
                                                is_correct,
                                                comparison,
                                                room.answer.clone(),
                                                room.p1.id.clone(),
                                                room.p1.session.clone(),
                                                room.p2.session.clone(),
                                                right_comp,
                                                comp_hide,
                                            )
                                        } else {
                                            continue;
                                        }
                                    };

                                    let cur_sess = if p1_id == *uid { &p1_sess } else { &p2_sess };
                                    let target_sess =
                                        if p1_id == *uid { &p2_sess } else { &p1_sess };

                                    let resp = WsGuessResponse {
                                        guess,
                                        comparison: comparison.clone(),
                                    };
                                    let _ = cur_sess
                                        .clone()
                                        .text(
                                            serde_json::to_string(&ServerMsg::GuessResp(resp))
                                                .unwrap(),
                                        )
                                        .await;
                                    let _ = target_sess
                                        .clone()
                                        .text(
                                            serde_json::to_string(&ServerMsg::OGuessResp(
                                                comp_hide,
                                            ))
                                            .unwrap(),
                                        )
                                        .await;

                                    if is_correct {
                                        let _ = cur_sess
                                            .clone()
                                            .text(
                                                serde_json::to_string(&ServerMsg::Over(
                                                    true,
                                                    (answer.clone(), right_comp.clone()),
                                                ))
                                                .unwrap(),
                                            )
                                            .await;
                                        let _ = target_sess
                                            .clone()
                                            .text(
                                                serde_json::to_string(&ServerMsg::Over(
                                                    false,
                                                    (answer.clone(), right_comp),
                                                ))
                                                .unwrap(),
                                            )
                                            .await;
                                    }
                                }
                            }
                        }
                        ClientMsg::Reset => {
                            if let Some(uid) = &current_user_id {
                                let rid = state.user_room.lock().unwrap().get(uid).cloned();
                                if let Some(rid) = rid {
                                    let mut rooms = state.rooms.lock().unwrap();
                                    if let Some(room) = rooms.get_mut(&rid) {
                                        if room.p1.id == *uid {
                                            room.reset.0 = true;
                                        } else {
                                            room.reset.1 = true;
                                        };

                                        // restart game
                                        if room.p1.id == *uid {
                                            let _ = room
                                                .p1
                                                .session
                                                .text(
                                                    serde_json::to_string(&ServerMsg::ResetOk)
                                                        .unwrap(),
                                                )
                                                .await;
                                        } else {
                                            let _ = room
                                                .p2
                                                .session
                                                .text(
                                                    serde_json::to_string(&ServerMsg::ResetOk)
                                                        .unwrap(),
                                                )
                                                .await;
                                        };
                                        if room.reset == (true, true) {
                                            if let Some(answer) =
                                                fetch_random_anime(1960, 2026).await
                                            {
                                                println!(
                                                    "Generate answer: {} \n {}",
                                                    answer.name, answer.name_cn
                                                );

                                                let _ = room
                                                    .p1
                                                    .session
                                                    .clone()
                                                    .text(
                                                        serde_json::to_string(&ServerMsg::Reset)
                                                            .unwrap(),
                                                    )
                                                    .await;

                                                let _ = room
                                                    .p2
                                                    .session
                                                    .clone()
                                                    .text(
                                                        serde_json::to_string(&ServerMsg::Reset)
                                                            .unwrap(),
                                                    )
                                                    .await;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    };
                }
                _ => break,
            }
        }

        if let Some(uid) = current_user_id {
            println!("Cleaning up user: {}", uid);
            state.waiting.lock().unwrap().retain(|p| p.id != uid);

            let rid = state.user_room.lock().unwrap().remove(&uid);
            if let Some(rid) = rid {
                let mut rooms = state.rooms.lock().unwrap();
                if let Some(room) = rooms.remove(&rid) {
                    let mut opponent_session = if room.p1.id == uid {
                        room.p2.session.clone()
                    } else {
                        room.p1.session.clone()
                    };
                    let opponent_id = if room.p1.id == uid {
                        room.p2.id
                    } else {
                        room.p1.id
                    };

                    state.user_room.lock().unwrap().remove(&opponent_id);
                    drop(rooms);
                    let right_comp = compare_anime(&room.answer, &room.answer);

                    let _ = opponent_session
                        .text(serde_json::to_string(&ServerMsg::Leave(room.answer, right_comp)).unwrap())
                        .await;
                }
            }
        }
        let _ = session.close(None).await;
    });

    Ok(response)
}
