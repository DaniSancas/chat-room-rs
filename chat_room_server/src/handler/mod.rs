pub mod room;
pub mod user;
pub mod websocket;

pub type Result<T> = std::result::Result<T, warp::Rejection>;
