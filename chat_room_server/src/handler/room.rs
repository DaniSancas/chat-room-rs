use super::Result;
use chat_room_common::model::{LoggedUsers, Room, Rooms};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use warp::{http::StatusCode, Reply};

#[derive(Deserialize, Serialize)]
pub struct RoomJoinRequest {
    pub user_name: String,
    pub token: String,
    pub room_name: String,
}

pub async fn join_room_handler(
    body: RoomJoinRequest,
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
                            info!("User {} already in room {}", user_name, room_name);
                        } else {
                            room.users.push(user_name.clone());
                            info!("User {} joined room {}", user_name, room_name);
                        }
                        Ok(StatusCode::OK)
                    }
                    None => {
                        let room = Room {
                            users: vec![user_name.clone()],
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
