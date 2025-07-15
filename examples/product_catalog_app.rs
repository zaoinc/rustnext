use rustnext::*;
use rustnext::ui::{Element, div, header, nav, a, text, main as main_element, h1, form, input, button, section, h2, ul, li, span, article, p, label, get_component_registry, get_renderer};
use rustnext::middleware::auth_guard::RateLimiter;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Mutex, Arc};
use log::{info, error};
use once_cell::sync::Lazy;
use urlencoding;

// In-memory storage for products
static PRODUCTS: Lazy<Mutex<Vec<Product>>> = Lazy::new(|| Mutex::new(vec![
    Product {
        id: 1,
        name: "RustNext T-Shirt".to_string(),
        description: "High-quality cotton t-shirt featuring the RustNext logo. Perfect for developers!".to_string(),
        price: 24.99,
        category: "Apparel".to_string(),
        created_at: "2024-01-01".to_string(),
    },
    Product {
        id: 2,
        name: "Rust Programming Book".to_string(),
        description: "A comprehensive guide to learning Rust, from basics to advanced topics.".to_string(),
        price: 49.99,
        category: "Books".to_string(),
        created_at: "2024-01-05".to_string(),
    },
    Product {
        id: 3,
        name: "Mechanical Keyboard (Rust Edition)".to_string(),
        description: "A durable mechanical keyboard with custom Rust-themed keycaps and switches.".to_string(),
        price: 129.99,
        category: "Electronics".to_string(),
        created_at: "2024-01-10".to_string(),
    },
]));

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Product {
    id: u32,
    name: String,
    description: String,
    price: f64,
    category: String,
    created_at: String,
}

// API Handler for getting all products
struct GetProductsHandler;

#[async_trait]
impl ApiHandler for GetProductsHandler {
    async fn handle(&self, _req: Request) -> Result<ApiResponse, ApiError> {
        let products = PRODUCTS.lock().unwrap().clone();
        Ok(ApiResponse::ok(serde_json::to_value(products).unwrap()))
    }
}

// API Handler for getting a single product
struct GetProductHandler;

#[async_trait]
impl ApiHandler for GetProductHandler {
    async fn handle(&self, req: Request) -> Result<ApiResponse, ApiError> {
        let product_id: u32 = req.param("id")
            .and_then(|id| id.parse().ok())
            .ok_or_else(|| ApiError::bad_request("Invalid product ID"))?;

        let products = PRODUCTS.lock().unwrap();
        if let Some(product) = products.iter().find(|p| p.id == product_id) {
            Ok(ApiResponse::ok(serde_json::to_value(product).unwrap()))
        } else {
            Err(ApiError::not_found(&format!("Product with ID {} not found", product_id)))
        }
    }
}

// API Handler for creating products
struct CreateProductHandler;

#[async_trait]
impl ApiHandler for CreateProductHandler {
    async fn handle(&self, mut req: Request) -> Result<ApiResponse, ApiError> {
        let form_data = req.form().await.map_err(|e| ApiError::bad_request(&format!("Failed to parse form data: {}", e)))?;
        
        let name = form_data.get("name").map(|s| s.trim()).filter(|s| !s.is_empty());
        let description = form_data.get("description").map(|s| s.trim()).filter(|s| !s.is_empty());
        let price_str = form_data.get("price").map(|s| s.trim()).filter(|s| !s.is_empty());
        let category = form_data.get("category").map(|s| s.trim()).filter(|s| !s.is_empty());

        let price: f64 = price_str
            .ok_or_else(|| ApiError::bad_request("Price is required."))?
            .parse()
            .map_err(|_| ApiError::bad_request("Invalid price format."))?;

        if name.is_none() || description.is_none() || category.is_none() {
            return Err(ApiError::bad_request("Name, description, price, and category are required."));
        }

        let mut products = PRODUCTS.lock().unwrap();
        let new_id = products.iter().map(|p| p.id).max().unwrap_or(0) + 1;

        let new_product = Product {
            id: new_id,
            name: name.unwrap().to_string(),
            description: description.unwrap().to_string(),
            price,
            category: category.unwrap().to_string(),
            created_at: chrono::Utc::now().format("%Y-%m-%d").to_string(),
        };
        products.push(new_product.clone());
        info!("New product created: {:?}", new_product);

        Ok(ApiResponse::ok(json!({"message": "Product created successfully", "product_id": new_product.id}))
            .header("Location", "/")
            .header("HX-Redirect", "/")
            .with_status(StatusCode::SEE_OTHER))
    }
}

