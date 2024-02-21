use chat_room_common::model::{LoginRequest, LoginResponse, LogoutRequest, User};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use uuid::Uuid;
use warp::{http::StatusCode, reply::json, Rejection, Reply};

pub type Result<T> = std::result::Result<T, Rejection>;
pub type LoggedUsers = Arc<Mutex<HashMap<String, User>>>;

pub async fn login_handler(body: LoginRequest, users: LoggedUsers) -> Result<impl Reply> {
    let user_name = body.user_name;
    let token = Uuid::new_v4().simple().to_string();

    login_user(&user_name, &token, users).await;
    Ok(json(&LoginResponse {
        token: token.clone(),
    }))
}

pub async fn logout_handler(body: LogoutRequest, logged_users: LoggedUsers) -> Result<impl Reply> {
    let user_name = body.user_name;
    let token = body.token;

    let mut users_lock = logged_users.lock().unwrap();
    match users_lock.get(&user_name) {
        Some(user) => {
            if user.token == token {
                users_lock.remove(&user_name);
                println!("User {} logged out", user_name);
                return Ok(StatusCode::OK);
            } else {
                println!("Wrong token for user {}", user_name);
                Ok(StatusCode::UNAUTHORIZED)
            }
        }
        None => {
            println!("User {} was not logged in", user_name);
            Ok(StatusCode::OK)
        }
    }
}
async fn login_user(user_name: &str, token: &str, logged_users: LoggedUsers) {
    let mut users_lock = logged_users.lock().unwrap();
    match users_lock.get(user_name) {
        Some(_) => {
            println!("User {} already logged in", user_name);
        }
        None => {
            let user = User {
                user_name: user_name.to_string(),
                token: token.to_string(),
                sender: None,
            };
            users_lock.insert(user_name.to_string(), user);
            println!("User {} logged in", user_name);
        }
    }
}
