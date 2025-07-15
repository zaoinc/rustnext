use crate::{Response, ui::{div, h1, p, text, get_renderer}};
use hyper::StatusCode;
use std::fmt;
use std::error::Error as StdError; // Alias for clarity

#[derive(Debug, Clone)] // Added Clone derive
pub enum AppError {
    NotFound(String),
    Internal(String),
    BadRequest(String),
    Unauthorized(String),
    Forbidden(String),
    // Add more specific errors as needed
    #[allow(dead_code)] // Allow unused variant for now
    Custom(StatusCode, String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::NotFound(msg) => write!(f, "Not Found: {}", msg),
            AppError::Internal(msg) => write!(f, "Internal Server Error: {}", msg),
            AppError::BadRequest(msg) => write!(f, "Bad Request: {}", msg),
            AppError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            AppError::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
            AppError::Custom(_, msg) => write!(f, "Custom Error: {}", msg),
        }
    }
}

impl StdError for AppError {}

// Convert generic Box<dyn Error> to AppError
impl From<Box<dyn StdError + Send + Sync>> for AppError {
    fn from(err: Box<dyn StdError + Send + Sync>) -> Self {
        // Attempt to downcast to specific AppError variants if possible,
        // otherwise wrap in Internal.
        if let Some(app_err) = err.downcast_ref::<AppError>() {
            app_err.clone() // Now AppError implements Clone
        } else {
            AppError::Internal(err.to_string())
        }
    }
}

impl From<hyper::Error> for AppError {
    fn from(err: hyper::Error) -> Self {
        AppError::Internal(format!("Hyper error: {}", err))
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::BadRequest(format!("JSON parsing error: {}", err))
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Internal(format!("IO error: {}", err))
    }
}

impl From<multer::Error> for AppError {
    fn from(err: multer::Error) -> Self {
        AppError::BadRequest(format!("Multipart parsing error: {}", err))
    }
}

impl From<url::ParseError> for AppError {
    fn from(err: url::ParseError) -> Self {
        AppError::BadRequest(format!("URL parsing error: {}", err))
    }
}

// Trait for converting AppError to Response
pub trait IntoResponse {
    fn into_response(&self) -> Result<Response, Box<dyn StdError + Send + Sync>>;
}

impl IntoResponse for AppError {
    fn into_response(&self) -> Result<Response, Box<dyn StdError + Send + Sync>> {
        let (status, message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.clone()),
            AppError::Custom(s, msg) => (*s, msg.clone()),
        };

        let error_page = div()
            .class("container")
            .child(h1().child(text(&format!("Error {}: {}", status.as_u16(), status.canonical_reason().unwrap_or("Unknown Error")))))
            .child(p().child(text(&message)));

        get_renderer().render_to_response(&error_page)
            .map(|res| res.status(status))
            .map_err(|e| e) // Changed this line: 'e' is already the correct Box<dyn StdError> type
    }
}
