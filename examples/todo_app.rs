use rustnext::*;
use rustnext::ui::{Element, div, header, nav, a, text, main as main_element, h1, form, input, button, section, h2, ul, li, span, get_component_registry, get_renderer};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Mutex, Arc};
use log::{info, error};
use once_cell::sync::Lazy;
use urlencoding;

// In-memory storage for todos
static TODOS: Lazy<Mutex<Vec<Todo>>> = Lazy::new(|| Mutex::new(vec![
    Todo {
        id: 1,
        task: "Learn RustNext".to_string(),
        completed: false,
    },
    Todo {
        id: 2,
        task: "Build a web app".to_string(),
        completed: true,
    },
    Todo {
        id: 3,
        task: "Explore Rust ecosystem".to_string(),
        completed: false,
    },
]));

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Todo {
    id: u32,
    task: String,
    completed: bool,
}

// API Handler for getting todos
struct GetTodosHandler;

#[async_trait]
impl ApiHandler for GetTodosHandler {
    async fn handle(&self, _req: Request) -> Result<ApiResponse, ApiError> {
        let todos = TODOS.lock().unwrap().clone();
        Ok(ApiResponse::ok(serde_json::to_value(todos).unwrap()))
    }
}

// API Handler for creating todos
struct CreateTodoHandler;

#[async_trait]
impl ApiHandler for CreateTodoHandler {
    async fn handle(&self, mut req: Request) -> Result<ApiResponse, ApiError> {
        let form_data = req.form().await.map_err(|e| ApiError::bad_request(&format!("Failed to parse form data: {}", e)))?;
        
        let task = form_data.get("task").map(|s| s.trim()).filter(|s| !s.is_empty());

        if task.is_none() {
            return Err(ApiError::bad_request("Task cannot be empty."));
        }

        let mut todos = TODOS.lock().unwrap();
        let new_id = todos.iter().map(|t| t.id).max().unwrap_or(0) + 1;

        let new_todo = Todo {
            id: new_id,
            task: task.unwrap().to_string(),
            completed: false,
        };
        todos.push(new_todo.clone());
        info!("New todo created: {:?}", new_todo);

        Ok(ApiResponse::ok(json!({"message": "Todo created successfully"}))
            .header("Location", "/")
            .header("HX-Redirect", "/") // For HTMX if used
            .with_status(StatusCode::SEE_OTHER))
    }
}

// API Handler for updating todo status
struct UpdateTodoHandler;

#[async_trait]
impl ApiHandler for UpdateTodoHandler {
    async fn handle(&self, req: Request) -> Result<ApiResponse, ApiError> {
        let todo_id: u32 = req.param("id")
            .and_then(|id| id.parse().ok())
            .ok_or_else(|| ApiError::bad_request("Invalid todo ID"))?;
        
        let mut todos = TODOS.lock().unwrap();
        if let Some(todo) = todos.iter_mut().find(|t| t.id == todo_id) {
            todo.completed = !todo.completed; // Toggle completion status
            info!("Todo {} updated: completed={}", todo_id, todo.completed);
            Ok(ApiResponse::ok(json!({"message": "Todo updated successfully", "todo": todo})))
        } else {
            Err(ApiError::not_found(&format!("Todo with ID {} not found", todo_id)))
        }
    }
}

// API Handler for deleting todos
struct DeleteTodoHandler;

#[async_trait]
impl ApiHandler for DeleteTodoHandler {
    async fn handle(&self, req: Request) -> Result<ApiResponse, ApiError> {
        let todo_id: u32 = req.param("id")
            .and_then(|id| id.parse().ok())
            .ok_or_else(|| ApiError::bad_request("Invalid todo ID"))?;
        
        let mut todos = TODOS.lock().unwrap();
        let initial_len = todos.len();
        todos.retain(|t| t.id != todo_id);

        if todos.len() < initial_len {
            info!("Todo with ID {} deleted", todo_id);
            Ok(ApiResponse::ok(json!({"message": "Todo deleted successfully"})))
        } else {
            Err(ApiError::not_found(&format!("Todo with ID {} not found", todo_id)))
        }
    }
}


