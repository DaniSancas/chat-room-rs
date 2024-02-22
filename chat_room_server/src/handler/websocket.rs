use crate::helper::*;

use super::Result;
use chat_room_common::model::{LoggedUsers, Rooms};
use futures::{FutureExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use warp::ws::{Message, WebSocket};
use warp::Reply;

#[derive(Deserialize, Serialize)]
pub struct IncomingMessage {
    pub room_name: String,
    pub message: String,
}

#[derive(Deserialize, Serialize)]
pub struct OutgoingMessage {
    pub user_name: String,
    pub room_name: String,
    pub message: String,
}

#[derive(Deserialize, Serialize)]
pub struct OutgoingCommunication {
    pub communication_type: String,
    pub content: OutgoingMessage,
}

pub async fn streaming_handler(
    ws: warp::ws::Ws,
    user_name: String,
    token: String,
    logged_users: LoggedUsers,
    rooms: Rooms,
) -> Result<impl Reply> {
    log_initiating_ws_connection(&user_name);
    let is_user_valid = match logged_users.read().await.get(&user_name) {
        Some(user) => {
            if user.token == token {
                true
            } else {
                log_user_not_authorized(&user_name);
                false
            }
        }
        None => {
            log_user_not_logged_in(&user_name);
            false
        }
    };

    if is_user_valid {
        Ok(ws.on_upgrade(move |socket| {
            user_connection(
                socket,
                user_name.clone(),
                logged_users.clone(),
                rooms.clone(),
            )
        }))
    } else {
        Err(warp::reject::not_found())
    }
}

pub async fn user_connection(
    ws: WebSocket,
    user_name: String,
    logged_users: LoggedUsers,
    rooms: Rooms,
) {
    let (user_ws_sender, mut user_ws_rcv) = ws.split();
    let (user_channel_sender, client_rcv) = mpsc::unbounded_channel();
    let client_channel_rcv = UnboundedReceiverStream::new(client_rcv);

    tokio::task::spawn(client_channel_rcv.forward(user_ws_sender).map(|result| {
        if let Err(e) = result {
            log_error_sending_ws_message(e.to_string().as_str());
        }
    }));

    // Initialize user's channel
    if let Some(user) = logged_users.write().await.get_mut(&user_name) {
        user.sender = Some(user_channel_sender);
        log_user_connected_to_ws(&user_name);
    }

    // Listen for messages from the user
    while let Some(result) = user_ws_rcv.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                log_error_receiving_ws_message(&user_name, e.to_string().as_str());
                break;
            }
        };
        send_message_to_the_room(&user_name, &msg, &logged_users, &rooms).await;
    }

    // User disconnected
    if let Some(user) = logged_users.write().await.get_mut(&user_name) {
        user.sender = None;
        log_user_disconnected_from_ws(&user_name);
    }
}

async fn send_message_to_the_room(
    user_name: &str,
    msg: &Message,
    logged_users: &LoggedUsers,
    rooms: &Rooms,
) {
    let message = if msg.is_text() {
        msg.to_str().unwrap()
    } else if msg.is_close() {
        log_closing_ws_connection(user_name);
        return;
    } else {
        log_received_ws_message_not_str(msg);
        return;
    };

    let incoming_msg: IncomingMessage = match from_str(message) {
        Ok(v) => v,
        Err(e) => {
            log_error_parsing_incoming_message(e.to_string().as_str());
            return;
        }
    };

    // Send message to all users in the room
    if let Some(room) = rooms.read().await.get(&incoming_msg.room_name) {
        let logged_users_lock = logged_users.read().await;
        room.users.iter().for_each(|logged_user| {
            if logged_user != user_name {
                // For each user in the room that is not the sender, send the message to their sender channel
                if let Some(user) = logged_users_lock.get(logged_user) {
                    if let Some(sender) = &user.sender {
                        let outgoing_communication = OutgoingCommunication {
                            communication_type: "message".to_string(),
                            content: OutgoingMessage {
                                user_name: user_name.to_string(),
                                room_name: incoming_msg.room_name.clone(),
                                message: incoming_msg.message.clone(),
                            },
                        };
                        if let Err(e) = sender.send(Ok(Message::text(
                            serde_json::to_string(&outgoing_communication).unwrap(),
                        ))) {
                            log_error_sending_ws_message(e.to_string().as_str());
                        }
                    }
                }
            }
        });
    } else {
        log_room_does_not_exist(&incoming_msg.room_name);
    }
}
