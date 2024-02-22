use std::collections::HashSet;

use crate::helper::*;

use super::Result;
use chat_room_common::model::{LoggedUsers, Room, Rooms};
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
                            log_user_already_joined_room(&room_name, &user_name);
                        } else {
                            room.users.insert(user_name.clone());
                            log_room_joined(&room_name, &user_name);
                        }
                        Ok(StatusCode::OK)
                    }
                    None => {
                        let room = Room {
                            users: HashSet::from([user_name.clone()]),
                        };
                        rooms_lock.insert(room_name.clone(), room);
                        log_user_created_room(&user_name, &room_name);
                        Ok(StatusCode::OK)
                    }
                }
            } else {
                log_user_not_authorized(&user_name);
                Ok(StatusCode::UNAUTHORIZED)
            }
        }
        None => {
            log_user_not_logged_in(&user_name);
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
            log_user_not_authorized(&user_name);
            Ok(StatusCode::UNAUTHORIZED)
        }
    } else {
        log_user_not_logged_in(&user_name);
        Ok(StatusCode::UNAUTHORIZED)
    }
}

pub async fn remove_user_from_all_rooms(user_name: &str, rooms: Rooms) {
    let mut rooms_lock = rooms.write().await;
    let mut rooms_to_remove: Vec<String> = vec![];
    // First, remove the user from all rooms
    for (room_name, room) in rooms_lock.iter_mut() {
        remove_user(room, user_name, room_name);
        if room.users.is_empty() {
            rooms_to_remove.push(room_name.clone());
        }
    }
    // Then, remove the rooms that are empty
    for room_name in rooms_to_remove {
        rooms_lock.remove(&room_name);
        log_room_removed(&room_name);
    }
}

pub async fn remove_user_from_single_room(user_name: &str, room_name: &str, rooms: Rooms) {
    let mut rooms_lock = rooms.write().await;
    if let Some(room) = rooms_lock.get_mut(room_name) {
        remove_user(room, user_name, room_name);
    } else {
        log_room_does_not_exist(room_name);
    }
    if rooms_lock.get(room_name).unwrap().users.is_empty() {
        rooms_lock.remove(room_name);
        log_room_removed(room_name);
    }
}

fn remove_user(room: &mut Room, user_name: &str, room_name: &str) {
    if room.users.contains(user_name) {
        room.users.remove(user_name);
        log_room_left(room_name, user_name);
    }
}