// Todo Layout Component
component!(TodoLayout, props => {
    let title = props.get("title").and_then(|v| v.as_str()).unwrap_or("RustNext Todo App");
    let children_html = props.get("children_html").and_then(|v| v.as_str()).unwrap_or("");
    let error_message = props.get("error_message").and_then(|v| v.as_str()).unwrap_or("");
    
    div()
        .child(
            header()
                .class("header")
                .child(
                    div()
                        .class("container")
                        .child(
                            nav()
                                .class("nav")
                                .child(a().prop("href", "/").child(text("Todos")))
                                .child(a().prop("href", "/about").child(text("About")))
                        )
                )
        )
        .child(
            main_element()
                .class("main")
                .child(
                    div()
                        .class("container")
                        .child(h1().child(text(title)))
                        .child(
                            if !error_message.is_empty() {
                                div()
                                    .class("error-message")
                                    .child(text(error_message))
                            } else {
                                div()
                            }
                        )
                        .child(
                            // Render children HTML directly using the _raw_html prop
                            div().prop("_raw_html", children_html)
                        )
                )
        )
        .child(
            footer()
                .class("footer")
                .child(
                    div()
                        .class("container")
                        .child(text("© 2024 RustNext Todo App. Built with Rust."))
                )
        )
});

// Todo Item Component
component!(TodoItem, props => {
    let id = props.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
    let task = props.get("task").and_then(|v| v.as_str()).unwrap_or("Untitled Task");
    let completed = props.get("completed").and_then(|v| v.as_bool()).unwrap_or(false);

    let task_class = if completed { "line-through text-gray-500" } else { "text-gray-800" };

    li()
        .class("flex items-center justify-between p-3 border-b border-gray-200 last:border-b-0")
        .child(
            div()
                .class("flex items-center")
                .child(
                    a() // Link to toggle completion
                        .prop("href", format!("/api/todos/{}/toggle", id))
                        .prop("method", "POST") // Use POST for state change
                        .class("mr-3")
                        .child(
                            input()
                                .prop("type", "checkbox")
                                .prop("checked", completed)
                                .prop("disabled", "true") // Disable direct interaction, link handles it
                                .class("form-checkbox h-5 w-5 text-blue-600")
                        )
                )
                .child(
                    span()
                        .class(task_class)
                        .child(text(task))
                )
        )
        .child(
            a() // Link to delete todo
                .prop("href", format!("/api/todos/{}/delete", id))
                .prop("method", "POST") // Use POST for state change
                .class("text-red-500 hover:text-red-700 text-sm")
                .child(text("Delete"))
        )
});

// Todo Form Component
component!(TodoForm, _props => {
    form()
        .prop("method", "POST")
        .prop("action", "/api/todos")
        .class("mt-4 p-4 border border-gray-200 rounded-md bg-gray-50")
        .child(
            div()
                .class("form-group")
                .child(
                    input()
                        .prop("type", "text")
                        .prop("name", "task")
                        .prop("placeholder", "Add a new todo...")
                        .prop("required", "true")
                        .class("form-control")
                )
        )
        .child(
            button()
                .prop("type", "submit")
                .class("btn mt-2")
                .child(text("Add Todo"))
        )
});

