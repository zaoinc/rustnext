pub mod app;
pub mod router;
pub mod handler;
pub mod middleware;
pub mod request;
pub mod response;
pub mod server;
pub mod static_files;
pub mod template;
pub mod auth;
pub mod cache;
pub mod compression;
pub mod database;
pub mod file_upload;
pub mod metrics;
pub mod session;
pub mod ui;
pub mod forms;
pub mod api;
pub mod config;
pub mod assets;
pub mod error; // New module export
pub mod logging; // New module export

// Optional dev module for development utilities
#[cfg(feature = "dev")]
pub mod dev;

pub use app::App;
pub use router::{Router, Route};
pub use handler::Handler;
pub use middleware::{Middleware, Logger, Cors};
pub use request::Request;
pub use response::Response;
pub use server::Server;

// UI exports
pub use ui::*;

// Form exports
pub use forms::*;

// API exports
pub use api::*;

// Config exports
pub use config::*;

// Asset exports
pub use assets::*;

// Error exports
pub use error::{AppError, IntoResponse}; // Export AppError and IntoResponse trait

// Logging exports
pub use logging::init_logging; // Export init_logging function

// Re-export commonly used types
pub use hyper::{Body, Method, StatusCode};
pub use serde::{Deserialize, Serialize};
pub use serde_json::{json, Value};
pub use async_trait::async_trait;

// Re-export global state getters
pub use config::{get_config, init_config};
pub use database::{get_database, init_database};
pub use cache::{get_cache, init_cache};
