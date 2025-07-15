use crate::{Request, Response, Handler};
use crate::middleware::Middleware; // Corrected import path for Middleware
use async_trait::async_trait;
use async_compression::tokio::write::{GzipEncoder, BrotliEncoder};
use tokio::io::AsyncWriteExt;
use std::sync::Arc;

pub struct CompressionMiddleware {
    min_size: usize,
}

impl CompressionMiddleware {
    pub fn new() -> Self {
        CompressionMiddleware {
            min_size: 1024, // Only compress responses larger than 1KB
        }
    }

    pub fn min_size(mut self, size: usize) -> Self {
        self.min_size = size;
        self
    }

    async fn compress_response(&self, response: Response, encoding: &str) -> Result<Response, Box<dyn std::error::Error + Send + Sync>> {
        let body_bytes = hyper::body::to_bytes(response.body).await?;
        
        if body_bytes.len() < self.min_size {
            return Ok(Response {
                status: response.status,
                headers: response.headers,
                body: hyper::Body::from(body_bytes),
            });
        }

        let compressed = match encoding {
            "gzip" => {
                let mut encoder = GzipEncoder::new(Vec::new());
                encoder.write_all(&body_bytes).await?;
                encoder.shutdown().await?;
                encoder.into_inner()
            }
            "br" => {
                let mut encoder = BrotliEncoder::new(Vec::new());
                encoder.write_all(&body_bytes).await?;
                encoder.shutdown().await?;
                encoder.into_inner()
            }
            _ => return Ok(Response {
                status: response.status,
                headers: response.headers,
                body: hyper::Body::from(body_bytes),
            }),
        };

        let mut headers = response.headers;
        headers.insert("Content-Encoding".to_string(), encoding.to_string());
        headers.insert("Content-Length".to_string(), compressed.len().to_string());

        Ok(Response {
            status: response.status,
            headers,
            body: hyper::Body::from(compressed),
        })
    }
}

#[async_trait]
impl Middleware for CompressionMiddleware {
    async fn handle(
        &self,
        req: Request,
        next: Arc<dyn Handler>,
    ) -> Result<Response, Box<dyn std::error::Error + Send + Sync>> {
        let accept_encoding = req.headers
            .get("accept-encoding")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        let response = next.handle(req).await?;

        // Choose compression method based on client support
        if accept_encoding.contains("br") {
            self.compress_response(response, "br").await
        } else if accept_encoding.contains("gzip") {
            self.compress_response(response, "gzip").await
        } else {
            Ok(response)
        }
    }
}
