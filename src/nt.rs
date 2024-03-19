use thiserror::Error;

pub mod client;
pub mod client_config;
pub mod message_type;
pub mod messages;
pub mod subscription;
pub mod topic;

pub use message_type::*;
pub use messages::*;
pub use subscription::*;
pub use topic::*;

pub use client::Client;
pub use client_config::Config;

#[derive(Error, Debug)]
pub enum Error {
    #[error("WebSocket error: {0:?}")]
    Tungstenite(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("Json error: {0:?}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("Io error: {0:?}")]
    Io(#[from] std::io::Error),
    #[error("Server Unexpectedly Closed")]
    Closed(#[from] tokio::sync::mpsc::error::SendError<tokio_tungstenite::tungstenite::Message>),

    #[error("Timed out connecting to server")]
    ConnectTimeout(#[from] tokio::time::error::Elapsed),
    #[error("Server responded with an invalid type of message")]
    InvalidMessageType(&'static str),
}

pub type Result<T> = std::result::Result<T, Error>;