// API Handler for updating products
struct UpdateProductHandler;

#[async_trait]
impl ApiHandler for UpdateProductHandler {
    async fn handle(&self, mut req: Request) -> Result<ApiResponse, ApiError> {
        let product_id: u32 = req.param("id")
            .and_then(|id| id.parse().ok())
            .ok_or_else(|| ApiError::bad_request("Invalid product ID"))?;

        let form_data = req.form().await.map_err(|e| ApiError::bad_request(&format!("Failed to parse form data: {}", e)))?;
        
        let name = form_data.get("name").map(|s| s.trim()).filter(|s| !s.is_empty());
        let description = form_data.get("description").map(|s| s.trim()).filter(|s| !s.is_empty());
        let price_str = form_data.get("price").map(|s| s.trim()).filter(|s| !s.is_empty());
        let category = form_data.get("category").map(|s| s.trim()).filter(|s| !s.is_empty());

        let price: f64 = price_str
            .ok_or_else(|| ApiError::bad_request("Price is required."))?
            .parse()
            .map_err(|_| ApiError::bad_request("Invalid price format."))?;

        let mut products = PRODUCTS.lock().unwrap();
        if let Some(product) = products.iter_mut().find(|p| p.id == product_id) {
            if let Some(n) = name { product.name = n.to_string(); }
            if let Some(d) = description { product.description = d.to_string(); }
            product.price = price;
            if let Some(c) = category { product.category = c.to_string(); }
            
            info!("Product {} updated: {:?}", product_id, product);
            Ok(ApiResponse::ok(json!({"message": "Product updated successfully", "product": product})))
        } else {
            Err(ApiError::not_found(&format!("Product with ID {} not found", product_id)))
        }
    }
}

// API Handler for deleting products
struct DeleteProductHandler;

#[async_trait]
impl ApiHandler for DeleteProductHandler {
    async fn handle(&self, req: Request) -> Result<ApiResponse, ApiError> {
        let product_id: u32 = req.param("id")
            .and_then(|id| id.parse().ok())
            .ok_or_else(|| ApiError::bad_request("Invalid product ID"))?;
        
        let mut products = PRODUCTS.lock().unwrap();
        let initial_len = products.len();
        products.retain(|p| p.id != product_id);

        if products.len() < initial_len {
            info!("Product with ID {} deleted", product_id);
            Ok(ApiResponse::ok(json!({"message": "Product deleted successfully"})))
        } else {
            Err(ApiError::not_found(&format!("Product with ID {} not found", product_id)))
        }
    }
}

// Catalog Layout Component
component!(CatalogLayout, props => {
    let title = props.get("title").and_then(|v| v.as_str()).unwrap_or("RustNext Product Catalog");
    let children_html = props.get("children_html").and_then(|v| v.as_str()).unwrap_or("");
    let error_message = props.get("error_message").and_then(|v| v.as_str()).unwrap_or("");
    let success_message = props.get("success_message").and_then(|v| v.as_str()).unwrap_or("");
    let active_path = props.get("active_path").and_then(|v| v.as_str()).unwrap_or("/");

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
                                .child(a().prop("href", "/").class(if active_path == "/" { "active" } else { "" }).child(text("Products")))
                                .child(a().prop("href", "/products/new").class(if active_path == "/products/new" { "active" } else { "" }).child(text("Add Product")))
                                .child(a().prop("href", "/about").class(if active_path == "/about" { "active" } else { "" }).child(text("About")))
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
                            if !success_message.is_empty() {
                                div()
                                    .class("success-message")
                                    .child(text(success_message))
                            } else {
                                div()
                            }
                        )
                        .child(
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
                        .child(text("© 2024 RustNext Product Catalog. Built with Rust."))
                )
        )
});

