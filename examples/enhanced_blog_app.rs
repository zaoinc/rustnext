use rustnext::*;
use rustnext::ui::{Element, div, header, nav, a, text, main as main_element, h1, form, input, button, section, h2, ul, li, span, article, p, get_component_registry, get_renderer};
use rustnext::middleware::auth_guard::RateLimiter;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Mutex, Arc};
use log::{info, error};
use once_cell::sync::Lazy;
use urlencoding;

// In-memory storage for blog posts (for demonstration without a database)
static BLOG_POSTS: Lazy<Mutex<Vec<BlogPost>>> = Lazy::new(|| Mutex::new(vec![
    BlogPost {
        id: 1,
        title: "Welcome to Enhanced RustNext Blog".to_string(),
        content: "This enhanced blog now includes forms, API routes, configuration management, asset optimization, and advanced middleware! This is a longer piece of content to demonstrate how the blog post card will display more text. We are excited about the future of web development with Rust and believe RustNext provides a solid foundation for building high-performance, reliable web applications. Stay tuned for more updates and features!".to_string(),
        author: "RustNext Team".to_string(),
        created_at: "2024-01-15".to_string(),
    },
    BlogPost {
        id: 2,
        title: "New Features in RustNext".to_string(),
        content: "We've added 5 major features: Form handling with validation, API routes, Environment configuration, Asset management with optimization, and Advanced middleware system. Each of these features is designed to make building robust web applications in Rust easier and more efficient. We've focused on providing a developer experience that is both powerful and intuitive, allowing you to leverage Rust's strengths without sacrificing productivity.".to_string(),
        author: "Developer".to_string(),
        created_at: "2024-01-16".to_string(),
    },
]));

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BlogPost {
    id: u32,
    title: String,
    content: String,
    author: String,
    created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreatePostRequest {
    title: String,
    content: String,
    author: String,
}

// API Handler for creating posts
struct CreatePostHandler;

#[async_trait]
impl ApiHandler for CreatePostHandler {
    async fn handle(&self, mut req: Request) -> Result<ApiResponse, ApiError> {
        let content_type = req.headers.get(hyper::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        let form_data = if content_type.starts_with("application/x-www-form-urlencoded") {
            req.form().await.map_err(|e| ApiError::bad_request(&format!("Failed to parse form data: {}", e)))?
        } else if content_type.starts_with("multipart/form-data") {
            return Err(ApiError::bad_request("Multipart form data not fully supported in this example. Please use application/x-www-form-urlencoded."));
        } else {
            return Err(ApiError::bad_request("Unsupported Content-Type for post creation."));
        };

        let title = form_data.get("title").map(|s| s.trim()).filter(|s| !s.is_empty());
        let content = form_data.get("content").map(|s| s.trim()).filter(|s| !s.is_empty());
        let author = form_data.get("author").map(|s| s.trim()).filter(|s| !s.is_empty());

        if title.is_none() || content.is_none() || author.is_none() {
            return Err(ApiError::bad_request("Title, content, and author are required."));
        }

        let mut posts = BLOG_POSTS.lock().unwrap();
        let new_id = posts.iter().map(|p| p.id).max().unwrap_or(0) + 1;

        let new_post = BlogPost {
            id: new_id,
            title: title.unwrap().to_string(),
            content: content.unwrap().to_string(),
            author: author.unwrap().to_string(),
            created_at: chrono::Utc::now().format("%Y-%m-%d").to_string(),
        };
        posts.push(new_post.clone());
        info!("New post created: {:?}", new_post);

        Ok(ApiResponse::ok(json!({"message": "Post created successfully"}))
            .header("Location", "/")
            .header("HX-Redirect", "/")
            .with_status(StatusCode::SEE_OTHER))
    }
}

// API Handler for getting posts
struct GetPostsHandler;

#[async_trait]
impl ApiHandler for GetPostsHandler {
    async fn handle(&self, _req: Request) -> Result<ApiResponse, ApiError> {
        let posts = BLOG_POSTS.lock().unwrap().clone();
        Ok(ApiResponse::ok(serde_json::to_value(posts).unwrap()))
    }
}

// Enhanced Blog Layout with form
component!(EnhancedBlogLayout, props => {
    let title = props.get("title").and_then(|v| v.as_str()).unwrap_or("Enhanced RustNext Blog");
    let show_form = props.get("show_form").and_then(|v| v.as_bool()).unwrap_or(false);
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
                                .child(a().prop("href", "/").child(text("Home")))
                                .child(a().prop("href", "/create").child(text("Create Post")))
                                .child(a().prop("href", "/api/posts").child(text("API")))
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
                            if show_form {
                                div()
                                    .class("card")
                                    .child(h2().child(text("Create New Post")))
                                    .child(
                                        form()
                                            .prop("method", "POST")
                                            .prop("action", "/api/posts")
                                            .child(
                                                div()
                                                    .class("form-group")
                                                    .child(
                                                        input()
                                                            .prop("type", "text")
                                                            .prop("name", "title")
                                                            .prop("placeholder", "Post title")
                                                            .prop("required", "true")
                                                            .class("form-control")
                                                    )
                                            )
                                            .child(
                                                div()
                                                    .class("form-group")
                                                    .child(
                                                        input()
                                                            .prop("type", "text")
                                                            .prop("name", "author")
                                                            .prop("placeholder", "Author name")
                                                            .prop("required", "true")
                                                            .class("form-control")
                                                    )
                                            )
                                            .child(
                                                div()
                                                    .class("form-group")
                                                    .child(
                                                        Element::new("textarea")
                                                            .prop("name", "content")
                                                            .prop("placeholder", "Post content")
                                                            .prop("required", "true")
                                                            .prop("rows", "5")
                                                            .class("form-control")
                                                    )
                                            )
                                            .child(
                                                button()
                                                    .prop("type", "submit")
                                                    .class("btn")
                                                    .child(text("Create Post"))
                                            )
                                    )
                            } else {
                                div() // Empty div if form is not shown
                            }
                        )
                        .child(
                            // Render children HTML directly using the new _raw_html prop
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
                        .child(text("© 2024 Enhanced RustNext Blog. Built with Rust + 5 New Features!"))
                )
        )
});

