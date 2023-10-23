pub mod clients;
pub use clients::{BlockingClient, BufferedClient, Client};

pub const DEFAULT_PORT: u16 = 6379;

mod connection;
pub use connection::Connection;

mod db;

mod frame;
pub use frame::Frame;

mod parse;

mod shutdown;

pub mod server;

mod cmd;
pub use cmd::Command;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub type Result<T> = std::result::Result<T, Error>;
