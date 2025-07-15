use crate::{Router, Request, Response, Handler, static_files::StaticFiles, template::TemplateEngine, error::{AppError, IntoResponse}};
use async_trait::async_trait;
use std::sync::Arc; // Ensure Arc is imported

pub struct App {
    router: Router,
    static_handler: Option<Arc<StaticFiles>>,
    template_engine: Option<Arc<TemplateEngine>>,
    // This field type is correct, it stores an Arc to the error handler trait object
    error_handler: Arc<dyn Fn(AppError) -> Result<Response, Box<dyn std::error::Error + Send + Sync>> + Send + Sync>,
}

impl App {
    pub fn new() -> Self {
        App {
            router: Router::new(),
            static_handler: None,
            template_engine: None,
            // Default error handler is also an Arc
            error_handler: Arc::new(|err: AppError| err.into_response()),
        }
    }

    pub fn router(mut self, router: Router) -> Self {
        self.router = router;
        self
    }

    pub fn static_files(mut self, dir: &str, prefix: &str) -> Self {
        self.static_handler = Some(Arc::new(StaticFiles::new(dir, prefix)));
        self
    }

    pub fn templates(mut self, engine: TemplateEngine) -> Self {
        self.template_engine = Some(Arc::new(engine));
        self
    }

    // Modified: Now accepts an Arc<dyn Fn(...)> directly
    pub fn error_handler(mut self, handler: Arc<dyn Fn(AppError) -> Result<Response, Box<dyn std::error::Error + Send + Sync>> + Send + Sync + 'static>) -> Self
    {
        self.error_handler = handler; // Directly assign the Arc
        self
    }
}

#[async_trait]
impl Handler for App {
    async fn handle(&self, req: Request) -> Result<Response, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(static_handler) = &self.static_handler {
            if req.uri.path().starts_with("/static") {
                return static_handler.handle(req).await;
            }
        }

        match self.router.handle_request(req).await {
            Ok(response) => Ok(response),
            Err(e) => {
                let app_error: AppError = e.into();
                (self.error_handler)(app_error)
            }
        }
    }
}
