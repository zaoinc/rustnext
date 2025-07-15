use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;

pub struct FileUpload {
    pub filename: String,
    pub content_type: String,
    pub size: usize,
    pub data: Vec<u8>,
}

impl FileUpload {
    pub async fn save_to(&self, directory: &str) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
        let path = PathBuf::from(directory).join(&self.filename);
        
        // Create directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        
        let mut file = fs::File::create(&path).await?;
        file.write_all(&self.data).await?;
        
        Ok(path)
    }
}

pub async fn parse_form_data(
    body: hyper::Body,
) -> Result<Vec<FileUpload>, Box<dyn std::error::Error + Send + Sync>> {
    let body_bytes = hyper::body::to_bytes(body).await?;
    
    // Simple form parsing - in a real implementation you'd use a proper multipart parser
    let uploads = vec![
        FileUpload {
            filename: "example.txt".to_string(),
            content_type: "text/plain".to_string(),
            size: body_bytes.len(),
            data: body_bytes.to_vec(),
        }
    ];

    Ok(uploads)
}