// Blog Post Card Component
component!(BlogPostCard, props => {
    let post_id = props.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
    let title = props.get("title").and_then(|v| v.as_str()).unwrap_or("Untitled");
    let content = props.get("content").and_then(|v| v.as_str()).unwrap_or("");
    let author = props.get("author").and_then(|v| v.as_str()).unwrap_or("Anonymous");
    let created_at = props.get("created_at").and_then(|v| v.as_str()).unwrap_or("");
    
    // Truncate content for preview (increased length)
    let preview = if content.len() > 300 {
        format!("{}...", &content[..300])
    } else {
        content.to_string()
    };
    
    article()
        .class("card mb-6") // Added margin-bottom for separation
        .child(
            h2()
                .child(
                    a()
                        .prop("href", format!("/post/{}", post_id))
                        .child(text(title))
                )
        )
        .child(
            p().class("text-md text-gray-800 font-semibold mt-1") // Larger, darker, bold author
                .child(text(&format!("By {}", author)))
        )
        .child(
            p().class("text-gray-700 mt-2") // Clearer content preview
                .child(text(&preview))
        )
        .child(
            div()
                .class("flex justify-between items-center mt-4") // Flexbox for alignment
                .child(
                    span()
                        .class("text-sm text-gray-600") // Slightly larger date
                        .child(text(&format!("Published on {}", created_at)))
                )
                .child(
                    a()
                        .class("btn")
                        .prop("href", format!("/post/{}", post_id))
                        .child(text("Read More"))
                )
        )
});

