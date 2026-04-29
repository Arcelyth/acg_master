use gloo_net::websocket::{futures::WebSocket, Message};
use futures::{SinkExt, StreamExt};
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use web_sys::window;

use crate::bangumi::anime::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ClientMsg {
    Join(String), // name
    Message(String),
    Guess(BangumiSubject),
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
    OGuessResp(CompareResult), // another guy's resp
    Over(bool, (BangumiSubject, CompareResult)), 
}

pub fn connect_ws(
    on_message: impl Fn(String) + 'static + Clone,
) -> futures::channel::mpsc::UnboundedSender<Message> {

    let url = if cfg!(debug_assertions) {
        format!("ws://localhost:8060/api/bangumi/anime/ws")
    } else {
        let origin = window().unwrap().location().origin().unwrap();
        let ws_url = origin.replace("http", "ws");
        format!("{}/api/bangumi/anime/ws", ws_url)
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
