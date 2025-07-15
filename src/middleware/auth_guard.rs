use crate::{Request, Response, Handler};
use crate::middleware::Middleware;
use async_trait::async_trait;
use std::sync::Arc;
use std::collections::HashMap; // Used for RateLimiter's requests field
use std::time::Instant; // Used for RateLimiter

pub struct AuthGuard {
    pub required_roles: Vec<String>,
    pub redirect_url: Option<String>,
}

impl AuthGuard {
    pub fn new() -> Self {
        AuthGuard {
            required_roles: Vec::new(),
            redirect_url: None,
        }
    }

    pub fn require_role(mut self, role: &str) -> Self {
        self.required_roles.push(role.to_string());
        self
    }

    pub fn redirect_to(mut self, url: &str) -> Self {
        self.redirect_url = Some(url.to_string());
        self
    }
}

#[async_trait]
impl Middleware for AuthGuard {
    async fn handle(
        &self,
        req: Request,
        next: Arc<dyn Handler>,
    ) -> Result<Response, Box<dyn std::error::Error + Send + Sync>> {
        // Check if user is authenticated
        if req.user_id.is_none() {
            if let Some(redirect_url) = &self.redirect_url {
                return Ok(Response::new().redirect(redirect_url));
            } else {
                return Ok(Response::new()
                    .status(hyper::StatusCode::UNAUTHORIZED)
                    .json(&serde_json::json!({"error": "Authentication required"}))?);
            }
        }

        // Check required roles
        if !self.required_roles.is_empty() {
            let user_has_required_role = self.required_roles.iter()
                .any(|required_role| req.user_roles.contains(required_role));
            
            if !user_has_required_role {
                return Ok(Response::new()
                    .status(hyper::StatusCode::FORBIDDEN)
                    .json(&serde_json::json!({"error": "Insufficient permissions"}))?);
            }
        }

        next.handle(req).await
    }
}

pub struct RateLimiter {
    pub max_requests: u32,
    pub window_seconds: u64,
    pub requests: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, (u32, std::time::Instant)>>>,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window_seconds: u64) -> Self {
        RateLimiter {
            max_requests,
            window_seconds,
            requests: std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
        }
    }
}

#[async_trait]
impl Middleware for RateLimiter {
    async fn handle(
        &self,
        req: Request,
        next: Arc<dyn Handler>,
    ) -> Result<Response, Box<dyn std::error::Error + Send + Sync>> {
        let client_ip = req.headers
            .get("x-forwarded-for")
            .or_else(|| req.headers.get("x-real-ip"))
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown")
            .to_string();

        let now = Instant::now();
        
        // Perform rate limiting logic in a separate, synchronous block
        let rate_limit_exceeded = {
            let mut requests_guard = self.requests.lock().unwrap(); // Acquire lock
            
            let (count, last_request) = requests_guard.entry(client_ip.clone())
                .or_insert((0, now));

            // Reset counter if window has passed
            if now.duration_since(*last_request).as_secs() > self.window_seconds {
                *count = 0;
                *last_request = now;
            }

            *count += 1;

            *count > self.max_requests // Return true if exceeded, false otherwise
        }; // `requests_guard` is dropped here, releasing the mutex

        if rate_limit_exceeded {
            return Ok(Response::new()
                .status(hyper::StatusCode::TOO_MANY_REQUESTS)
                .header("Retry-After", &self.window_seconds.to_string())
                .json(&serde_json::json!({"error": "Rate limit exceeded"}))?);
        }

        next.handle(req).await
    }
}