// Product Card Component
component!(ProductCard, props => {
    let product_id = props.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
    let name = props.get("name").and_then(|v| v.as_str()).unwrap_or("Untitled Product");
    let price = props.get("price").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let category = props.get("category").and_then(|v| v.as_str()).unwrap_or("Uncategorized");
    
    article()
        .class("card")
        .child(
            h3()
                .child(
                    a()
                        .prop("href", format!("/products/{}", product_id))
                        .child(text(name))
                )
        )
        .child(
            p().class("text-gray-700 text-sm mt-2")
                .child(text(&format!("Category: {}", category)))
        )
        .child(
            div()
                .class("flex justify-between items-center mt-4 text-lg font-bold")
                .child(
                    span().child(text(&format!("${:.2}", price)))
                )
                .child(
                    a()
                        .class("btn btn-secondary text-sm")
                        .prop("href", format!("/products/{}", product_id))
                        .child(text("View Details"))
                )
        )
});

// Product Form Component (for both create and edit)
component!(ProductForm, props => {
    let product_id = props.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
    let name = props.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let description = props.get("description").and_then(|v| v.as_str()).unwrap_or("");
    let price = props.get("price").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let category = props.get("category").and_then(|v| v.as_str()).unwrap_or("");

    let action_url = if product_id == 0 {
        "/api/products".to_string()
    } else {
        format!("/api/products/{}/update", product_id)
    };
    let submit_text = if product_id == 0 { "Add Product" } else { "Update Product" };

    form()
        .prop("method", "POST")
        .prop("action", action_url)
        .class("mt-4 p-4 border border-gray-200 rounded-md bg-gray-50")
        .child(
            div()
                .class("form-group")
                .child(label().prop("for", "name").child(text("Product Name")))
                .child(
                    input()
                        .prop("type", "text")
                        .prop("name", "name")
                        .prop("placeholder", "e.g., RustNext Mug")
                        .prop("required", "true")
                        .prop("value", name)
                        .class("form-control")
                )
        )
        .child(
            div()
                .class("form-group")
                .child(label().prop("for", "description").child(text("Description")))
                .child(
                    Element::new("textarea")
                        .prop("name", "description")
                        .prop("placeholder", "Detailed description of the product...")
                        .prop("required", "true")
                        .prop("rows", "3")
                        .child(text(description)) // Set textarea content
                        .class("form-control")
                )
        )
        .child(
            div()
                .class("form-group")
                .child(label().prop("for", "price").child(text("Price")))
                .child(
                    input()
                        .prop("type", "number")
                        .prop("name", "price")
                        .prop("step", "0.01")
                        .prop("placeholder", "e.g., 19.99")
                        .prop("required", "true")
                        .prop("value", format!("{:.2}", price)) // FIX: Removed &
                        .class("form-control")
                )
        )
        .child(
            div()
                .class("form-group")
                .child(label().prop("for", "category").child(text("Category")))
                .child(
                    input()
                        .prop("type", "text")
                        .prop("name", "category")
                        .prop("placeholder", "e.g., Merchandise, Books, Electronics")
                        .prop("required", "true")
                        .prop("value", category)
                        .class("form-control")
                )
        )
        .child(
            button()
                .prop("type", "submit")
                .class("btn mt-2")
                .child(text(submit_text))
        )
});


