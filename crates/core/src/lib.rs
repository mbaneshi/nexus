//! Nexus core — shared types, database, config, and error handling.
//!
//! This crate has zero dependencies on any feature crate or surface crate.
//! All other nexus crates depend on this.

pub mod config;
pub mod db;
pub mod error;
pub mod models;
pub mod output;
pub mod ports;

pub use config::Config;
pub use error::{NexusError, Result};
