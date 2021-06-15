pub mod server;
pub mod client;


mod handle;
mod database;

type Result<T> = std::result::Result<T,String>;