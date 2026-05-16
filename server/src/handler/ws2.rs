use actix_web::{HttpRequest, HttpResponse, Responder, web};
use actix_ws::{Message, Session};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use uuid::Uuid;

use crate::handler::bangumi::*;

#[derive(Clone)]
pub struct MultiState {
    rooms: Arc<Mutex<HashMap<String, Room>>>,
    user_room: Arc<Mutex<HashMap<String, String>>>,
}

impl MultiState {
    pub fn new() -> Self {
        Self {
            rooms: Arc::new(Mutex::new(HashMap::new())),
            user_room: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ClientMsg {
    Join(String, String),       // room_id and username
    CreateRoom(String, String), // room name and creator's name
    Start,
    Message(String),
    Guess(BangumiSubject),
    Prepare,
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
    Start,
    CreateRoomOk,
    OJoinSucc(String),                   // other player's name
    JoinSucc(Vec<(String, PlayerData)>), // other players' name and data
    Response(String, String),            // username and message
    GuessResp(WsGuessResponse, usize),
    OGuessResp(String, usize), // another guy's resp and guess_time
    Over(
        Option<String>,
        HashMap<String, Vec<String>>,
        (BangumiSubject, CompareResult),
    ), // winner's name
    Prepare(String),           // player's name
    Reset,
    ResetOk,
    Leave(String), // username
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoomData {
    pub answer: BangumiSubject,
    pub max_guess: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct PlayerData {
    pub reset: bool,
    pub guess_time: usize,
    pub is_prepared: bool,
    pub guesses: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum RoomState {
    Waiting,
    Playing,
}

#[derive(Clone)]
pub struct Room {
    pub state: RoomState,
    pub name: String,
    pub host: String, // host's id
    pub players: Vec<(Player, PlayerData)>,
    pub data: Option<RoomData>,
}

impl Room {
    pub fn reset(&mut self) {
        self.state = RoomState::Waiting;

        self.data = None;

        for (_, data) in &mut self.players {
            data.reset = false;
            data.guess_time = 0;
            data.is_prepared = false;
            data.guesses.clear();
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct RoomInfo {
    pub state: RoomState,
    pub name: String,
    pub player_num: usize,
    pub id: String,
}

#[derive(Clone)]
pub struct Player {
    pub id: String,
    pub name: String,
    pub session: Session,
}

pub async fn get_rooms(
    _req: HttpRequest,
    data: web::Data<MultiState>,
) -> actix_web::Result<impl Responder> {
    let rooms = data.rooms.lock().unwrap();

    let room_list: Vec<RoomInfo> = rooms
        .iter()
        .map(|(id, room)| RoomInfo {
            id: id.clone(),
            state: room.state.clone(),
            name: room.name.clone(),
            player_num: room.players.len(),
        })
        .collect();

    Ok(HttpResponse::Ok().json(room_list))
}

const MAX_GUESS: usize = 20;
const MAX_ROOM: usize = 100;
const MAX_PLAYER: usize = 10;

pub async fn ws(
    req: HttpRequest,
    body: web::Payload,
    data: web::Data<MultiState>,
) -> actix_web::Result<impl Responder> {
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, body)?;
    let state = data.get_ref().clone();

    actix_web::rt::spawn(async move {
        let current_user_id = format!("user-{}", Uuid::new_v4());

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
                Message::Pong(_) => {}
                Message::Close(_) => break,
                Message::Binary(_) | Message::Continuation(_) | Message::Nop => {}
                Message::Text(msg) => {
                    // rate limiting
                    let now = Instant::now();
                    let elapsed = now.duration_since(last_tick).as_secs_f64();
                    last_tick = now;
                    tokens = (tokens + elapsed * refill_rate).min(max_tokens);
                    if tokens < 1.0 {
                        println!(
                            "Rate limit exceeded for user: {}. Closing connection.",
                            current_user_id
                        );
                        break;
                    }
                    tokens -= 1.0;

                    let Ok(client_msg) = serde_json::from_str::<ClientMsg>(&msg) else {
                        continue;
                    };
                    match client_msg {
                        ClientMsg::CreateRoom(room_name, name) => {
                            let user_id = current_user_id.clone();
                            let player = Player {
                                id: user_id.clone(),
                                name: name.clone(),
                                session: session.clone(),
                            };
                            let room = Room {
                                name: room_name.clone(),
                                state: RoomState::Waiting,
                                host: user_id.clone(),
                                players: vec![(player, PlayerData::default())],
                                data: None,
                            };
                            let mut rooms = state.rooms.lock().unwrap();
                            let room_id = Uuid::new_v4().to_string();
                            if rooms.len() < MAX_ROOM {
                                rooms.insert(room_id.clone(), room);
                                state.user_room.lock().unwrap().insert(user_id, room_id);
                            }
                            println!("{} create a room: {}.", name, room_name);
                            let msg_str = serde_json::to_string(&ServerMsg::CreateRoomOk).unwrap();
                            let _ = session.text(msg_str.clone()).await;
                        }

                        ClientMsg::Join(room_id, name) => {
                            let user_id = current_user_id.clone();

                            let result = {
                                let mut rooms = state.rooms.lock().unwrap();

                                if let Some(room) = rooms.get_mut(&room_id) {
                                    if room.players.len() >= MAX_PLAYER
                                        || room.players.iter().any(|p| p.0.name == name)
                                    {
                                        None
                                    } else {
                                        let old_players: Vec<(String, PlayerData)> = room
                                            .players
                                            .iter()
                                            .map(|(p, d)| (p.name.clone(), d.clone()))
                                            .collect();

                                        let new_player = Player {
                                            id: user_id.clone(),
                                            name: name.clone(),
                                            session: session.clone(),
                                        };

                                        room.players.push((new_player, PlayerData::default()));

                                        state
                                            .user_room
                                            .lock()
                                            .unwrap()
                                            .insert(user_id.clone(), room_id.clone());

                                        let other_sessions: Vec<_> = room
                                            .players
                                            .iter()
                                            .filter(|p| p.0.id != user_id)
                                            .map(|p| p.0.session.clone())
                                            .collect();

                                        Some((old_players, other_sessions))
                                    }
                                } else {
                                    None
                                }
                            };

                            if let Some((old_players, others)) = result {
                                let msg_to_others =
                                    serde_json::to_string(&ServerMsg::OJoinSucc(name.clone()))
                                        .unwrap();

                                for mut s in others {
                                    let _ = s.text(msg_to_others.clone()).await;
                                }

                                let msg_to_me =
                                    serde_json::to_string(&ServerMsg::JoinSucc(old_players))
                                        .unwrap();

                                let _ = session.text(msg_to_me).await;
                            }
                        }

                        ClientMsg::Prepare => {
                            let uid = &current_user_id;
                            let sessions_and_name = {
                                let rid = state.user_room.lock().unwrap().get(uid).cloned();
                                if let Some(rid) = rid {
                                    let mut rooms = state.rooms.lock().unwrap();
                                    if let Some(room) = rooms.get_mut(&rid) {
                                        let mut name = String::new();
                                        for player in &mut room.players {
                                            if player.0.id == *uid {
                                                player.1.is_prepared = true;
                                                name = player.0.name.clone();
                                                break;
                                            }
                                        }
                                        Some((
                                            room.players
                                                .iter()
                                                .map(|p| p.0.session.clone())
                                                .collect::<Vec<_>>(),
                                            name,
                                        ))
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            };

                            if let Some((sessions, name)) = sessions_and_name {
                                let msg_str =
                                    serde_json::to_string(&ServerMsg::Prepare(name)).unwrap();
                                for mut s in sessions {
                                    let _ = s.text(msg_str.clone()).await;
                                }
                            }
                        }

                        ClientMsg::Start => {
                            let uid = &current_user_id;
                            let rid = state.user_room.lock().unwrap().get(uid).cloned();
                            if let Some(rid) = rid {
                                let is_ready = {
                                    let rooms = state.rooms.lock().unwrap();
                                    if let Some(room) = rooms.get(&rid) {
                                        !room.players.iter().any(|p| !p.1.is_prepared)
                                    } else {
                                        false
                                    }
                                };

                                if is_ready {
                                    if let Some(answer) = fetch_random_anime(1960, 2026).await {
                                        println!(
                                            "Generate answer: {} \n {}",
                                            answer.name, answer.name_cn
                                        );
                                        let sessions = {
                                            let mut rooms = state.rooms.lock().unwrap();
                                            if let Some(room) = rooms.get_mut(&rid) {
                                                if room.host == *uid {
                                                    room.state = RoomState::Playing;
                                                    let data = RoomData {
                                                        answer: answer,
                                                        max_guess: MAX_GUESS,
                                                    };
                                                    room.data = Some(data);
                                                    Some(
                                                        room.players
                                                            .iter()
                                                            .map(|p| p.0.session.clone())
                                                            .collect::<Vec<_>>(),
                                                    )
                                                } else {
                                                    None
                                                }
                                            } else {
                                                None
                                            }
                                        };

                                        if let Some(sessions) = sessions {
                                            let msg_str =
                                                serde_json::to_string(&ServerMsg::Start).unwrap();
                                            for mut s in sessions {
                                                let _ = s.text(msg_str.clone()).await;
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        ClientMsg::Message(m) => {
                            let uid = &current_user_id;
                            let (target_sessions, name) = {
                                let rid = state.user_room.lock().unwrap().get(uid).cloned();

                                if let Some(rid) = rid {
                                    let rooms = state.rooms.lock().unwrap();

                                    if let Some(room) = rooms.get(&rid) {
                                        let target_sessions = room
                                            .players
                                            .iter()
                                            .filter(|p| p.0.id != *uid)
                                            .map(|p| p.0.session.clone())
                                            .collect::<Vec<_>>();

                                        let name = room
                                            .players
                                            .iter()
                                            .find(|p| p.0.id == *uid)
                                            .map(|p| p.0.name.clone())
                                            .unwrap_or_default();

                                        (Some(target_sessions), name)
                                    } else {
                                        (None, String::new())
                                    }
                                } else {
                                    (None, String::new())
                                }
                            };

                            if let Some(sessions) = target_sessions {
                                let msg_str =
                                    serde_json::to_string(&ServerMsg::Response(name, m)).unwrap();
                                for mut s in sessions {
                                    let _ = s.text(msg_str.clone()).await;
                                }
                            }
                        }

                        ClientMsg::Guess(guess) => {
                            let uid = &current_user_id;
                            let rid = {
                                let map = state.user_room.lock().unwrap();
                                map.get(uid).cloned()
                            };
                            let Some(rid) = rid else {
                                continue;
                            };

                            let (
                                mut sender_session,
                                other_sessions,
                                cur_gt,
                                answer,
                                max_guess,
                                name,
                            ) = {
                                let mut rooms = state.rooms.lock().unwrap();
                                let Some(room) = rooms.get_mut(&rid) else {
                                    continue;
                                };

                                if room.state != RoomState::Playing {
                                    continue;
                                }

                                let Some(data) = &room.data else {
                                    continue;
                                };
                                let max_guess = data.max_guess;
                                let answer = data.answer.clone();
                                let mut exceeded = false;
                                let mut sender_session = None;

                                for player in &mut room.players {
                                    if player.0.id == *uid {
                                        if player.1.guess_time >= max_guess {
                                            exceeded = true;
                                        } else {
                                            player.1.guess_time += 1;
                                            player.1.guesses.push(guess.name_cn.clone());
                                            sender_session = Some(player.0.session.clone());
                                        }
                                    }
                                }

                                if exceeded || sender_session.is_none() {
                                    continue;
                                }

                                let current_player =
                                    room.players.iter().find(|p| p.0.id == *uid).unwrap();

                                let cur_gt = current_player.1.guess_time;

                                let name = current_player.0.name.clone();

                                let other_sessions: Vec<_> = room
                                    .players
                                    .iter()
                                    .filter(|p| p.0.id != *uid)
                                    .map(|p| p.0.session.clone())
                                    .collect();

                                (
                                    sender_session.unwrap(),
                                    other_sessions,
                                    cur_gt,
                                    answer,
                                    max_guess,
                                    name,
                                )
                            };

                            let is_correct = is_guess_right(&guess, &answer);
                            let comparison = compare_anime(&guess, &answer);
                            let right_comp = compare_anime(&answer, &answer);
                            //                            let comp_hide = get_hide_subject(&answer, &guess);

                            let is_draw = {
                                let rooms = state.rooms.lock().unwrap();
                                if let Some(room) = rooms.get(&rid) {
                                    !is_correct
                                        && room.players.iter().all(|p| p.1.guess_time >= max_guess)
                                } else {
                                    false
                                }
                            };

                            let mut all_guesses: HashMap<String, Vec<String>> = HashMap::new();
                            if is_correct || is_draw {
                                let mut rooms = state.rooms.lock().unwrap();
                                if let Some(room) = rooms.get_mut(&rid) {
                                    room.state = RoomState::Waiting;
                                    all_guesses = room
                                        .players
                                        .iter()
                                        .map(|p| (p.0.name.clone(), p.1.guesses.clone()))
                                        .collect();
                                }
                            }

                            let resp = WsGuessResponse {
                                guess,
                                comparison: comparison.clone(),
                            };

                            let _ = sender_session
                                .text(
                                    serde_json::to_string(&ServerMsg::GuessResp(resp, cur_gt))
                                        .unwrap(),
                                )
                                .await;

                            let target_msg = serde_json::to_string(&ServerMsg::OGuessResp(
                                name.to_string(),
                                //                                (comp_hide, cur_gt),
                                cur_gt,
                            ))
                            .unwrap();
                            for mut s in other_sessions.clone() {
                                let _ = s.text(target_msg.clone()).await;
                            }

                            if is_correct {
                                let _ = sender_session
                                    .text(
                                        serde_json::to_string(&ServerMsg::Over(
                                            Some(name.clone()),
                                            all_guesses.clone(),
                                            (answer.clone(), right_comp.clone()),
                                        ))
                                        .unwrap(),
                                    )
                                    .await;
                                let over_target_msg = serde_json::to_string(&ServerMsg::Over(
                                    Some(name),
                                    all_guesses,
                                    (answer.clone(), right_comp.clone()),
                                ))
                                .unwrap();
                                for mut s in other_sessions {
                                    let _ = s.text(over_target_msg.clone()).await;
                                }

                                // reset
                                {
                                    let mut rooms = state.rooms.lock().unwrap();

                                    if let Some(room) = rooms.get_mut(&rid) {
                                        room.reset();
                                    }
                                }
                            } else if is_draw {
                                let draw_msg = serde_json::to_string(&ServerMsg::Over(
                                    None,
                                    all_guesses,
                                    (answer.clone(), right_comp.clone()),
                                ))
                                .unwrap();
                                let _ = sender_session.text(draw_msg.clone()).await;
                                for mut s in other_sessions {
                                    let _ = s.text(draw_msg.clone()).await;
                                }

                                // reset
                                {
                                    let mut rooms = state.rooms.lock().unwrap();

                                    if let Some(room) = rooms.get_mut(&rid) {
                                        room.reset();
                                    }
                                }
                            }
                        }

                        ClientMsg::Reset => {
                            let uid = &current_user_id;

                            let rid = {
                                let map = state.user_room.lock().unwrap();
                                map.get(uid).cloned()
                            };

                            let Some(rid) = rid else {
                                continue;
                            };

                            let (should_restart, sessions) = {
                                let mut rooms = state.rooms.lock().unwrap();
                                let Some(room) = rooms.get_mut(&rid) else {
                                    continue;
                                };

                                let mut all_reset = true;
                                for player in &mut room.players {
                                    if player.0.id == *uid {
                                        player.1.reset = true;
                                    }
                                    if !player.1.reset {
                                        all_reset = false;
                                    }
                                }

                                let sessions: Vec<_> =
                                    room.players.iter().map(|p| p.0.session.clone()).collect();
                                (all_reset, sessions)
                            };

                            let _ = session
                                .text(serde_json::to_string(&ServerMsg::ResetOk).unwrap())
                                .await;

                            if should_restart {
                                if let Some(answer) = fetch_random_anime(1960, 2026).await {
                                    let mut rooms = state.rooms.lock().unwrap();
                                    if let Some(room) = rooms.get_mut(&rid) {
                                        if let Some(data) = &mut room.data {
                                            data.answer = answer.clone();
                                        }
                                        room.state = RoomState::Waiting;
                                        for player in &mut room.players {
                                            player.1.reset = false;
                                            player.1.guess_time = 0;
                                            player.1.is_prepared = false;
                                        }
                                    } else {
                                        continue;
                                    }

                                    let reset_msg =
                                        serde_json::to_string(&ServerMsg::Reset).unwrap();
                                    for mut s in sessions {
                                        let _ = s.text(reset_msg.clone()).await;
                                    }
                                }
                            }
                        }

                        ClientMsg::ILeave => break,
                    };
                }
            }
        }

        println!("Cleaning up user: {}", current_user_id);

        let rid = state.user_room.lock().unwrap().remove(&current_user_id);

        let mut sessions_to_notify = Vec::new();
        let mut leave_msg = None;

        if let Some(rid) = rid {
            let mut rooms = state.rooms.lock().unwrap();
            let mut room_empty = false;

            if let Some(room) = rooms.get_mut(&rid) {
                let leave_name = room
                    .players
                    .iter()
                    .find(|p| p.0.id == current_user_id)
                    .map(|p| p.0.name.clone())
                    .unwrap_or_else(|| "Unknown".to_string());

                room.players.retain(|p| p.0.id != current_user_id);

                room_empty = room.players.is_empty();

                if !room_empty {
                    leave_msg = Some(serde_json::to_string(&ServerMsg::Leave(leave_name)).unwrap());

                    sessions_to_notify = room.players.iter().map(|p| p.0.session.clone()).collect();
                }
            }

            if room_empty {
                rooms.remove(&rid);
            }
        }

        if let Some(msg) = leave_msg {
            for mut s in sessions_to_notify {
                let _ = s.text(msg.clone()).await;
            }
        }

        let _ = session.close(None).await;
    });

    Ok(response)
}
