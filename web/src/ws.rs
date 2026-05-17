use std::collections::HashMap;

use futures::{SinkExt, StreamExt};
use gloo_net::websocket::{Message, futures::WebSocket};
use leptos::task::spawn_local;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use web_sys::window;

use crate::bangumi::anime::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ClientMsg {
    Join(String, String),       // room_id and username
    CreateRoom(String, String), // room name and creator's name
    Start(MultiConfig),
    Message(String),
    Guess(BangumiSubject),
    Prepare,
    Reset,
    ILeave, // sender leave
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct MultiConfig {
    pub max_guess: usize,
    pub start_year: usize,
    pub end_year: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum RoomState {
    Waiting,
    Playing,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WsGuessResponse {
    pub guess: BangumiSubject,
    pub comparison: CompareResult,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct PlayerData {
    pub reset: bool,
    pub guess_time: usize,
    pub is_prepared: bool,
    pub points: usize,
    pub is_host: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ServerMsg {
    Start(MultiConfig),
    CreateRoomOk,
    JoinSucc(Vec<(String, PlayerData)>), // other players' name and data
    OJoinSucc(String),                   // other player's name
    Response(String, String),
    GuessResp(WsGuessResponse, usize),
    //    OGuessResp(String, (BangumiSubjectHide, usize)), // another guy's resp
    OGuessResp(String, usize), // another guy's resp
    Over(
        Option<String>,
        HashMap<String, Vec<String>>,
        (BangumiSubject, CompareResult),
    ), // winner's name

    Prepare(String), // player's name
    Reset,
    ResetOk,
    Leave(String), // opponent leave
    ErrMsg(ErrType),
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct RoomInfo {
    pub id: String,
    pub state: RoomState,
    pub name: String,
    pub player_num: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ErrType {
    None,
    DupName,
    InvalidNameLen,
    InvalidRoomNameLen,
}

pub fn connect_ws(
    on_message: impl Fn(String) + 'static + Clone,
) -> futures::channel::mpsc::UnboundedSender<Message> {
    let url = if cfg!(debug_assertions) {
        format!("ws://localhost:8060/api/bangumi/anime/ws")
    } else {
        let loc = window().unwrap().location();
        let protocol = loc.protocol().unwrap();
        let host = loc.host().unwrap();

        let ws_protocol = if protocol == "https:" { "wss" } else { "ws" };

        format!("{}://{}/api/bangumi/anime/ws", ws_protocol, host)
    };

    let ws = WebSocket::open(&url).unwrap();
    let (mut write, mut read) = ws.split();

    let (tx, mut rx) = futures::channel::mpsc::unbounded::<Message>();

    spawn_local(async move {
        while let Some(msg) = rx.next().await {
            let _ = write.send(msg).await;
        }
    });

    spawn_local(async move {
        while let Some(msg) = read.next().await {
            if let Ok(Message::Text(text)) = msg {
                on_message(text);
            }
        }
    });

    tx
}

pub async fn get_rooms() -> Vec<RoomInfo> {
    let client = Client::new();

    let url = if cfg!(debug_assertions) {
        format!("http://localhost:8060/api/bangumi/anime/rooms")
    } else {
        let origin = window().unwrap().location().origin().unwrap();
        format!("{}/api/bangumi/anime/rooms", origin)
    };

    let res = client.get(url).fetch_credentials_include().send().await;
    if let Ok(response) = res {
        if response.status() == 200 {
            if let Ok(result) = response.json().await {
                return result;
            }
        }
    }
    vec![]
}

#[derive(Serialize, Deserialize)]
pub struct CreateRoomReq {
    pub room_name: String,
    pub user_name: String,
}

#[derive(Serialize, Deserialize)]
pub struct CreateRoomRes {
    pub room_id: String,
}

pub async fn create_a_room(room_name: String, user_name: String) -> bool {
    let client = Client::new();

    let url = if cfg!(debug_assertions) {
        format!("http://localhost:8060/api/bangumi/anime/create_room")
    } else {
        let origin = window().unwrap().location().origin().unwrap();
        format!("{}/api/bangumi/anime/create_room", origin)
    };
    let req = CreateRoomReq {
        room_name,
        user_name,
    };

    let res = client
        .post(url)
        .fetch_credentials_include()
        .json(&req)
        .send()
        .await;
    if let Ok(response) = res {
        if response.status() == 200 {
            true
        } else {
            false
        }
    } else {
        false
    }
}
