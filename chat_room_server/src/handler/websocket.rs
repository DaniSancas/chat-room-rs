use crate::helper::*;

use super::Result;
use crate::handler::user::AuthRequest;
use chat_room_common::model::{LoggedUsers, Rooms};
use futures::stream::SplitStream;
use futures::{FutureExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use uuid::Uuid;
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
    pub content: CommunicationContent,
}

#[derive(Deserialize, Serialize)]
pub enum CommunicationContent {
    OutgoingMessage(OutgoingMessage),
}

pub async fn streaming_handler(
    ws: warp::ws::Ws,
    logged_users: LoggedUsers,
    rooms: Rooms,
) -> Result<impl Reply> {
    log_initiating_ws_connection();

    Ok(ws.on_upgrade(move |socket| user_connection(socket, logged_users.clone(), rooms.clone())))
}

async fn get_username_from_first_message(
    user_ws_rcv: &mut SplitStream<WebSocket>,
    logged_users: &LoggedUsers,
) -> Option<String> {
    let logged_users_lock = logged_users.read().await;

    user_ws_rcv
        .next()
        .await
        .and_then(|first_message| match first_message {
            Ok(msg) => Some(msg),
            Err(e) => {
                log_error_receiving_ws_message("unknown", e.to_string().as_str());
                None
            }
        })
        .and_then(|msg| match msg.to_str() {
            Ok(text) => Some(text.to_string()),
            Err(_) => {
                log_received_ws_message_not_str(&msg);
                None
            }
        })
        .and_then(|message| match from_str::<AuthRequest>(message.as_str()) {
            Ok(auth) => Some(auth),
            Err(e) => {
                log_error_parsing_incoming_message(e.to_string().as_str());
                None
            }
        })
        .and_then(|auth| match logged_users_lock.get(&auth.user_name) {
            Some(user) => {
                if user.token == auth.token {
                    Some(auth.user_name)
                } else {
                    log_user_not_authorized(&auth.user_name);
                    None
                }
            }
            None => {
                log_user_not_logged_in(&auth.user_name);
                None
            }
        })
}

pub async fn user_connection(ws: WebSocket, logged_users: LoggedUsers, rooms: Rooms) {
    // Split the WebSocket into a sender and receive of messages
    let (user_ws_sender, mut user_ws_rcv) = ws.split();

    let option_user_name = get_username_from_first_message(&mut user_ws_rcv, &logged_users).await;

    let user_name = match option_user_name {
        Some(user_name) => user_name,
        None => {
            log_error_receiving_ws_message("unknown", "no message received");
            return;
        }
    };

    let (user_channel_sender, client_rcv) = mpsc::unbounded_channel();
    let client_channel_rcv = UnboundedReceiverStream::new(client_rcv);

    tokio::task::spawn(client_channel_rcv.forward(user_ws_sender).map(|result| {
        if let Err(e) = result {
            log_error_sending_ws_message(e.to_string().as_str());
        }
    }));

    // Generate a unique identifier for the user's sender channel
    // This is used to remove the sender channel when the user disconnects
    let sender_uuid = Uuid::new_v4().simple().to_string();

    // Initialize user's channel
    if let Some(user) = logged_users.write().await.get_mut(&user_name) {
        user.sender.insert(sender_uuid.clone(), user_channel_sender);
        log_user_connected_to_ws(&user_name);
        log_current_user_connections(&user_name, user.sender.len());
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
    // Remove the sender channel related to this websocket connection
    if let Some(user) = logged_users.write().await.get_mut(&user_name) {
        user.sender.remove(&sender_uuid);
        log_user_disconnected_from_ws(&user_name);
        log_remaining_connections_for_user(&user_name, user.sender.len());
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
        if room.users.contains(user_name) {
            // Only send the message if the user is in the room
            let logged_users_lock = logged_users.read().await;
            room.users.iter().for_each(|logged_user| {
                // For each user in the room,, get all sender channels it has and send the message (including the sender's own channels)
                if let Some(user) = logged_users_lock.get(logged_user) {
                    user.sender.values().for_each(|sender| {
                        let outgoing_communication = OutgoingCommunication {
                            communication_type: "message".to_string(),
                            content: CommunicationContent::OutgoingMessage(OutgoingMessage {
                                user_name: user_name.to_string(),
                                room_name: incoming_msg.room_name.clone(),
                                message: incoming_msg.message.clone(),
                            }),
                        };
                        if let Err(e) = sender.send(Ok(Message::text(
                            serde_json::to_string(&outgoing_communication).unwrap(),
                        ))) {
                            log_error_sending_ws_message(e.to_string().as_str());
                        }
                    });
                }
            });
        } else {
            log_user_not_joined_room(&incoming_msg.room_name, user_name);
        }
    } else {
        log_room_does_not_exist(&incoming_msg.room_name);
    }
}
