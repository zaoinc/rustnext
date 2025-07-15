use crate::{Request, Response};
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use once_cell::sync::OnceCell;
use tokio::sync::Mutex;
use regex::Regex; // Add this import

pub struct ApiRoute {
    pub path: String,
    pub method: hyper::Method,
    pub regex: Regex, // Add regex field
    pub param_names: Vec<String>, // Add param_names field
    pub handler: Box<dyn ApiHandler>,
}

impl ApiRoute {
    pub fn new<H>(method: hyper::Method, path: &str, handler: H) -> Self
    where
        H: ApiHandler + 'static,
    {
        let (regex, param_names) = Self::path_to_regex(path);
        ApiRoute {
            path: path.to_string(),
            method,
            regex,
            param_names,
            handler: Box::new(handler),
        }
    }

    // Copied from src/router.rs to enable regex matching for API routes
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
}

#[async_trait]
pub trait ApiHandler: Send + Sync {
    async fn handle(&self, req: Request) -> Result<ApiResponse, ApiError>;
}

#[derive(Debug)]
pub struct ApiResponse {
    pub status: hyper::StatusCode,
    pub data: Value,
    pub headers: HashMap<String, String>,
}

impl ApiResponse {
    pub fn ok(data: Value) -> Self {
        ApiResponse {
            status: hyper::StatusCode::OK,
            data,
            headers: HashMap::new(),
        }
    }

    pub fn created(data: Value) -> Self {
        ApiResponse {
            status: hyper::StatusCode::CREATED,
            data,
            headers: HashMap::new(),
        }
    }

    pub fn error(status: hyper::StatusCode, message: &str) -> Self {
        ApiResponse {
            status,
            data: serde_json::json!({"error": message}),
            headers: HashMap::new(),
        }
    }

    pub fn with_status(mut self, status: hyper::StatusCode) -> Self {
        self.status = status;
        self
    }

    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }
}

#[derive(Debug)]
pub struct ApiError {
    pub status: hyper::StatusCode,
    pub message: String,
}

impl ApiError {
    pub fn bad_request(message: &str) -> Self {
        ApiError {
            status: hyper::StatusCode::BAD_REQUEST,
            message: message.to_string(),
        }
    }

    pub fn not_found(message: &str) -> Self {
        ApiError {
            status: hyper::StatusCode::NOT_FOUND,
            message: message.to_string(),
        }
    }

    pub fn internal_error(message: &str) -> Self {
        ApiError {
            status: hyper::StatusCode::INTERNAL_SERVER_ERROR,
            message: message.to_string(),
        }
    }
}

pub struct ApiRegistry {
    routes: Vec<ApiRoute>,
}

impl ApiRegistry {
    pub fn new() -> Self {
        ApiRegistry {
            routes: Vec::new(),
        }
    }

    pub fn add_route<H>(&mut self, method: hyper::Method, path: &str, handler: H)
    where
        H: ApiHandler + 'static,
    {
        self.routes.push(ApiRoute::new(method, path, handler)); // Use the new constructor
    }

    pub async fn handle_request(&self, mut req: Request) -> Option<Response> {
        let req_path = req.uri.path().to_string(); // Clone path once for iteration
        let req_method = req.method.clone(); // Clone method once for iteration

        for route in &self.routes {
            if route.method == req_method {
                if let Some(captures) = route.regex.captures(&req_path) {
                    // Found a match! Now, extract parameters and update the request.
                    let mut params = HashMap::new();
                    for (i, param_name) in route.param_names.iter().enumerate() {
                        if let Some(capture) = captures.get(i + 1) {
                            params.insert(param_name.clone(), capture.as_str().to_string());
                        }
                    }
                    
                    // Extend the request's parameters with the newly extracted ones
                    req.params.extend(params);

                    // Call the handler with the modified `req`
                    match route.handler.handle(req).await { // `req` is moved here
                        Ok(api_response) => {
                            let mut response = Response::new()
                                .status(api_response.status)
                                .json(&api_response.data)
                                .unwrap_or_else(|_| Response::new().status(hyper::StatusCode::INTERNAL_SERVER_ERROR));
                            
                            for (key, value) in api_response.headers {
                                response.headers.insert(key, value);
                            }
                            
                            return Some(response);
                        }
                        Err(api_error) => {
                            return Some(
                                Response::new()
                                    .status(api_error.status)
                                    .json(&serde_json::json!({"error": api_error.message}))
                                    .unwrap_or_else(|_| Response::new().status(hyper::StatusCode::INTERNAL_SERVER_ERROR))
                            );
                        }
                    }
                }
            }
        }
        None // If no route matches after checking all, return None
    }
}

static GLOBAL_API_REGISTRY: OnceCell<Mutex<ApiRegistry>> = OnceCell::new();

pub fn get_api_registry() -> &'static Mutex<ApiRegistry> {
    GLOBAL_API_REGISTRY.get_or_init(|| Mutex::new(ApiRegistry::new()))
}

#[macro_export]
macro_rules! api_route {
    ($method:expr, $path:expr, $handler:expr) => {
        // This macro now expands to an async block that returns a Future
        async {
            let mut registry = crate::api::get_api_registry().lock().await;
            registry.add_route($method, $path, $handler); // This will now call ApiRoute::new internally
            Ok::<(), Box<dyn std::error::Error + Send + Sync>>(()) // Return a Result
        }
    };
}
