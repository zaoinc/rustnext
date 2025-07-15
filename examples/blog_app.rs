use rustnext::*;
use rustnext::ui::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BlogPost {
    id: u32,
    title: String,
    content: String,
    author: String,
    created_at: String,
}

// Sample blog data
fn get_blog_posts() -> Vec<BlogPost> {
    vec![
        BlogPost {
            id: 1,
            title: "Welcome to RustNext Blog".to_string(),
            content: "This is our first blog post using the RustNext framework. RustNext allows you to build modern web applications using only Rust, without writing HTML directly.".to_string(),
            author: "RustNext Team".to_string(),
            created_at: "2024-01-15".to_string(),
        },
        BlogPost {
            id: 2,
            title: "Building Web Apps with Rust".to_string(),
            content: "Rust is becoming increasingly popular for web development. With RustNext, you can build full-stack applications using a familiar Next.js-like API but entirely in Rust.".to_string(),
            author: "Jane Developer".to_string(),
            created_at: "2024-01-16".to_string(),
        },
        BlogPost {
            id: 3,
            title: "The Future of Web Development".to_string(),
            content: "As web applications become more complex, having type safety and performance becomes crucial. RustNext provides both while maintaining developer productivity.".to_string(),
            author: "Tech Enthusiast".to_string(),
            created_at: "2024-01-17".to_string(),
        },
    ]
}

// Blog Layout Component
component!(blog_layout, props => {
    let title = props.get("title").and_then(|v| v.as_str()).unwrap_or("RustNext Blog");
    let children = props.get("children").cloned().unwrap_or_default();
    
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
                                .child(
                                    a()
                                        .prop("href", "/")
                                        .child(text("Home"))
                                )
                                .child(
                                    a()
                                        .prop("href", "/about")
                                        .child(text("About"))
                                )
                                .child(
                                    a()
                                        .prop("href", "/contact")
                                        .child(text("Contact"))
                                )
                        )
                )
        )
        .child(
            main()
                .class("main")
                .child(
                    div()
                        .class("container")
                        .child(
                            h1().child(text(title))
                        )
                        .child(
                            div().prop("innerHTML", children.to_string())
                        )
                )
        )
        .child(
            footer()
                .class("footer")
                .child(
                    div()
                        .class("container")
                        .child(text("© 2024 RustNext Blog. Built with Rust."))
                )
        )
});

// Blog Post Card Component
component!(blog_post_card, props => {
    let post_id = props.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
    let title = props.get("title").and_then(|v| v.as_str()).unwrap_or("Untitled");
    let content = props.get("content").and_then(|v| v.as_str()).unwrap_or("");
    let author = props.get("author").and_then(|v| v.as_str()).unwrap_or("Anonymous");
    let created_at = props.get("created_at").and_then(|v| v.as_str()).unwrap_or("");
    
    // Truncate content for preview
    let preview = if content.len() > 150 {
        format!("{}...", &content[..150])
    } else {
        content.to_string()
    };
    
    article()
        .class("card")
        .child(
            h2()
                .child(
                    a()
                        .prop("href", format!("/post/{}", post_id))
                        .child(text(title))
                )
        )
        .child(
            p().child(text(&preview))
        )
        .child(
            div()
                .child(
                    span()
                        .child(text(&format!("By {} on {}", author, created_at)))
                )
                .child(
                    a()
                        .class("btn")
                        .prop("href", format!("/post/{}", post_id))
                        .child(text("Read More"))
                )
        )
});

// Home Page
page!(HomePage, req => {
    let posts = get_blog_posts();
    let mut post_cards = Vec::new();
    
    for post in posts {
        let mut post_props = HashMap::new();
        post_props.insert("id".to_string(), json!(post.id));
        post_props.insert("title".to_string(), json!(post.title));
        post_props.insert("content".to_string(), json!(post.content));
        post_props.insert("author".to_string(), json!(post.author));
        post_props.insert("created_at".to_string(), json!(post.created_at));
        
        if let Some(card) = get_component_registry().render("blog_post_card", &post_props) {
            post_cards.push(card);
        }
    }
    
    let mut layout_props = HashMap::new();
    layout_props.insert("title".to_string(), json!("Latest Posts"));
    
    let content = section()
        .child(
            h2().child(text("Recent Blog Posts"))
        )
        .children(post_cards);
    
    layout_props.insert("children".to_string(), json!(get_renderer().render_to_html(&content)));
    
    get_component_registry().render("blog_layout", &layout_props).unwrap_or_else(|| {
        div().child(text("Error rendering page"))
    })
});

// Individual Post Page
page!(PostPage, req => {
    let post_id: u32 = req.param("id")
        .and_then(|id| id.parse().ok())
        .unwrap_or(1);
    
    let posts = get_blog_posts();
    let post = posts.iter().find(|p| p.id == post_id);
    
    let content = if let Some(post) = post {
        article()
            .class("card")
            .child(
                h1().child(text(&post.title))
            )
            .child(
                div()
                    .child(text(&format!("By {} on {}", post.author, post.created_at)))
            )
            .child(
                div()
                    .child(text(&post.content))
            )
            .child(
                div()
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
    layout_props.insert("title".to_string(), json!(
        post.map(|p| p.title.as_str()).unwrap_or("Post Not Found")
    ));
    layout_props.insert("children".to_string(), json!(get_renderer().render_to_html(&content)));
    
    get_component_registry().render("blog_layout", &layout_props).unwrap_or_else(|| {
        div().child(text("Error rendering page"))
    })
});

// About Page
page!(AboutPage, req => {
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
                )
        );
    
    let mut layout_props = HashMap::new();
    layout_props.insert("title".to_string(), json!("About"));
    layout_props.insert("children".to_string(), json!(get_renderer().render_to_html(&content)));
    
    get_component_registry().render("blog_layout", &layout_props).unwrap_or_else(|| {
        div().child(text("Error rendering page"))
    })
});

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Register components
    register_component!("blog_layout", blog_layout);
    register_component!("blog_post_card", blog_post_card);
    
    // Register pages
    register_page!("/", HomePage);
    register_page!("/post/:id", PostPage);
    register_page!("/about", AboutPage);
    
    // Create router with page handler
    let router = Router::new()
        .get("/", |req| async move {
            if let Some(element) = get_page_registry().render_page("/", &req) {
                get_renderer().render_to_response(&element)
            } else {
                Ok(Response::new().status(StatusCode::NOT_FOUND).text("Page not found"))
            }
        })
        .get("/post/:id", |req| async move {
            if let Some(element) = get_page_registry().render_page("/post/:id", &req) {
                get_renderer().render_to_response(&element)
            } else {
                Ok(Response::new().status(StatusCode::NOT_FOUND).text("Page not found"))
            }
        })
        .get("/about", |req| async move {
            if let Some(element) = get_page_registry().render_page("/about", &req) {
                get_renderer().render_to_response(&element)
            } else {
                Ok(Response::new().status(StatusCode::NOT_FOUND).text("Page not found"))
            }
        });

    // Create and run server
    let app = App::new().router(router);
    let addr: SocketAddr = "127.0.0.1:3000".parse()?;
    let server = Server::new(app, addr);
    
    println!("🚀 RustNext Blog running at http://127.0.0.1:3000");
    println!("📝 Available routes:");
    println!("   http://127.0.0.1:3000/        - Home page with blog posts");
    println!("   http://127.0.0.1:3000/post/1  - Individual blog post");
    println!("   http://127.0.0.1:3000/about   - About page");
    
    server.run().await
}
