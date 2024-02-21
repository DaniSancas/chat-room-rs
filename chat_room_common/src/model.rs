use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use warp::ws::Message;

pub struct User {
    pub user_name: String,
    pub token: String,
    pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>,
}

#[derive(Deserialize, Serialize)]
pub struct LoginRequest {
    pub user_name: String,
}

#[derive(Serialize, Debug)]
pub struct LoginResponse {
    pub token: String,
}

#[derive(Deserialize, Serialize)]
pub struct LogoutRequest {
    pub user_name: String,
    pub token: String,
}