// Home Page for Todos
page!(HomePage, req => {
    let todos_cloned = {
        TODOS.lock().unwrap().clone()
    };

    let mut todo_items = Vec::new();
    for todo in todos_cloned {
        let mut todo_props = HashMap::new();
        todo_props.insert("id".to_string(), json!(todo.id));
        todo_props.insert("task".to_string(), json!(todo.task));
        todo_props.insert("completed".to_string(), json!(todo.completed));
        
        let component_registry = get_component_registry().lock().await;
        if let Some(item) = component_registry.render("todo_item", &todo_props).await {
            todo_items.push(item);
        }
    }

    let todo_list_section = section()
        .class("card mt-4")
        .child(h2().class("text-xl font-bold mb-4").child(text("My Todos")))
        .child(
            ul().children(todo_items)
        );

    let todo_form_element = {
        let component_registry = get_component_registry().lock().await;
        component_registry.render("todo_form", &HashMap::new()).await.unwrap_or_else(|| {
            div().child(text("Error rendering todo form"))
        })
    };

    let content = div()
        .child(todo_form_element)
        .child(todo_list_section);

    let mut layout_props = HashMap::new();
    layout_props.insert("title".to_string(), json!("Todo List"));
    layout_props.insert("children_html".to_string(), json!(get_renderer().render_to_html(&content)));

    // Check for a query parameter indicating an error after redirect
    if let Some(error_msg) = req.query_param("error") {
        layout_props.insert("error_message".to_string(), json!(urlencoding::decode(error_msg).unwrap_or_default()));
    }

    let component_registry = get_component_registry().lock().await;
    let rendered_element = component_registry.render("todo_layout", &layout_props).await;
    rendered_element.unwrap_or_else(|| {
        div().child(text("Error rendering page"))
    })
});

