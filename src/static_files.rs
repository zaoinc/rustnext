use crate::{Request, Response, Handler};
use async_trait::async_trait;
use std::path::Path;
use tokio::fs;

pub struct StaticFiles {
    dir: String,
    prefix: String,
}

impl StaticFiles {
    pub fn new(dir: &str, prefix: &str) -> Self {
        StaticFiles {
            dir: dir.to_string(),
            prefix: prefix.to_string(),
        }
    }

    async fn serve_file(&self, path: &str) -> Result<Response, Box<dyn std::error::Error + Send + Sync>> {
        let file_path = Path::new(&self.dir).join(path.trim_start_matches('/'));
        
        // Security check: prevent directory traversal
        let canonical_dir = std::fs::canonicalize(&self.dir)?;
        let canonical_file = match file_path.canonicalize() {
            Ok(path) => path,
            Err(_) => {
                return Ok(Response::new()
                    .status(hyper::StatusCode::NOT_FOUND)
                    .text("File not found"));
            }
        };
        
        if !canonical_file.starts_with(&canonical_dir) {
            return Ok(Response::new()
                .status(hyper::StatusCode::FORBIDDEN)
                .text("Forbidden"));
        }

        match fs::read(&file_path).await {
            Ok(contents) => {
                let mime_type = mime_guess::from_path(&file_path)
                    .first_or_octet_stream()
                    .to_string();
                
                Ok(Response::new()
                    .header("Content-Type", &mime_type)
                    .header("Content-Length", &contents.len().to_string())
                    .header("Cache-Control", "public, max-age=3600") // 1 hour cache
                    .status(hyper::StatusCode::OK)
                    .body(hyper::Body::from(contents)))
            }
            Err(_) => Ok(Response::new()
                .status(hyper::StatusCode::NOT_FOUND)
                .text("File not found")),
        }
    }
}

#[async_trait]
impl Handler for StaticFiles {
    async fn handle(&self, req: Request) -> Result<Response, Box<dyn std::error::Error + Send + Sync>> {
        let path = req.uri.path();
        if path.starts_with(&self.prefix) {
            let file_path = &path[self.prefix.len()..];
            self.serve_file(file_path).await
        } else {
            Ok(Response::new()
                .status(hyper::StatusCode::NOT_FOUND)
                .text("Not Found"))
        }
    }
}
