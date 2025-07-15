use hyper::{Body, Response as HyperResponse, StatusCode};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Response {
    pub status: StatusCode,
    pub headers: HashMap<String, String>,
    pub body: Body,
}

impl Response {
    pub fn new() -> Self {
        Response {
            status: StatusCode::OK,
            headers: HashMap::new(),
            body: Body::empty(),
        }
    }

    pub fn status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }

    // Modified to accept any type that can be converted into a String
    pub fn header<V: Into<String>>(mut self, key: &str, value: V) -> Self {
        self.headers.insert(key.to_string(), value.into());
        self
    }

    pub fn json<T: Serialize>(mut self, data: &T) -> Result<Self, serde_json::Error> {
        let json_str = serde_json::to_string(data)?;
        self.body = Body::from(json_str);
        self.headers.insert("Content-Type".to_string(), "application/json".to_string());
        Ok(self)
    }

    pub fn text(mut self, text: &str) -> Self {
        self.body = Body::from(text.to_string());
        self.headers.insert("Content-Type".to_string(), "text/plain".to_string());
        self
    }

    pub fn html(mut self, html: &str) -> Self {
        self.body = Body::from(html.to_string());
        self.headers.insert("Content-Type".to_string(), "text/html".to_string());
        self
    }

    pub fn body(mut self, body: Body) -> Self {
        self.body = body;
        self
    }

    pub fn redirect(mut self, location: &str) -> Self {
        self.status = StatusCode::FOUND;
        self.headers.insert("Location".to_string(), location.to_string());
        self
    }

    pub fn into_hyper(self) -> HyperResponse<Body> {
        let mut response = HyperResponse::builder().status(self.status);
        
        for (key, value) in self.headers {
            response = response.header(key, value);
        }
        
        response.body(self.body).unwrap()
    }
}

impl Default for Response {
    fn default() -> Self {
        Self::new()
    }
}