// About Page for Todo App
page!(AboutPage, _req => {
    let content = section()
        .child(
            h2().child(text("About This Todo App"))
        )
        .child(
            div()
                .class("card")
                .child(
                    p().child(text("This is a simple Todo List application built with RustNext."))
                )
                .child(
                    p().child(text("It demonstrates basic CRUD operations using RustNext's component system, routing, and API handlers."))
                )
                .child(
                    h3().child(text("Key Features Demonstrated:"))
                )
                .child(
                    ul()
                        .child(li().child(text("UI Components for Todo Items and Form")))
                        .child(li().child(text("API Endpoints for managing Todos (GET, POST, PUT, DELETE)")))
                        .child(li().child(text("In-memory data storage (for simplicity)")))
                        .child(li().child(text("Form submission and POST-redirect-GET pattern")))
                )
        );

    let mut layout_props = HashMap::new();
    layout_props.insert("title".to_string(), json!("About Todo App"));
    layout_props.insert("children_html".to_string(), json!(get_renderer().render_to_html(&content)));

    let component_registry = get_component_registry().lock().await;
    let rendered_element = component_registry.render("todo_layout", &layout_props).await;
    rendered_element.unwrap_or_else(|| {
        div().child(text("Error rendering page"))
    })
});


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize logging first
    init_logging();

    // Initialize configuration from file (if exists) or environment
    let config = Config::load(Some("config.toml"));
    init_config(config.clone());

    info!("🔧 Configuration loaded:");
    info!("   Server: {}:{}", config.server.host, config.server.port);
    info!("   Features: compression={}, metrics={}, logging={}", config.features.compression, config.features.metrics, config.features.logging);
    
    // Register components
    register_component!("todo_layout", TodoLayout).await?;
    register_component!("todo_item", TodoItem).await?;
    register_component!("todo_form", TodoForm).await?;
    
    // Register pages
    register_page!("/", HomePage).await?;
    register_page!("/about", AboutPage).await?;

    // Register API routes
    api_route!(hyper::Method::GET, "/api/todos", GetTodosHandler).await?;
    api_route!(hyper::Method::POST, "/api/todos", CreateTodoHandler).await?;
    api_route!(hyper::Method::POST, "/api/todos/:id/toggle", UpdateTodoHandler).await?; // Using POST for toggle
    api_route!(hyper::Method::POST, "/api/todos/:id/delete", DeleteTodoHandler).await?; // Using POST for delete

    // Create router
    let router = Router::new()
        .get("/", |req| async move {
            let page_registry = get_page_registry().lock().await;
            let element_option = page_registry.render_page("/", &req).await;
            if let Some(element) = element_option {
                get_renderer().render_to_response(&element)
            } else {
                Err(Box::new(AppError::NotFound("Home page not found".to_string())) as Box<dyn std::error::Error + Send + Sync>)
            }
        })
        .get("/about", |req| async move {
            let page_registry = get_page_registry().lock().await;
            let element_option = page_registry.render_page("/about", &req).await;
            if let Some(element) = element_option {
                get_renderer().render_to_response(&element)
            } else {
                Err(Box::new(AppError::NotFound("About page not found".to_string())) as Box<dyn std::error::Error + Send + Sync>)
            }
        })
        .post("/api/todos", |req| async move {
            let api_registry = get_api_registry().lock().await;
            match api_registry.handle_request(req).await {
                Some(response) => {
                    // If API call was successful and resulted in a redirect, return it
                    if response.status == StatusCode::SEE_OTHER {
                        Ok(response)
                    } else {
                        // If API call failed, extract error and redirect back to home with error message
                        let body_bytes = hyper::body::to_bytes(response.body).await.map_err(|e| Box::new(AppError::Internal(e.to_string())) as Box<dyn std::error::Error + Send + Sync>)?;
                        let error_json: serde_json::Value = serde_json::from_slice(&body_bytes).map_err(|e| Box::new(AppError::BadRequest(e.to_string())) as Box<dyn std::error::Error + Send + Sync>)?;
                        let error_msg = error_json["error"].as_str().unwrap_or("Unknown error").to_string();
                        Ok(Response::new()
                            .status(StatusCode::SEE_OTHER)
                            .header("Location", format!("/?error={}", urlencoding::encode(&error_msg)))
                            .header("HX-Redirect", format!("/?error={}", urlencoding::encode(&error_msg))))
                    }
                }
                None => Err(Box::new(AppError::NotFound("API endpoint /api/todos (POST) not found".to_string())) as Box<dyn std::error::Error + Send + Sync>),
            }
        })
        .post("/api/todos/:id/toggle", |req| async move {
            let api_registry = get_api_registry().lock().await;
            match api_registry.handle_request(req).await {
                Some(response) => {
                    // After toggle, redirect back to home to show updated list
                    Ok(Response::new().status(StatusCode::SEE_OTHER).header("Location", "/"))
                }
                None => Err(Box::new(AppError::NotFound("API endpoint /api/todos/:id/toggle (POST) not found".to_string())) as Box<dyn std::error::Error + Send + Sync>),
            }
        })
        .post("/api/todos/:id/delete", |req| async move {
            let api_registry = get_api_registry().lock().await;
            match api_registry.handle_request(req).await {
                Some(response) => {
                    // After delete, redirect back to home to show updated list
                    Ok(Response::new().status(StatusCode::SEE_OTHER).header("Location", "/"))
                }
                None => Err(Box::new(AppError::NotFound("API endpoint /api/todos/:id/delete (POST) not found".to_string())) as Box<dyn std::error::Error + Send + Sync>),
            }
        });

    // Define a custom error handler for the App
    let custom_error_handler = Arc::new(|err: AppError| {
        error!("Application Error: {}", err);
        err.into_response()
    });

    // Create and run server
    let app = App::new()
        .router(router)
        .error_handler(custom_error_handler);

    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;
    let server = Server::new(app, addr);
    
    info!("🚀 RustNext Todo App running at http://{}:{}", config.server.host, config.server.port);
    info!("📝 Available routes:");
    info!("   http://{}:{}/           - Todo list home page", config.server.host, config.server.port);
    info!("   http://{}:{}/about      - About page", config.server.host, config.server.port);
    info!("   http://{}:{}/api/todos  - API endpoints (GET/POST/PUT/DELETE)", config.server.host, config.server.port);
    info!("");
    info!("🎯 Features Demonstrated:");
    info!("   ✅ UI Components for Todo Items and Form");
    info!("   ✅ API Endpoints for managing Todos (GET, POST, PUT, DELETE)");
    info!("   ✅ In-memory data storage (for simplicity)");
    info!("   ✅ Form submission and POST-redirect-GET pattern");
    
    server.run().await
}
