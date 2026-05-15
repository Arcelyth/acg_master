use futures::{SinkExt, StreamExt};
use gloo_net::websocket::{Message, futures::WebSocket};
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use web_sys::window;
use reqwest::Client;

use crate::bangumi::anime::*;

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

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum RoomState {
    Waiting,
    Playing,
    Finished,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WsGuessResponse {
    pub guess: BangumiSubject,
    pub comparison: CompareResult,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ServerMsg {
    Start,
    JoinSucc(String), // player's name
    Response(String),
    GuessResp(WsGuessResponse, usize),
    OGuessResp(BangumiSubjectHide), // another guy's resp
    Over(bool, (BangumiSubject, CompareResult)),
    Prepare(String), // player's name
    Reset,
    ResetOk,
    Leave(BangumiSubject, CompareResult), // opponent leave
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct RoomInfo {
    pub state: RoomState,
    pub name: String,
    pub player_num: usize,
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
        format!("http://localhost:8060/api/bangumi/anime/get_rooms")
    } else {
        let origin = window().unwrap().location().origin().unwrap();
        format!("{}/api/bangumi/anime/get_rooms", origin)
    };

    let res = client
        .get(url)
        .fetch_credentials_include()
        .send()
        .await;
    if let Ok(response) = res {
        if response.status() == 200 {
            if let Ok(result) = response.json().await {
                return result;
            }
        }
    }
    vec![]
}


