//! A key-value store database system.
//! provide: server & client
//!
//! author: [ZhuoEr Liu](http://github.com/mrxiaozhuox)
//!
//! support data-type:
//! - String
//! - Number
//! - Boolean
//! - Dict
//!
//! ### public mod
//!
//! - *server*: Dorea-server system - start & manager the server.
//! - *client*: Dorea-client system - connect & execute the database.
//!

#[macro_use]
mod macros;

pub mod server;
pub mod client;
pub mod tools;


mod handle;
mod database;
mod structure;
mod logger;


pub type Result<T> = std::result::Result<T,String>;