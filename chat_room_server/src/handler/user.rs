use super::Result;
use chat_room_common::model::{LoggedUsers, Rooms, User};
use log::{info, warn};
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
pub struct LogoutRequest {
    pub user_name: String,
    pub token: String,
}

pub async fn login_handler(body: LoginRequest, users: LoggedUsers) -> Result<impl Reply> {
    let user_name = body.user_name;
    let token = Uuid::new_v4().simple().to_string();

    login_user(&user_name, &token, users).await;
    Ok(json(&LoginResponse {
        token: token.clone(),
    }))
}

pub async fn logout_handler(
    body: LogoutRequest,
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
                let mut rooms_lock = rooms.write().await;
                for room in rooms_lock.values_mut() {
                    room.users.retain(|u| u != &user_name);
                }
                info!("User {} left all rooms", user_name);

                // Remove user from logged users
                users_lock.remove(&user_name);
                info!("User {} logged out", user_name);
                return Ok(StatusCode::OK);
            } else {
                warn!("Wrong token for user {}", user_name);
                Ok(StatusCode::UNAUTHORIZED)
            }
        }
        None => {
            warn!("User {} was not logged in", user_name);
            Ok(StatusCode::OK)
        }
    }
}

async fn login_user(user_name: &str, token: &str, logged_users: LoggedUsers) {
    let mut users_lock = logged_users.write().await;
    match users_lock.get(user_name) {
        Some(_) => {
            info!("User {} already logged in", user_name);
        }
        None => {
            let user = User {
                user_name: user_name.to_string(),
                token: token.to_string(),
                sender: None,
            };
            users_lock.insert(user_name.to_string(), user);
            info!("User {} logged in", user_name);
        }
    }
}
