# chat-room-rs

A pet project to create a multi-room chat with websockets in Rust 🦀

## Motivation

- Learn about [warp](https://crates.io/crates/warp) and websockets in Rust.
- Evaluate performance of the solution.

Inspired in: 
https://blog.logrocket.com/build-websocket-server-with-rust/


## Endpoints

### User

```sh
# Login into the system
POST /login
params: { "user_name": "<user_name>" }
response: { "token": "<token>" }
```

```sh
# Logout from the system
POST /logout
params: { "user_name": "<user_name>", "token": "<token>" }
response: 200 | 403
```

### Room

```sh
# Create or join an already created room
POST /room/join
params: { "user_name": "<user_name>", "token": "<token>", "room_name": "<room_name>" }
response: 200 | 403
```

```sh
# Leave a room
POST /room/leave
params: { "user_name": "<user_name>", "token": "<token>", "room_name": "<room_name>" }
response: 200
```
