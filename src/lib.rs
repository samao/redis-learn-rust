mod clients;

pub const DEFAULT_PORT: u16 = 6379;

mod connection;
pub use connection::Connection;

mod db;
use db::{Db, DbDropGuard};

mod frame;
pub use frame::Frame;

mod parse;
use parse::{Parse, ParseError};

pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub type Result<T> = std::result::Result<T, Error>;
