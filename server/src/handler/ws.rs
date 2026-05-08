use actix_web::{HttpRequest, Responder, web};
use actix_ws::{Message, Session};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
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
    ILeave, // sender leave
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
    GuessResp(WsGuessResponse, usize),
    OGuessResp(BangumiSubjectHide), // another guy's resp
    Over(bool, (BangumiSubject, CompareResult)),
    Reset,
    ResetOk,
    Leave(BangumiSubject, CompareResult), // opponent leave
}

#[derive(Clone)]
pub struct Room {
    pub p1: Player,
    pub p2: Player,
    pub answer: BangumiSubject,
    pub reset: (bool, bool),
    pub max_guess: usize,
    pub guess_time: (usize, usize),
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
    const MAX_GUESS: usize = 20;
    let state = data.get_ref().clone();

    actix_web::rt::spawn(async move {
        let mut current_user_id: Option<String> = None;

        // token bucket
        let mut last_tick = Instant::now();
        let mut tokens: f64 = 10.0;
        let max_tokens: f64 = 10.0;
        let refill_rate: f64 = 2.0;

        while let Some(Ok(msg)) = msg_stream.recv().await {
            match msg {
                Message::Ping(bytes) => {
                    if session.pong(&bytes).await.is_err() {
                        return;
                    }
                }

                Message::Text(msg) => {
                    // rate limiting
                    let now = Instant::now();
                    let elapsed = now.duration_since(last_tick).as_secs_f64();
                    last_tick = now;
                    tokens = (tokens + elapsed * refill_rate).min(max_tokens);
                    if tokens < 1.0 {
                        println!(
                            "Rate limit exceeded for user: {:?}. Closing connection.",
                            current_user_id
                        );
                        break;
                    }
                    tokens -= 1.0;

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
                                                max_guess: MAX_GUESS,
                                                guess_time: (0, 0),
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
                                let target_sess = {
                                    let rid = state.user_room.lock().unwrap().get(uid).cloned();

                                    if let Some(rid) = rid {
                                        let rooms = state.rooms.lock().unwrap();
                                        rooms.get(&rid).map(|room| {
                                            if room.p1.id == *uid {
                                                room.p2.session.clone()
                                            } else {
                                                room.p1.session.clone()
                                            }
                                        })
                                    } else {
                                        None
                                    }
                                };

                                if let Some(mut sess) = target_sess {
                                    let _ = sess
                                        .text(
                                            serde_json::to_string(&ServerMsg::Response(m)).unwrap(),
                                        )
                                        .await;
                                }
                            }
                        }
                        ClientMsg::Guess(guess) => {
                            let uid = match &current_user_id {
                                Some(id) => id.clone(),
                                None => continue,
                            };
                            let rid = {
                                let map = state.user_room.lock().unwrap();
                                map.get(&uid).cloned()
                            };
                            let Some(rid) = rid else {
                                continue;
                            };
                            let (
                                is_correct,
                                comparison,
                                answer,
                                p1_id,
                                p1_sess,
                                p2_sess,
                                right_comp,
                                comp_hide,
                                is_draw,
                                cur_gt,
                            ) = {
                                let mut rooms = state.rooms.lock().unwrap();
                                let Some(room) = rooms.get_mut(&rid) else {
                                    continue;
                                };

                                let is_p1 = room.p1.id == uid;

                                let cur_gt = if is_p1 {
                                    if room.guess_time.0 >= room.max_guess {
                                        continue;
                                    }
                                    room.guess_time.0 += 1;
                                    room.guess_time.0
                                } else {
                                    if room.guess_time.1 >= room.max_guess {
                                        continue;
                                    }
                                    room.guess_time.1 += 1;
                                    room.guess_time.1
                                };

                                let is_correct = is_guess_right(&guess, &room.answer);
                                let comparison = compare_anime(&guess, &room.answer);
                                let right_comp = compare_anime(&room.answer, &room.answer);
                                let comp_hide = get_hide_subject(&room.answer, &guess);

                                let is_draw = !is_correct
                                    && room.guess_time.0 >= room.max_guess
                                    && room.guess_time.1 >= room.max_guess;

                                (
                                    is_correct,
                                    comparison,
                                    room.answer.clone(),
                                    room.p1.id.clone(),
                                    room.p1.session.clone(),
                                    room.p2.session.clone(),
                                    right_comp,
                                    comp_hide,
                                    is_draw,
                                    cur_gt,
                                )
                            };

                            let is_p1 = p1_id == uid;
                            let cur_sess = if is_p1 { &p1_sess } else { &p2_sess };
                            let target_sess = if is_p1 { &p2_sess } else { &p1_sess };

                            let resp = WsGuessResponse {
                                guess,
                                comparison: comparison.clone(),
                            };
                            let _ = cur_sess
                                .clone()
                                .text(
                                    serde_json::to_string(&ServerMsg::GuessResp(resp, cur_gt))
                                        .unwrap(),
                                )
                                .await;

                            let _ = target_sess
                                .clone()
                                .text(
                                    serde_json::to_string(&ServerMsg::OGuessResp(comp_hide))
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
                            } else if is_draw {
                                let draw_msg = serde_json::to_string(&ServerMsg::Over(
                                    false,
                                    (answer.clone(), right_comp),
                                ))
                                .unwrap();

                                let _ = p1_sess.clone().text(draw_msg.clone()).await;
                                let _ = p2_sess.clone().text(draw_msg).await;
                            }
                        }

                        ClientMsg::Reset => {
                            let uid = match &current_user_id {
                                Some(uid) => uid.clone(),
                                None => continue,
                            };

                            let rid = {
                                let map = state.user_room.lock().unwrap();
                                map.get(&uid).cloned()
                            };

                            let Some(rid) = rid else {
                                continue;
                            };

                            let (p1_session, p2_session, should_restart) = {
                                let mut rooms = state.rooms.lock().unwrap();

                                let Some(room) = rooms.get_mut(&rid) else {
                                    continue;
                                };

                                if room.p1.id == uid {
                                    room.reset.0 = true;
                                } else {
                                    room.reset.1 = true;
                                }

                                let should_restart = room.reset == (true, true);

                                (
                                    room.p1.session.clone(),
                                    room.p2.session.clone(),
                                    should_restart,
                                )
                            };

                            let _ = session
                                .text(serde_json::to_string(&ServerMsg::ResetOk).unwrap())
                                .await;

                            if should_restart {
                                if let Some(answer) = fetch_random_anime(1960, 2026).await {
                                    {
                                        let mut rooms = state.rooms.lock().unwrap();

                                        if let Some(room) = rooms.get_mut(&rid) {
                                            room.answer = answer.clone();
                                            room.reset = (false, false);
                                            room.guess_time = (0, 0);
                                        } else {
                                            continue;
                                        }
                                    }

                                    let reset_msg =
                                        serde_json::to_string(&ServerMsg::Reset).unwrap();

                                    let _ = p1_session.clone().text(reset_msg.clone()).await;

                                    let _ = p2_session.clone().text(reset_msg).await;
                                }
                            }
                        }

                        ClientMsg::ILeave => break,
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
                        .text(
                            serde_json::to_string(&ServerMsg::Leave(room.answer, right_comp))
                                .unwrap(),
                        )
                        .await;
                }
            }
        }
        let _ = session.close(None).await;
    });

    Ok(response)
}
