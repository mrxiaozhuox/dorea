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


pub mod server;
pub mod client;


mod handle;
mod database;
mod structure;

type Result<T> = std::result::Result<T,String>;