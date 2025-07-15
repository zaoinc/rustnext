use crate::{Request, Response, Handler};
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use tokio::fs;

pub struct AssetManager {
    pub root_dir: PathBuf,
    pub cache: HashMap<String, CachedAsset>,
    pub optimization: AssetOptimization,
}

#[derive(Clone)]
pub struct CachedAsset {
    pub content: Vec<u8>,
    pub content_type: String,
    pub etag: String,
    pub last_modified: String,
}

pub struct AssetOptimization {
    pub minify_css: bool,
    pub minify_js: bool,
    pub compress_images: bool,
    pub cache_duration: u64,
}

impl Default for AssetOptimization {
    fn default() -> Self {
        AssetOptimization {
            minify_css: true,
            minify_js: true,
            compress_images: true,
            cache_duration: 3600, // 1 hour
        }
    }
}

impl AssetManager {
    pub fn new<P: AsRef<Path>>(root_dir: P) -> Self {
        AssetManager {
            root_dir: root_dir.as_ref().to_path_buf(),
            cache: HashMap::new(),
            optimization: AssetOptimization::default(),
        }
    }

    pub async fn serve_asset(&mut self, path: &str) -> Result<Response, Box<dyn std::error::Error + Send + Sync>> {
        let file_path = self.root_dir.join(path.trim_start_matches('/'));
        
        // Security check: prevent directory traversal
        let canonical_root = fs::canonicalize(&self.root_dir).await?;
        let canonical_file = match fs::canonicalize(&file_path).await {
            Ok(path) => path,
            Err(_) => {
                return Ok(Response::new()
                    .status(hyper::StatusCode::NOT_FOUND)
                    .text("Asset not found"));
            }
        };
        
        if !canonical_file.starts_with(&canonical_root) {
            return Ok(Response::new()
                .status(hyper::StatusCode::FORBIDDEN)
                .text("Forbidden"));
        }

        // Check cache first
        if let Some(cached) = self.cache.get(path) {
            return Ok(Response::new()
                .header("Content-Type", &cached.content_type)
                .header("ETag", &cached.etag)
                .header("Cache-Control", &format!("public, max-age={}", self.optimization.cache_duration))
                .body(hyper::Body::from(cached.content.clone())));
        }

        // Read and process file
        let content = fs::read(&file_path).await?;
        let content_type = self.get_content_type(&file_path);
        let processed_content = self.optimize_content(&content, &content_type).await?;
        
        // Generate ETag using a simple hash
        let etag = format!("\"{}\"", format!("{:x}", md5::compute(&processed_content)));
        
        // Cache the asset
        let cached_asset = CachedAsset {
            content: processed_content.clone(),
            content_type: content_type.clone(),
            etag: etag.clone(),
            last_modified: chrono::Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string(),
        };
        self.cache.insert(path.to_string(), cached_asset);

        Ok(Response::new()
            .header("Content-Type", &content_type)
            .header("ETag", &etag)
            .header("Cache-Control", &format!("public, max-age={}", self.optimization.cache_duration))
            .body(hyper::Body::from(processed_content)))
    }

    async fn optimize_content(&self, content: &[u8], content_type: &str) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        match content_type {
            "text/css" if self.optimization.minify_css => {
                // Simple CSS minification (remove comments and extra whitespace)
                let css_content = String::from_utf8_lossy(content);
                let minified = css_content
                    .lines()
                    .map(|line| line.trim())
                    .filter(|line| !line.starts_with("/*") && !line.is_empty())
                    .collect::<Vec<_>>()
                    .join(" ");
                Ok(minified.into_bytes())
            }
            "application/javascript" | "text/javascript" if self.optimization.minify_js => {
                // Simple JS minification (remove comments and extra whitespace)
                let js_content = String::from_utf8_lossy(content);
                let minified = js_content
                    .lines()
                    .map(|line| line.trim())
                    .filter(|line| !line.trim_start().starts_with("//") && !line.is_empty())
                    .collect::<Vec<_>>()
                    .join(" ");
                Ok(minified.into_bytes())
            }
            _ => Ok(content.to_vec()),
        }
    }

    fn get_content_type(&self, path: &Path) -> String {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("css") => "text/css".to_string(),
            Some("js") => "application/javascript".to_string(),
            Some("png") => "image/png".to_string(),
            Some("jpg") | Some("jpeg") => "image/jpeg".to_string(),
            Some("gif") => "image/gif".to_string(),
            Some("svg") => "image/svg+xml".to_string(),
            Some("woff") => "font/woff".to_string(),
            Some("woff2") => "font/woff2".to_string(),
            Some("ttf") => "font/ttf".to_string(),
            Some("ico") => "image/x-icon".to_string(),
            _ => "application/octet-stream".to_string(),
        }
    }
}

#[async_trait]
impl Handler for AssetManager {
    async fn handle(&self, req: Request) -> Result<Response, Box<dyn std::error::Error + Send + Sync>> {
        let path = req.uri.path();
        let mut manager = self.clone();
        manager.serve_asset(path).await
    }
}

impl Clone for AssetManager {
    fn clone(&self) -> Self {
        AssetManager {
            root_dir: self.root_dir.clone(),
            cache: self.cache.clone(),
            optimization: AssetOptimization {
                minify_css: self.optimization.minify_css,
                minify_js: self.optimization.minify_js,
                compress_images: self.optimization.compress_images,
                cache_duration: self.optimization.cache_duration,
            },
        }
    }
}
