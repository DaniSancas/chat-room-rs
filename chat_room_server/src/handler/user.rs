use super::Result;
use crate::{handler::room::remove_user_from_all_rooms, helper::*};
use chat_room_common::model::{LoggedUsers, Rooms, SenderMap, User};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use warp::{http::StatusCode, reply::json, Reply};

#[derive(Deserialize, Serialize)]
pub struct LoginRequest {
    pub user_name: String,
}

#[derive(Serialize, Debug)]
pub struct LoginResponse {
    pub token: String,
}

#[derive(Deserialize, Serialize)]
pub struct AuthRequest {
    pub user_name: String,
    pub token: String,
}

#[derive(Deserialize, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

pub async fn login_handler(body: LoginRequest, logged_users: LoggedUsers) -> Result<impl Reply> {
    let user_name = body.user_name;
    let token = Uuid::new_v4().simple().to_string();

    let mut users_lock = logged_users.write().await;
    match users_lock.get(&user_name) {
        Some(_) => {
            log_user_already_logged_in(&user_name);
            Ok(json(&ErrorResponse {
                error: "User already logged in".to_string(),
            }))
        }
        None => {
            let user = User {
                user_name: user_name.to_string(),
                token: token.to_string(),
                sender: SenderMap::new(),
            };
            users_lock.insert(user_name.to_string(), user);
            log_user_logged_in(&user_name);
            Ok(json(&LoginResponse {
                token: token.clone(),
            }))
        }
    }
}

pub async fn logout_handler(
    body: AuthRequest,
    logged_users: LoggedUsers,
    rooms: Rooms,
) -> Result<impl Reply> {
    let user_name = body.user_name;
    let token = body.token;

    let mut users_lock = logged_users.write().await;
    match users_lock.get(&user_name) {
        Some(user) => {
            if user.token == token {
                // Remove user from rooms
                remove_user_from_all_rooms(&user_name, rooms).await;
                log_all_rooms_left(&user_name);

                // Remove user from logged users
                users_lock.remove(&user_name);
                log_user_logged_out(&user_name);
                Ok(StatusCode::OK)
            } else {
                log_user_not_authorized(&user_name);
                Ok(StatusCode::UNAUTHORIZED)
            }
        }
        None => {
            log_user_not_logged_in(&user_name);
            Ok(StatusCode::OK)
        }
    }
}