// Product Listing Page
page!(ProductListingPage, req => {
    let products_cloned = {
        PRODUCTS.lock().unwrap().clone()
    };

    let mut product_cards_futures = Vec::new();
    for product in products_cloned {
        let mut product_props = HashMap::new();
        product_props.insert("id".to_string(), json!(product.id));
        product_props.insert("name".to_string(), json!(product.name));
        product_props.insert("price".to_string(), json!(product.price));
        product_props.insert("category".to_string(), json!(product.category));
        
        let component_registry_arc = get_component_registry().clone();
        product_cards_futures.push(async move {
            let component_registry = component_registry_arc.lock().await;
            component_registry.render("product_card", &product_props).await.unwrap_or_else(|| div())
        });
    }

    let product_cards = futures::future::join_all(product_cards_futures).await;

    let content = section()
        .child(h2().class("text-2xl font-bold mb-4").child(text("All Products")))
        .child(
            div()
                .class("grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6")
                .children(product_cards)
        );

    let mut layout_props = HashMap::new();
    layout_props.insert("title".to_string(), json!("Product Catalog"));
    layout_props.insert("children_html".to_string(), json!(get_renderer().render_to_html(&content)));
    layout_props.insert("active_path".to_string(), json!(req.uri.path()));

    if let Some(error_msg) = req.query_param("error") {
        layout_props.insert("error_message".to_string(), json!(urlencoding::decode(error_msg).unwrap_or_default()));
    }
    if let Some(success_msg) = req.query_param("success") {
        layout_props.insert("success_message".to_string(), json!(urlencoding::decode(success_msg).unwrap_or_default()));
    }
    
    let component_registry = get_component_registry().lock().await;
    let rendered_element = component_registry.render("catalog_layout", &layout_props).await;
    rendered_element.unwrap_or_else(|| {
        div().child(text("Error rendering page"))
    })
});

// New Product Page
page!(NewProductPage, req => {
    let mut layout_props = HashMap::new();
    layout_props.insert("title".to_string(), json!("Add New Product"));
    let product_form_element = {
        let component_registry = get_component_registry().lock().await;
        component_registry.render("product_form", &HashMap::new()).await.unwrap_or_else(|| div())
    };
    layout_props.insert("children_html".to_string(), json!(get_renderer().render_to_html(&product_form_element)));
    layout_props.insert("active_path".to_string(), json!(req.uri.path()));

    if let Some(error_msg) = req.query_param("error") {
        layout_props.insert("error_message".to_string(), json!(urlencoding::decode(error_msg).unwrap_or_default()));
    }
    if let Some(success_msg) = req.query_param("success") {
        layout_props.insert("success_message".to_string(), json!(urlencoding::decode(success_msg).unwrap_or_default()));
    }
    
    let component_registry = get_component_registry().lock().await;
    let rendered_element = component_registry.render("catalog_layout", &layout_props).await;
    rendered_element.unwrap_or_else(|| {
        div().child(text("Error rendering page"))
    })
});

