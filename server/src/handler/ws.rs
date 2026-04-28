use actix_web::{HttpRequest, Responder, web};
use actix_ws::Message;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ClientMsg {
    Join,
    Message(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ServerMsg{
    JoinSucc,
    Response(String),
}


pub async fn ws(req: HttpRequest, body: web::Payload) -> actix_web::Result<impl Responder> {
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, body)?;
    actix_web::rt::spawn(async move {
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
                    let reply = match client_msg {
                        ClientMsg::Join => {
                            println!("Someone Join");
                            ServerMsg::JoinSucc
                        }
                        ClientMsg::Message(m) => {
                            ServerMsg::Response(format!("echo: {}", m))
                        }
                    };

                    if let Ok(text) = serde_json::to_string(&reply) {
                        // reply
                        if session.text(text).await.is_err() {
                            return;
                        }
                    }
                }
                _ => break,
            }
        }

        let _ = session.close(None).await;
    });

    Ok(response)
}
