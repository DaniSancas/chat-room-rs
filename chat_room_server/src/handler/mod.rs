pub mod room;
pub mod user;

pub type Result<T> = std::result::Result<T, warp::Rejection>;
