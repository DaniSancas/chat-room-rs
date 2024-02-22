mod handler;
mod helper;

use handler::websocket::streaming_handler;
use std::{collections::HashMap, convert::Infallible, sync::Arc};
use tokio::sync::RwLock;

use chat_room_common::model::{LoggedUsers, Rooms};
use handler::room::{join_handler, leave_handler};
use handler::user::{login_handler, logout_handler};
use log::LevelFilter;
use simple_logger::SimpleLogger;
use warp::Filter;

#[tokio::main]
async fn main() {
    // Set up logging
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .unwrap();

    // Set up shared state
    let logged_users: LoggedUsers = Arc::new(RwLock::new(HashMap::new()));
    let rooms: Rooms = Arc::new(RwLock::new(HashMap::new()));

    // Routes and handlers
    // User
    let login_route = warp::path("login")
        // POST /login
        .and(warp::post())
        .and(warp::body::json())
        .and(with_logged_users(logged_users.clone()))
        .and_then(login_handler);

    let logout_route = warp::path("logout")
        // POST /logout
        .and(warp::post())
        .and(warp::body::json())
        .and(with_logged_users(logged_users.clone()))
        .and(with_rooms(rooms.clone()))
        .and_then(logout_handler);

    // Rooms
    let join_room_route = warp::path!("room" / "join")
        // POST /room/join
        .and(warp::post())
        .and(warp::body::json())
        .and(with_logged_users(logged_users.clone()))
        .and(with_rooms(rooms.clone()))
        .and_then(join_handler);

    let leave_room_route = warp::path!("room" / "leave")
        // POST /room/leave
        .and(warp::post())
        .and(warp::body::json())
        .and(with_logged_users(logged_users.clone()))
        .and(with_rooms(rooms.clone()))
        .and_then(leave_handler);

    let ws_route = warp::path("streaming")
        // GET /streaming/:user_name/:token
        .and(warp::get())
        .and(warp::ws())
        .and(warp::path::param())
        .and(warp::path::param())
        .and(with_logged_users(logged_users.clone()))
        .and(with_rooms(rooms.clone()))
        .and_then(streaming_handler);

    // Serve routes
    let routes = login_route
        .or(logout_route)
        .or(join_room_route)
        .or(leave_room_route)
        .or(ws_route)
        .with(warp::cors().allow_any_origin());
    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}

fn with_logged_users(
    users: LoggedUsers,
) -> impl Filter<Extract = (LoggedUsers,), Error = Infallible> + Clone {
    warp::any().map(move || users.clone())
}

fn with_rooms(rooms: Rooms) -> impl Filter<Extract = (Rooms,), Error = Infallible> + Clone {
    warp::any().map(move || rooms.clone())
}
