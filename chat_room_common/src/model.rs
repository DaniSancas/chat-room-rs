use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use tokio::sync::{mpsc, RwLock};
use warp::ws::Message;

pub struct User {
    pub user_name: String,
    pub token: String,
    pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>,
}

pub struct Room {
    pub users: HashSet<String>,
}

pub type UserMap = HashMap<String, User>;
pub type LoggedUsers = Arc<RwLock<UserMap>>;
pub type RoomMap = HashMap<String, Room>;
pub type Rooms = Arc<RwLock<RoomMap>>;
