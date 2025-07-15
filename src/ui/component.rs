use crate::ui::Element;
use serde_json::Value;
use std::collections::HashMap;
use once_cell::sync::OnceCell;
use tokio::sync::Mutex;
use async_trait::async_trait;

#[async_trait]
pub trait Component: Send + Sync {
    async fn render(&self, props: &HashMap<String, Value>) -> Element;
}

pub struct ComponentRegistry {
    components: HashMap<String, Box<dyn Component>>,
}

impl ComponentRegistry {
    pub fn new() -> Self {
        ComponentRegistry {
            components: HashMap::new(),
        }
    }

    pub fn register<C>(&mut self, name: &str, component: C)
    where
        C: Component + 'static,
    {
        self.components.insert(name.to_string(), Box::new(component));
    }

    pub async fn render(&self, name: &str, props: &HashMap<String, Value>) -> Option<Element> {
        if let Some(component) = self.components.get(name) {
            Some(component.render(props).await)
        } else {
            None
        }
    }
}

static GLOBAL_REGISTRY: OnceCell<Mutex<ComponentRegistry>> = OnceCell::new();

pub fn get_component_registry() -> &'static Mutex<ComponentRegistry> {
    GLOBAL_REGISTRY.get_or_init(|| Mutex::new(ComponentRegistry::new()))
}

#[macro_export]
macro_rules! component {
    ($name:ident, $props:ident => $body:expr) => {
        pub struct $name;
        
        #[async_trait]
        impl crate::ui::Component for $name {
            async fn render(&self, $props: &std::collections::HashMap<String, serde_json::Value>) -> crate::ui::Element {
                $body
            }
        }
    };
}

#[macro_export]
macro_rules! register_component {
    ($name:expr, $component_struct:ident) => {
        // This macro now expands to an async block that returns a Future
        async {
            let mut registry = crate::ui::get_component_registry().lock().await; // Removed .unwrap()
            registry.register($name, $component_struct {});
            Ok::<(), Box<dyn std::error::Error + Send + Sync>>(()) // Return a Result
        }
    };
}
