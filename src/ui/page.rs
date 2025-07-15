use crate::ui::Element;
use crate::Request;
use serde_json::Value;
use std::collections::HashMap;
use once_cell::sync::OnceCell;
use tokio::sync::Mutex;
use async_trait::async_trait;

#[async_trait]
pub trait Page: Send + Sync {
    async fn render(&self, req: &Request) -> Element;
    fn get_props(&self, _req: &Request) -> HashMap<String, Value> {
        HashMap::new()
    }
}

pub struct PageRegistry {
    pages: HashMap<String, Box<dyn Page>>,
}

impl PageRegistry {
    pub fn new() -> Self {
        PageRegistry {
            pages: HashMap::new(),
        }
    }

    pub fn register<P>(&mut self, path: &str, page: P)
    where
        P: Page + 'static,
    {
        self.pages.insert(path.to_string(), Box::new(page));
    }

    pub async fn render_page(&self, path: &str, req: &Request) -> Option<Element> {
        if let Some(page) = self.pages.get(path) {
            Some(page.render(req).await)
        } else {
            None
        }
    }
}

static GLOBAL_PAGE_REGISTRY: OnceCell<Mutex<PageRegistry>> = OnceCell::new();

pub fn get_page_registry() -> &'static Mutex<PageRegistry> {
    GLOBAL_PAGE_REGISTRY.get_or_init(|| Mutex::new(PageRegistry::new()))
}

#[macro_export]
macro_rules! page {
    ($name:ident, $req:ident => $body:expr) => {
        pub struct $name;
        
        #[async_trait]
        impl crate::ui::Page for $name {
            async fn render(&self, $req: &crate::Request) -> crate::ui::Element {
                $body
            }
        }
    };
}

#[macro_export]
macro_rules! register_page {
    ($path:expr, $page_struct:ident) => {
        // This macro now expands to an async block that returns a Future
        async {
            let mut registry = crate::ui::get_page_registry().lock().await; // Removed .unwrap()
            registry.register($path, $page_struct {});
            Ok::<(), Box<dyn std::error::Error + Send + Sync>>(()) // Return a Result
        }
    };
}