// Product Detail Page
page!(ProductDetailPage, req => {
    let product_id: u32 = req.param("id")
        .and_then(|id| id.parse().ok())
        .unwrap_or(0);
    
    let product_option = {
        PRODUCTS.lock().unwrap().iter().find(|p| p.id == product_id).cloned()
    };

    let content = if let Some(ref product) = product_option {
        div()
            .child(
                article()
                    .class("card mb-6")
                    .child(h2().class("text-2xl font-bold mb-2").child(text(&product.name)))
                    .child(p().class("text-gray-700 mb-3").child(text(&product.description)))
                    .child(
                        div()
                            .class("flex justify-between items-center text-sm text-gray-600")
                            .child(span().child(text(&format!("Category: {}", product.category))))
                            .child(span().child(text(&format!("Price: ${:.2}", product.price))))
                            .child(span().child(text(&format!("Added: {}", product.created_at))))
                    )
                    .child(
                        div()
                            .class("mt-6 flex gap-3")
                            .child(a().class("btn btn-secondary").prop("href", "/").child(text("← Back to Products")))
                            .child(a().class("btn").prop("href", format!("/products/{}/edit", product.id)).child(text("Edit Product")))
                            .child(
                                a() // Delete button
                                    .prop("href", format!("/api/products/{}/delete", product.id))
                                    .prop("method", "POST")
                                    .class("btn btn-danger")
                                    .child(text("Delete Product"))
                            )
                    )
            )
    } else {
        div()
            .class("card")
            .child(h1().child(text("Product Not Found")))
            .child(p().child(text("The requested product could not be found.")))
            .child(a().class("btn").prop("href", "/").child(text("← Back to Products")))
    };
    
    let mut layout_props = HashMap::new();
    layout_props.insert("title".to_string(), json!(
        product_option.map(|p| p.name).unwrap_or_else(|| "Product Not Found".to_string())
    ));
    layout_props.insert("children_html".to_string(), json!(get_renderer().render_to_html(&content)));
    layout_props.insert("active_path".to_string(), json!(req.uri.path()));

    if let Some(error_msg) = req.query_param("error") {
        layout_props.insert("error_message".to_string(), json!(urlencoding::decode(error_msg).unwrap_or_default()));
    }
    if let Some(success_msg) = req.query_param("success") {
        layout_props.insert("success_message".to_string(), json!(urlencoding::decode(success_msg).unwrap_or_default()));
    }
    
    let component_registry = get_component_registry().lock().await;
    let rendered_element = component_registry.render("catalog_layout", &layout_props).await;
    rendered_element.unwrap_or_else(|| {
        div().child(text("Error rendering page"))
    })
});

// Edit Product Page
page!(EditProductPage, req => {
    let product_id: u32 = req.param("id")
        .and_then(|id| id.parse().ok())
        .unwrap_or(0);
    
    let product_option = {
        PRODUCTS.lock().unwrap().iter().find(|p| p.id == product_id).cloned()
    };

    let product_form_element = if let Some(product) = product_option.clone() {
        let mut form_props = HashMap::new();
        form_props.insert("id".to_string(), json!(product.id));
        form_props.insert("name".to_string(), json!(product.name));
        form_props.insert("description".to_string(), json!(product.description));
        form_props.insert("price".to_string(), json!(product.price));
        form_props.insert("category".to_string(), json!(product.category));
        
        let component_registry = get_component_registry().lock().await;
        component_registry.render("product_form", &form_props).await.unwrap_or_else(|| div())
    } else {
        div()
            .class("card")
            .child(h1().child(text("Product Not Found")))
            .child(p().child(text("The product you are trying to edit could not be found.")))
            .child(a().class("btn").prop("href", "/").child(text("← Back to Products")))
    };

    let mut layout_props = HashMap::new();
    layout_props.insert("title".to_string(), json!(
        product_option.map(|p| format!("Edit {}", p.name)).unwrap_or_else(|| "Edit Product".to_string())
    ));
    layout_props.insert("children_html".to_string(), json!(get_renderer().render_to_html(&product_form_element)));
    layout_props.insert("active_path".to_string(), json!(req.uri.path()));

    if let Some(error_msg) = req.query_param("error") {
        layout_props.insert("error_message".to_string(), json!(urlencoding::decode(error_msg).unwrap_or_default()));
    }
    if let Some(success_msg) = req.query_param("success") {
        layout_props.insert("success_message".to_string(), json!(urlencoding::decode(success_msg).unwrap_or_default()));
    }
    
    let component_registry = get_component_registry().lock().await;
    let rendered_element = component_registry.render("catalog_layout", &layout_props).await;
    rendered_element.unwrap_or_else(|| {
        div().child(text("Error rendering page"))
    })
});


