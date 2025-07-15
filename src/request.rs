use hyper::{Body, Request as HyperRequest, Method, Uri};
use serde_json::Value;
use std::collections::HashMap;
use url::form_urlencoded;
use multer::Multipart;

#[derive(Debug)]
pub struct Request {
    pub method: Method,
    pub uri: Uri,
    pub headers: hyper::HeaderMap,
    pub body: Option<Body>, // Changed to Option<Body>
    pub params: HashMap<String, String>,
    pub query: HashMap<String, String>,
    pub json_body: Option<Value>,
    pub form_body: Option<HashMap<String, String>>,
    // Fields used by middleware
    pub user_id: Option<String>,
    pub user_roles: Vec<String>,
    pub session: Option<crate::session::Session>,
}

impl Request {
    pub async fn from_hyper(req: HyperRequest<Body>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let (parts, body) = req.into_parts();
        let query = Self::parse_query(&parts.uri);
        
        Ok(Request {
            method: parts.method,
            uri: parts.uri,
            headers: parts.headers,
            body: Some(body), // Store body as Some
            params: HashMap::new(),
            query,
            json_body: None,
            form_body: None,
            user_id: None,
            user_roles: Vec::new(),
            session: None,
        })
    }

    pub async fn json(&mut self) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        if self.json_body.is_none() {
            let body_bytes = hyper::body::to_bytes(self.body.take().unwrap_or_default()).await?; // Take body
            if !body_bytes.is_empty() {
                self.json_body = Some(serde_json::from_slice(&body_bytes)?);
            }
        }
        Ok(self.json_body.clone().unwrap_or(Value::Null))
    }

    pub async fn form(&mut self) -> Result<&HashMap<String, String>, Box<dyn std::error::Error + Send + Sync>> {
        if self.form_body.is_none() {
            let body_bytes = hyper::body::to_bytes(self.body.take().unwrap_or_default()).await?; // Take body
            let body_str = String::from_utf8(body_bytes.to_vec())?;
            let parsed_form: HashMap<String, String> = form_urlencoded::parse(body_str.as_bytes())
                .into_owned()
                .collect();
            self.form_body = Some(parsed_form);
        }
        Ok(self.form_body.as_ref().unwrap())
    }

    pub fn multipart(&mut self) -> Result<Multipart, Box<dyn std::error::Error + Send + Sync>> {
        let content_type = self.headers.get(hyper::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .ok_or("Missing Content-Type header for multipart form")?;
        
        let boundary = multer::parse_boundary(content_type)?;
        // Create a new Multipart instance, consuming the body
        Ok(Multipart::new(self.body.take().unwrap_or_default(), boundary))
    }

    pub fn param(&self, key: &str) -> Option<&String> {
        self.params.get(key)
    }

    pub fn query_param(&self, key: &str) -> Option<&String> {
        self.query.get(key)
    }

    fn parse_query(uri: &Uri) -> HashMap<String, String> {
        let mut query = HashMap::new();
        if let Some(query_str) = uri.query() {
            for pair in query_str.split('&') {
                if let Some((key, value)) = pair.split_once('=') {
                    query.insert(
                        percent_encoding::percent_decode_str(key).decode_utf8_lossy().to_string(),
                        percent_encoding::percent_decode_str(value).decode_utf8_lossy().to_string(),
                    );
                }
            }
        }
        query
    }
}
