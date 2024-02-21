use std::collections::HashSet;

use super::Result;
use chat_room_common::model::{LoggedUsers, Room, Rooms};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use warp::{http::StatusCode, Reply};

#[derive(Deserialize, Serialize)]
pub struct JoinOrLeaveRequest {
    pub user_name: String,
    pub token: String,
    pub room_name: String,
}

pub async fn join_handler(
    body: JoinOrLeaveRequest,
    logged_users: LoggedUsers,
    rooms: Rooms,
) -> Result<impl Reply> {
    let user_name = body.user_name;
    let token = body.token;
    let room_name = body.room_name;

    let logged_users_lock = logged_users.read().await;

    match logged_users_lock.get(&user_name) {
        Some(user) => {
            if user.token == token {
                let mut rooms_lock = rooms.write().await;
                match rooms_lock.get_mut(&room_name) {
                    Some(room) => {
                        if room.users.contains(&user_name) {
                            warn!("User {} already joined room {}", user_name, room_name);
                        } else {
                            room.users.insert(user_name.clone());
                            info!("User {} joined room {}", user_name, room_name);
                        }
                        Ok(StatusCode::OK)
                    }
                    None => {
                        let room = Room {
                            users: HashSet::from([user_name.clone()]),
                        };
                        rooms_lock.insert(room_name.clone(), room);
                        info!("User {} created room {}", user_name, room_name);
                        Ok(StatusCode::OK)
                    }
                }
            } else {
                warn!("Wrong token for user {}", user_name);
                Ok(StatusCode::UNAUTHORIZED)
            }
        }
        None => {
            warn!("User {} was not logged in", user_name);
            Ok(StatusCode::UNAUTHORIZED)
        }
    }
}

pub async fn leave_handler(
    body: JoinOrLeaveRequest,
    logged_users: LoggedUsers,
    rooms: Rooms,
) -> Result<impl Reply> {
    let user_name = body.user_name;
    let token = body.token;
    let room_name = body.room_name;

    let logged_users_lock = logged_users.read().await;
    if let Some(user) = logged_users_lock.get(&user_name) {
        if user.token == token {
            remove_user_from_single_room(&user_name, &room_name, rooms).await;
            Ok(StatusCode::OK)
        } else {
            warn!("Wrong token for user {}", user_name);
            Ok(StatusCode::UNAUTHORIZED)
        }
    } else {
        warn!("User {} was not logged in", user_name);
        Ok(StatusCode::UNAUTHORIZED)
    }
}

pub async fn remove_user_from_all_rooms(user_name: &str, rooms: Rooms) {
    for (room_name, room) in rooms.write().await.iter_mut() {
        remove_user(room, user_name, room_name);
    }
}

pub async fn remove_user_from_single_room(user_name: &str, room_name: &str, rooms: Rooms) {
    if let Some(room) = rooms.write().await.get_mut(room_name) {
        remove_user(room, user_name, room_name);
    } else {
        warn!("Room {} does not exist", room_name);
    }
}

fn remove_user(room: &mut Room, user_name: &str, room_name: &str) {
    if room.users.contains(user_name) {
        room.users.remove(user_name);
        info!("User {} left room {}", user_name, room_name);
    }
}