// About Page for Product Catalog App
page!(AboutPage, req => {
    let content = section()
        .child(
            h2().class("text-2xl font-bold mb-4").child(text("About This Product Catalog App"))
        )
        .child(
            div()
                .class("card")
                .child(
                    p().child(text("This is a simple Product Catalog application built with RustNext."))
                )
                .child(
                    p().child(text("It demonstrates full-stack capabilities including: "))
                )
                .child(
                    ul()
                        .class("list-disc pl-5 space-y-2")
                        .child(li().child(text("UI Components for Product Listing and Details")))
                        .child(li().child(text("API Endpoints for managing Products (Create, Read, Update, Delete)")))
                        .child(li().child(text("In-memory data storage (for simplicity, can be extended with a database)")))
                        .child(li().child(text("Form submission and POST-redirect-GET pattern for robust data handling")))
                        .child(li().child(text("Improved UI/UX with modern CSS styling")))
                        .child(li().child(text("Error and success message display after redirects")))
                )
                .child(
                    p().class("mt-4").child(text("This application serves as a comprehensive example of building interactive web applications with RustNext, showcasing how different parts of the framework work together."))
                )
        );

    let mut layout_props = HashMap::new();
    layout_props.insert("title".to_string(), json!("About Product Catalog"));
    layout_props.insert("children_html".to_string(), json!(get_renderer().render_to_html(&content)));
    layout_props.insert("active_path".to_string(), json!(req.uri.path()));

    let component_registry = get_component_registry().lock().await;
    let rendered_element = component_registry.render("catalog_layout", &layout_props).await;
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
    info!("   API endpoints: /api/products, /api/products/:id, /api/products/:id/update, /api/products/:id/delete");
    info!("");
    info!("🎯 Features Demonstrated:");
    info!("   ✅ Enhanced UI/UX with modern CSS styling");
    info!("   ✅ Product Catalog data model");
    info!("   ✅ CRUD operations via API routes");
    info!("   ✅ Forms for creating and updating Products");
    info!("   ✅ POST-redirect-GET pattern for form submissions");
    info!("   ✅ Display of error/success messages after redirects");
    
    // Register components
    register_component!("catalog_layout", CatalogLayout).await?;
    register_component!("product_card", ProductCard).await?;
    register_component!("product_form", ProductForm).await?;
    
    // Register pages
    register_page!("/", ProductListingPage).await?;
    register_page!("/products/new", NewProductPage).await?;
    register_page!("/products/:id", ProductDetailPage).await?;
    register_page!("/products/:id/edit", EditProductPage).await?;
    register_page!("/about", AboutPage).await?;

    // Register API routes
    api_route!(hyper::Method::GET, "/api/products", GetProductsHandler).await?;
    api_route!(hyper::Method::GET, "/api/products/:id", GetProductHandler).await?;
    api_route!(hyper::Method::POST, "/api/products", CreateProductHandler).await?;
    api_route!(hyper::Method::POST, "/api/products/:id/update", UpdateProductHandler).await?;
    api_route!(hyper::Method::POST, "/api/products/:id/delete", DeleteProductHandler).await?;

    // Define a custom error handler for the App
    let custom_error_handler = Arc::new(|err: AppError| {
        error!("Application Error: {}", err);
        err.into_response()
    });

    // Create router
    let router = Router::new()
        .use_middleware(RateLimiter::new(100, 60))
        .get("/", |req| async move {
            let page_registry = get_page_registry().lock().await;
            let element_option = page_registry.render_page("/", &req).await;
            if let Some(element) = element_option {
                get_renderer().render_to_response(&element)
            } else {
                Err(Box::new(AppError::NotFound("Product listing page not found".to_string())) as Box<dyn std::error::Error + Send + Sync>)
            }
        })
        .get("/products/new", |req| async move {
            let page_registry = get_page_registry().lock().await;
            let element_option = page_registry.render_page("/products/new", &req).await;
            if let Some(element) = element_option {
                get_renderer().render_to_response(&element)
            } else {
                Err(Box::new(AppError::NotFound("New product page not found".to_string())) as Box<dyn std::error::Error + Send + Sync>)
            }
        })
        .get("/products/:id", |req| async move {
            let page_registry = get_page_registry().lock().await;
            let element_option = page_registry.render_page("/products/:id", &req).await;
            if let Some(element) = element_option {
                get_renderer().render_to_response(&element)
            } else {
                Err(Box::new(AppError::NotFound(format!("Product {} not found", req.param("id").unwrap_or(&"unknown".to_string())))) as Box<dyn std::error::Error + Send + Sync>)
            }
        })
        .get("/products/:id/edit", |req| async move {
            let page_registry = get_page_registry().lock().await;
            let element_option = page_registry.render_page("/products/:id/edit", &req).await;
            if let Some(element) = element_option {
                get_renderer().render_to_response(&element)
            } else {
                Err(Box::new(AppError::NotFound(format!("Edit product page for {} not found", req.param("id").unwrap_or(&"unknown".to_string())))) as Box<dyn std::error::Error + Send + Sync>)
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
        .post("/api/products", |req: Request| async move {
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
                            .header("Location", &format!("/products/new?error={}", urlencoding::encode(&error_msg)))
                            .header("HX-Redirect", &format!("/products/new?error={}", urlencoding::encode(&error_msg))))
                    }
                }
                None => Err(Box::new(AppError::NotFound("API endpoint /api/products (POST) not found".to_string())) as Box<dyn std::error::Error + Send + Sync>),
            }
        })
        .post("/api/products/:id/update", |req: Request| async move {
            let api_registry = get_api_registry().lock().await;
            let product_id_str = req.param("id").cloned().unwrap_or_default();
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
                            .header("Location", &format!("/products/{}?error={}", product_id_str, urlencoding::encode(&error_msg)))
                            .header("HX-Redirect", &format!("/products/{}?error={}", product_id_str, urlencoding::encode(&error_msg))))
                    }
                }
                None => Err(Box::new(AppError::NotFound("API endpoint /api/products/:id/update (POST) not found".to_string())) as Box<dyn std::error::Error + Send + Sync>),
            }
        })
        .post("/api/products/:id/delete", |req: Request| async move {
            let api_registry = get_api_registry().lock().await;
            match api_registry.handle_request(req).await {
                Some(_response) => {
                    // After delete, redirect back to product listing to show updated list
                    Ok(Response::new().status(StatusCode::SEE_OTHER).header("Location", "/?success=Product%20deleted%20successfully"))
                }
                None => Err(Box::new(AppError::NotFound("API endpoint /api/products/:id/delete (POST) not found".to_string())) as Box<dyn std::error::Error + Send + Sync>),
            }
        });

    // Create and run server
    let app = App::new()
        .router(router)
        .error_handler(custom_error_handler); // Use the defined error handler

    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;
    let server = Server::new(app, addr);
    
    info!("🚀 RustNext Product Catalog running at http://{}:{}", config.server.host, config.server.port);
    info!("📝 Available routes:");
    info!("   http://{}:{}/           - Product listing home page", config.server.host, config.server.port);
    info!("   http://{}:{}/products/new - Add new product form", config.server.host, config.server.port);
    info!("   http://{}:{}/products/1  - Individual product detail page", config.server.host, config.server.port);
    info!("   http://{}:{}/products/1/edit - Edit product form", config.server.host, config.server.port);
    info!("   http://{}:{}/about      - About page", config.server.host, config.server.port);
    info!("   API endpoints: /api/products, /api/products/:id, /api/products/:id/update, /api/products/:id/delete");
    info!("");
    info!("🎯 Features Demonstrated:");
    info!("   ✅ Enhanced UI/UX with modern CSS styling");
    info!("   ✅ Product Catalog data model");
    info!("   ✅ CRUD operations via API routes");
    info!("   ✅ Forms for creating and updating Products");
    info!("   ✅ POST-redirect-GET pattern for form submissions");
    info!("   ✅ Display of error/success messages after redirects");
    
    server.run().await
}
