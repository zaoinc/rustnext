use crate::{Request, Response, Handler};
use crate::middleware::Middleware; // Corrected import path for Middleware
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Instant;
use std::sync::Mutex;
// Removed unused import: use std::collections::HashMap;

#[derive(Clone)]
pub struct Metrics {
    pub request_counter: Arc<Mutex<u64>>,
    pub request_duration: Arc<Mutex<Vec<f64>>>,
    pub error_counter: Arc<Mutex<u64>>,
}

impl Metrics {
    pub fn new() -> Self {
        Metrics {
            request_counter: Arc::new(Mutex::new(0)),
            request_duration: Arc::new(Mutex::new(Vec::new())),
            error_counter: Arc::new(Mutex::new(0)),
        }
    }

    pub fn export(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let request_count = *self.request_counter.lock().unwrap();
        let error_count = *self.error_counter.lock().unwrap();
        let durations = self.request_duration.lock().unwrap();
        let avg_duration = if durations.is_empty() { 0.0 } else { durations.iter().sum::<f64>() / durations.len() as f64 };
        
        Ok(format!(
            "# HELP http_requests_total Total HTTP requests\n# TYPE http_requests_total counter\nhttp_requests_total {}\n# HELP http_errors_total Total HTTP errors\n# TYPE http_errors_total counter\nhttp_errors_total {}\n# HELP http_request_duration_avg Average HTTP request duration\n# TYPE http_request_duration_avg gauge\nhttp_request_duration_avg {}\n",
            request_count, error_count, avg_duration
        ))
    }
}

pub struct MetricsMiddleware {
    metrics: Arc<Metrics>,
}

impl MetricsMiddleware {
    pub fn new(metrics: Arc<Metrics>) -> Self {
        MetricsMiddleware { metrics }
    }
}

#[async_trait]
impl Middleware for MetricsMiddleware {
    async fn handle(
        &self,
        req: Request,
        next: Arc<dyn Handler>,
    ) -> Result<Response, Box<dyn std::error::Error + Send + Sync>> {
        let start = Instant::now();
        {
            let mut counter = self.metrics.request_counter.lock().unwrap();
            *counter += 1;
        }
        
        let result = next.handle(req).await;
        
        let duration = start.elapsed();
        {
            let mut durations = self.metrics.request_duration.lock().unwrap();
            durations.push(duration.as_secs_f64());
        }
        
        if result.is_err() {
            let mut error_counter = self.metrics.error_counter.lock().unwrap();
            *error_counter += 1;
        }
        
        result
    }
}
