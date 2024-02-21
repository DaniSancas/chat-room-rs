mod handler;

use std::{
    collections::HashMap,
    convert::Infallible,
    sync::{Arc, Mutex},
};

use handler::LoggedUsers;
use warp::Filter;

#[tokio::main]
async fn main() {
    let logged_users: LoggedUsers = Arc::new(Mutex::new(HashMap::new()));

    let login_routes = warp::path("login")
        // POST /login
        .and(warp::post())
        .and(warp::body::json())
        .and(with_logged_users(logged_users.clone()))
        .and_then(handler::login_handler);

    let logout_routes = warp::path("logout")
        // POST /logout
        .and(warp::post())
        .and(warp::body::json())
        .and(with_logged_users(logged_users.clone()))
        .and_then(handler::logout_handler);

    let routes = login_routes
        .or(logout_routes)
        .with(warp::cors().allow_any_origin());

    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}

fn with_logged_users(
    users: LoggedUsers,
) -> impl Filter<Extract = (LoggedUsers,), Error = Infallible> + Clone {
    warp::any().map(move || users.clone())
}