// Create Post Page
page!(CreatePostPage, req => {
    let mut layout_props = HashMap::new();
    layout_props.insert("title".to_string(), json!("Create New Post"));
    layout_props.insert("show_form".to_string(), json!(true));
    layout_props.insert("children_html".to_string(), json!(""));

    // Check for a query parameter indicating an error after redirect
    if let Some(error_msg) = req.query_param("error") {
        layout_props.insert("error_message".to_string(), json!(urlencoding::decode(error_msg).unwrap_or_default()));
    }
    
    let component_registry = get_component_registry().lock().await;
    let rendered_element = component_registry.render("enhanced_blog_layout", &layout_props).await;
    rendered_element.unwrap_or_else(|| {
        div().child(text("Error rendering page"))
    })
});

// Individual Post Page
page!(PostPage, req => {
    let post_id: u32 = req.param("id")
        .and_then(|id| id.parse().ok())
        .unwrap_or(1);
    
    // Acquire lock and clone the post within a separate scope
    // This ensures the MutexGuard is dropped before any subsequent .await calls
    let post_clone = {
        let posts = BLOG_POSTS.lock().unwrap();
        posts.iter().find(|p| p.id == post_id).cloned()
    }; // MutexGuard `posts` is dropped here

    let content = if let Some(post) = post_clone.clone() { // Use the cloned post
        article()
            .class("card")
            .child(
                h1().child(text(&post.title))
            )
            .child(
                p().class("text-md text-gray-800 font-semibold mt-1") // Larger, darker, bold author
                    .child(text(&format!("By {}", post.author)))
            )
            .child(
                p().class("text-sm text-gray-600 mt-1") // Slightly larger date
                    .child(text(&format!("Published on {}", post.created_at)))
            )
            .child(
                p() // Changed div to p for semantic correctness for content
                    .class("text-lg text-gray-800 leading-relaxed mt-4") // Larger, darker content with more line height
                    .child(text(&post.content)) // Full content
            )
            .child(
                div()
                    .class("mt-6") // Add some margin
                    .child(
                        a()
                            .class("btn")
                            .prop("href", "/")
                            .child(text("← Back to Home"))
                    )
            )
    } else {
        div()
            .class("card")
            .child(
                h1().child(text("Post Not Found"))
            )
            .child(
                p().child(text("The requested blog post could not be found."))
            )
            .child(
                a()
                    .class("btn")
                    .prop("href", "/")
                    .child(text("← Back to Home"))
            )
    };
    
    let mut layout_props = HashMap::new();
    // Clone the title string to ensure it's owned
    layout_props.insert("title".to_string(), json!(
        post_clone.map(|p| p.title).unwrap_or_else(|| "Post Not Found".to_string())
    ));
    layout_props.insert("children_html".to_string(), json!(get_renderer().render_to_html(&content)));
    layout_props.insert("show_form".to_string(), json!(false));
    
    let component_registry = get_component_registry().lock().await;
    let rendered_element = component_registry.render("enhanced_blog_layout", &layout_props).await;
    rendered_element.unwrap_or_else(|| {
        div().child(text("Error rendering page"))
    })
});

// About Page
page!(AboutPage, _req => {
    let content = section()
        .child(
            h2().child(text("About RustNext Blog"))
        )
        .child(
            div()
                .class("card")
                .child(
                    p().child(text("Welcome to our blog built with RustNext! This demonstrates how you can create modern web applications using only Rust."))
                )
                .child(
                    p().child(text("RustNext provides a Next.js-like experience but with the power and safety of Rust. No HTML required - everything is built using Rust components."))
                )
                .child(
                    h3().child(text("Features:"))
                )
                .child(
                    ul()
                        .child(li().child(text("Type-safe component system")))
                        .child(li().child(text("Server-side rendering")))
                        .child(li().child(text("File-based routing")))
                        .child(li().child(text("Built-in styling")))
                        .child(li().child(text("No HTML required")))
                        .child(li().child(text("✅ Enhanced Request Body Parsing (Forms)")))
                        .child(li().child(text("✅ Custom Error Handling & Pages")))
                        .child(li().child(text("✅ File-based Configuration (config.toml)")))
                        .child(li().child(text("✅ Structured Logging")))
                        .child(li().child(text("✅ Safe Global State Management (once_cell)")))
                )
        );
    
    let mut layout_props = HashMap::new();
    layout_props.insert("title".to_string(), json!("About"));
    layout_props.insert("children_html".to_string(), json!(get_renderer().render_to_html(&content)));
    layout_props.insert("show_form".to_string(), json!(false));
    
    let component_registry = get_component_registry().lock().await;
    let rendered_element = component_registry.render("enhanced_blog_layout", &layout_props).await;
    rendered_element.unwrap_or_else(|| {
        div().child(text("Error rendering page"))
    })
});

