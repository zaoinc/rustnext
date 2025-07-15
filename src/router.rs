use crate::{Request, Response, Handler, error::AppError}; // Updated imports
use crate::middleware::Middleware;
use async_trait::async_trait;
use hyper::Method;
use regex::Regex;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

#[derive(Clone)]
pub struct Route {
    pub path: String,
    pub method: Method,
    pub regex: Regex,
    pub param_names: Vec<String>,
    pub handler: Arc<dyn Handler>,
}

// Implement Debug manually for Route
impl fmt::Debug for Route {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Route")
            .field("path", &self.path)
            .field("method", &self.method)
            .field("param_names", &self.param_names)
            .finish()
    }
}
impl Route {
    pub fn new(method: Method, path: &str, handler: Arc<dyn Handler>) -> Self {
        let (regex, param_names) = Self::path_to_regex(path);
        Route {
            path: path.to_string(),
            method,
            regex,
            param_names,
            handler,
        }
    }

    fn path_to_regex(path: &str) -> (Regex, Vec<String>) {
        let mut regex_str = String::new();
        let mut param_names = Vec::new();
        let mut chars = path.chars().peekable();

        regex_str.push('^');

        while let Some(ch) = chars.next() {
            match ch {
                ':' => {
                    let mut param_name = String::new();
                    while let Some(&next_ch) = chars.peek() {
                        if next_ch.is_alphanumeric() || next_ch == '_' {
                            param_name.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                    param_names.push(param_name);
                    regex_str.push_str("([^/]+)");
                }
                '*' => {
                    regex_str.push_str("(.*)");
                }
                '.' | '+' | '?' | '^' | '$' | '{' | '}' | '[' | ']' | '|' | '(' | ')' | '\\' => {
                    regex_str.push('\\');
                    regex_str.push(ch);
                }
                _ => regex_str.push(ch),
            }
        }

        regex_str.push('$');
        (Regex::new(&regex_str).unwrap(), param_names)
    }

    pub fn matches(&self, method: &Method, path: &str) -> Option<HashMap<String, String>> {
        if self.method != *method {
            return None;
        }

        if let Some(captures) = self.regex.captures(path) {
            let mut params = HashMap::new();
            for (i, param_name) in self.param_names.iter().enumerate() {
                if let Some(capture) = captures.get(i + 1) {
                    params.insert(param_name.clone(), capture.as_str().to_string());
                }
            }
            Some(params)
        } else {
            None
        }
    }
}

pub struct Router {
    routes: Vec<Route>,
    middleware: Vec<Arc<dyn Middleware>>,
}

impl Router {
    pub fn new() -> Self {
        Router {
            routes: Vec::new(),
            middleware: Vec::new(),
        }
    }

    pub fn get<H>(mut self, path: &str, handler: H) -> Self
    where
        H: Handler + 'static,
    {
        self.routes.push(Route::new(Method::GET, path, Arc::new(handler)));
        self
    }

    pub fn post<H>(mut self, path: &str, handler: H) -> Self
    where
        H: Handler + 'static,
    {
        self.routes.push(Route::new(Method::POST, path, Arc::new(handler)));
        self
    }

    pub fn put<H>(mut self, path: &str, handler: H) -> Self
    where
        H: Handler + 'static,
    {
        self.routes.push(Route::new(Method::PUT, path, Arc::new(handler)));
        self
    }

    pub fn delete<H>(mut self, path: &str, handler: H) -> Self
    where
        H: Handler + 'static,
    {
        self.routes.push(Route::new(Method::DELETE, path, Arc::new(handler)));
        self
    }

    pub fn use_middleware<M>(mut self, middleware: M) -> Self
    where
        M: Middleware + 'static,
    {
        self.middleware.push(Arc::new(middleware));
        self
    }

    pub async fn handle_request(&self, mut req: Request) -> Result<Response, Box<dyn std::error::Error + Send + Sync>> {
        // Find matching route
        for route in &self.routes {
            if let Some(params) = route.matches(&req.method, req.uri.path()) {
                req.params = params;
                
                // Apply middleware chain
                let handler = route.handler.clone();
                let final_handler = self.middleware.iter().rev().fold(handler, |next, middleware| {
                    let middleware = middleware.clone();
                    Arc::new(MiddlewareHandler { middleware, next })
                });
                
                return final_handler.handle(req).await;
            }
        }

        // No route found, return 404 error
        Err(Box::new(AppError::NotFound(format!("Route not found: {}", req.uri.path()))))
    }
}

// Helper struct to chain middleware
struct MiddlewareHandler {
    middleware: Arc<dyn Middleware>,
    next: Arc<dyn Handler>,
}

#[async_trait]
impl Handler for MiddlewareHandler {
    async fn handle(&self, req: Request) -> Result<Response, Box<dyn std::error::Error + Send + Sync>> {
        self.middleware.handle(req, self.next.clone()).await
    }
}