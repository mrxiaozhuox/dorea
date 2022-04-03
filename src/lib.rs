//! Dorea Storage Databasepub mod server;

#![allow(dead_code)]
use once_cell::sync::Lazy;

// Dorea db version (current)
pub const DOREA_VERSION: &str = "0.4.0";

// current version support load-storage version list.
#[allow(dead_code)]
static COMPATIBLE_VERSION: Lazy<Vec<String>> = Lazy::new(|| {
    vec![
        format!(
            "{:x}",
            md5::compute(format!("Dorea::{}", DOREA_VERSION).as_bytes())
        ),
        format!(
            "{:x}",
            md5::compute(format!("Dorea::{}", "0.3.0").as_bytes())
        ),
        format!(
            "{:x}",
            md5::compute(format!("Dorea::{}", "0.3.0-alpha").as_bytes())
        ),
    ]
});

#[cfg(feature = "server")]
pub mod server;

#[cfg(feature = "client")]
pub mod client;

#[cfg(feature = "processor")]
pub mod value;

#[cfg(feature = "processor")]
pub mod network;

#[cfg(feature = "processor")]
pub mod macros;

#[cfg(feature = "processor")]
pub mod docs;

#[cfg(feature = "server")]
mod command;

#[cfg(feature = "server")]
mod configure;

#[cfg(feature = "server")]
mod database;

#[cfg(feature = "server")]
mod event;

#[cfg(feature = "server")]
mod handle;

#[cfg(feature = "server")]
mod logger;

#[cfg(feature = "server")]
mod service;
mod tool;

type Result<T> = std::result::Result<T, anyhow::Error>;