// Enhanced Home Page with API integration info
page!(EnhancedHomePage, _req => {
    // Acquire lock and clone the entire Vec<BlogPost> within a separate scope
    // This ensures the MutexGuard is dropped before any subsequent .await calls
    let posts_cloned = {
        BLOG_POSTS.lock().unwrap().clone()
    }; // MutexGuard is dropped here
    
    let mut post_cards = Vec::new();

    for post in posts_cloned { // Iterate over the cloned Vec
        let mut post_props = HashMap::new();
        post_props.insert("id".to_string(), json!(post.id));
        post_props.insert("title".to_string(), json!(post.title));
        post_props.insert("content".to_string(), json!(post.content));
        post_props.insert("author".to_string(), json!(post.author));
        post_props.insert("created_at".to_string(), json!(post.created_at));
        
        let component_registry = get_component_registry().lock().await;
        if let Some(card) = component_registry.render("blog_post_card", &post_props).await {
            post_cards.push(card);
        }
    }

    let content = section()
        .child(h2().child(text("Enhanced Blog Features")))
        .child(
            div()
                .class("card")
                .child(h3().child(text("New Features Added:")))
                .child(
                    ul()
                        .child(li().child(text("✅ Form Handling & Validation (via Request::form())")))
                        .child(li().child(text("✅ API Routes with POST-redirect-GET pattern")))
                        .child(li().child(text("✅ File-based Configuration (config.toml)")))
                        .child(li().child(text("✅ Custom Error Handling & Pages")))
                        .child(li().child(text("✅ Structured Logging (check console output)")))
                        .child(li().child(text("✅ Safe Global State Management (once_cell)")))
                )
        )
        .child(h2().child(text("Recent Blog Posts")))
        .children(post_cards);

    let mut layout_props = HashMap::new();
    layout_props.insert("title".to_string(), json!("Enhanced RustNext Blog"));
    layout_props.insert("show_form".to_string(), json!(false));
    layout_props.insert("children_html".to_string(), json!(get_renderer().render_to_html(&content)));
    
    let component_registry = get_component_registry().lock().await;
    let rendered_element = component_registry.render("enhanced_blog_layout", &layout_props).await;
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
    info!("   Blog Name: {}", config.custom.get("blog_name").unwrap_or(&"Default Blog".to_string()));

    // Register components
    register_component!("enhanced_blog_layout", EnhancedBlogLayout).await?;
    register_component!("blog_post_card", BlogPostCard).await?;
    
    // Register pages
    register_page!("/", EnhancedHomePage).await?;
    register_page!("/create", CreatePostPage).await?;
    register_page!("/post/:id", PostPage).await?;
    register_page!("/about", AboutPage).await?;

    // Register API routes
    api_route!(hyper::Method::GET, "/api/posts", GetPostsHandler).await?;
    api_route!(hyper::Method::POST, "/api/posts", CreatePostHandler).await?;

    // Create asset manager
    let asset_manager = AssetManager::new("assets");

    // Define a custom error handler for the App
    let custom_error_handler = Arc::new(|err: AppError| {
        error!("Application Error: {}", err);
        err.into_response()
    });

    // Create router with all features
    let router = Router::new()
        .use_middleware(RateLimiter::new(100, 60))
        .get("/", |req| async move {
            let page_registry = get_page_registry().lock().await;
            let element_option = page_registry.render_page("/", &req).await;
            if let Some(element) = element_option {
                get_renderer().render_to_response(&element)
            } else {
                Err(Box::new(AppError::NotFound("Home page not found".to_string())) as Box<dyn std::error::Error + Send + Sync>)
            }
        })
        .get("/create", |req| async move {
            let page_registry = get_page_registry().lock().await;
            let element_option = page_registry.render_page("/create", &req).await;
            if let Some(element) = element_option {
                get_renderer().render_to_response(&element)
            } else {
                Err(Box::new(AppError::NotFound("Create post page not found".to_string())) as Box<dyn std::error::Error + Send + Sync>)
            }
        })
        .get("/post/:id", |req| async move {
            let page_registry = get_page_registry().lock().await;
            let element_option = page_registry.render_page("/post/:id", &req).await;
            if let Some(element) = element_option {
                get_renderer().render_to_response(&element)
            } else {
                Err(Box::new(AppError::NotFound(format!("Post {} not found", req.param("id").unwrap_or(&"unknown".to_string())))) as Box<dyn std::error::Error + Send + Sync>)
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
        .get("/api/posts", |req| async move {
            let api_registry = get_api_registry().lock().await;
            if let Some(response) = api_registry.handle_request(req).await {
                Ok(response)
            } else {
                Err(Box::new(AppError::NotFound("API endpoint /api/posts (GET) not found".to_string())) as Box<dyn std::error::Error + Send + Sync>)
            }
        })
        .post("/api/posts", |req| async move {
            let api_registry = get_api_registry().lock().await;
            match api_registry.handle_request(req).await {
                Some(response) => {
                    if response.status == StatusCode::SEE_OTHER {
                        Ok(response)
                    } else {
                        let body_bytes = hyper::body::to_bytes(response.body).await.map_err(|e| Box::new(AppError::Internal(e.to_string())) as Box<dyn std::error::Error + Send + Sync>)?;
                        let error_json: serde_json::Value = serde_json::from_slice(&body_bytes).map_err(|e| Box::new(AppError::BadRequest(e.to_string())) as Box<dyn std::error::Error + Send + Sync>)?;
                        let error_msg = error_json["error"].as_str().unwrap_or("Unknown error").to_string();
                        Ok(Response::new()
                            .status(StatusCode::SEE_OTHER)
                            .header("Location", format!("/create?error={}", urlencoding::encode(&error_msg)))
                            .header("HX-Redirect", format!("/create?error={}", urlencoding::encode(&error_msg))))
                    }
                }
                None => Err(Box::new(AppError::NotFound("API endpoint /api/posts (POST) not found".to_string())) as Box<dyn std::error::Error + Send + Sync>),
            }
        })
        .get("/assets/*", move |req| {
            let asset_manager = asset_manager.clone();
            async move {
                asset_manager.handle(req).await
            }
        });

    // Create and run server
    let app = App::new()
        .router(router)
        .error_handler(custom_error_handler);

    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;
    let server = Server::new(app, addr);
    
    info!("🚀 Enhanced RustNext Blog running at http://{}:{}", config.server.host, config.server.port);
    info!("📝 Available routes:");
    info!("   http://{}:{}/           - Enhanced home page", config.server.host, config.server.port);
    info!("   http://{}:{}/create     - Create new post form", config.server.host, config.server.port);
    info!("   http://{}:{}/post/1     - Individual blog post", config.server.host, config.server.port);
    info!("   http://{}:{}/about      - About page", config.server.host, config.server.port);
    info!("   http://{}:{}/api/posts  - API endpoints (GET/POST)", config.server.host, config.server.port);
    info!("   http://{}:{}/assets/*   - Optimized static assets", config.server.host, config.server.port);
    info!("");
    info!("🎯 New Features Demonstrated:");
    info!("   ✅ Form Handling & Validation (via Request::form())");
    info!("   ✅ API Routes with POST-redirect-GET pattern");
    info!("   ✅ File-based Configuration (config.toml)");
    info!("   ✅ Custom Error Handling & Pages (try /non-existent-page)");
    info!("   ✅ Structured Logging (check console output)");
    info!("   ✅ Safe Global State Management (once_cell)");
    
    server.run().await
}
