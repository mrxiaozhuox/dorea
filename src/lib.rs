//! Dorea Storage Databasepub mod server;

const DOREA_VERSION: &'static str = "0.3.0";

pub mod server;
pub mod value;

mod configuration;
mod handle;
mod network;
mod command;
mod database;


type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T,Error>;