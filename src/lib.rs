//! Dorea Storage Databasepub mod server;
#[allow(dead_code)]
use once_cell::sync::Lazy;

// Dorea db version (current)
const DOREA_VERSION: &'static str = "0.3.0";

// current version support load-storage version list.
#[allow(dead_code)]
const COMPATIBLE_VERSION: Lazy<Vec<&'static str>> = Lazy::new(|| {
    vec![
        DOREA_VERSION
    ]
});

#[cfg(feature = "server")]
pub mod server;

#[cfg(feature = "parser")]
pub mod value;

#[cfg(feature = "parser")]
pub mod network;

mod configuration;
mod handle;
mod command;
mod database;
mod logger;


type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T,Error>;