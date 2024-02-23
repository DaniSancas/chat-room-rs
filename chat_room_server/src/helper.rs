use log::{error, info, warn};

pub fn log_user_not_authorized(user_name: &str) {
    warn!("Wrong token for user {}", user_name);
}

pub fn log_user_not_logged_in(user_name: &str) {
    warn!("User {} was not logged in", user_name);
}

pub fn log_user_logged_in(user_name: &str) {
    info!("User {} logged in", user_name);
}

pub fn log_user_logged_out(user_name: &str) {
    info!("User {} logged out", user_name);
}

pub fn log_all_rooms_left(user_name: &str) {
    info!("User {} left all rooms", user_name);
}

pub fn log_user_already_logged_in(user_name: &str) {
    warn!("User {} already logged in", user_name);
}

pub fn log_room_left(room_name: &str, user_name: &str) {
    info!("User {} left room {}", user_name, room_name);
}

pub fn log_room_removed(room_name: &str) {
    info!("Room {} removed", room_name);
}

pub fn log_room_does_not_exist(room_name: &str) {
    warn!("Room {} does not exist", room_name);
}

pub fn log_user_already_joined_room(room_name: &str, user_name: &str) {
    warn!("User {} already joined room {}", user_name, room_name);
}

pub fn log_room_joined(room_name: &str, user_name: &str) {
    info!("User {} joined room {}", user_name, room_name);
}

pub fn log_user_created_room(user_name: &str, room_name: &str) {
    info!("User {} created room {}", user_name, room_name);
}

pub fn log_user_connected_to_ws(user_name: &str) {
    info!("User {} connected to websocket", user_name);
}

pub fn log_user_disconnected_from_ws(user_name: &str) {
    info!("User {} disconnected from websocket", user_name);
}

pub fn log_error_receiving_ws_message(user_name: &str, e: &str) {
    error!(
        "Error receiving websocket message for user {}: {}",
        user_name, e
    );
}

pub fn log_error_sending_ws_message(e: &str) {
    error!("Error sending websocket message: {}", e);
}

pub fn log_received_ws_message_not_str(msg: &warp::ws::Message) {
    warn!("Received websocket message that is not a string: {:?}", msg);
}

pub fn log_initiating_ws_connection() {
    info!("Initiating websocket connection...");
}

pub fn log_closing_ws_connection(user_name: &str) {
    info!("Closing websocket connection for user {}...", user_name);
}
pub fn log_error_parsing_incoming_message(e: &str) {
    error!("Error parsing incoming message: {}", e);
}
