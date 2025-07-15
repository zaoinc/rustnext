use crate::{Request, Response, Handler};
use async_trait::async_trait;
use std::sync::Arc;

// Moved Middleware trait definition here
#[async_trait]
pub trait Middleware: Send + Sync + 'static {
    async fn handle(
        &self,
        req: Request,
        next: Arc<dyn Handler>,
    ) -> Result<Response, Box<dyn std::error::Error + Send + Sync>>;
}

// Moved from src/middleware.rs
// Logger middleware
pub struct Logger;

#[async_trait]
impl Middleware for Logger {
    async fn handle(
        &self,
        req: Request,
        next: Arc<dyn Handler>,
    ) -> Result<Response, Box<dyn std::error::Error + Send + Sync>> {
        let start = std::time::Instant::now();
        let method = req.method.clone();
        let uri = req.uri.clone();
        
        let response = next.handle(req).await?;
        
        let duration = start.elapsed(); // `duration` is already defined here
        println!("{} {} {} - {:?}", method, uri, response.status, duration);
        
        Ok(response)
    }
}

// CORS middleware
pub struct Cors {
    pub allow_origin: String,
    pub allow_methods: String,
    pub allow_headers: String,
}

impl Cors {
    pub fn new() -> Self {
        Cors {
            allow_origin: "*".to_string(),
            allow_methods: "GET, POST, PUT, DELETE, OPTIONS".to_string(),
            allow_headers: "Content-Type, Authorization".to_string(),
        }
    }

    pub fn allow_origin(mut self, origin: &str) -> Self {
        self.allow_origin = origin.to_string();
        self
    }
}

#[async_trait]
impl Middleware for Cors {
    async fn handle(
        &self,
        req: Request,
        next: Arc<dyn Handler>,
    ) -> Result<Response, Box<dyn std::error::Error + Send + Sync>> {
        if req.method == hyper::Method::OPTIONS {
            return Ok(Response::new()
                .header("Access-Control-Allow-Origin", &self.allow_origin)
                .header("Access-Control-Allow-Methods", &self.allow_methods)
                .header("Access-Control-Allow-Headers", &self.allow_headers)
                .status(hyper::StatusCode::OK));
        }

        let mut response = next.handle(req).await?;
        response.headers.insert("Access-Control-Allow-Origin".to_string(), self.allow_origin.clone());
        Ok(response)
    }
}

// Existing module declarations
pub mod auth_guard;

// Export all public middleware components and the trait
pub use auth_guard::{AuthGuard, RateLimiter};
// Removed redundant `pub use super::middleware::...` as they are defined directly in this mod.rs
// pub use super::middleware::Middleware;
// pub use super::middleware::Logger;
// pub use super::middleware::Cors;
