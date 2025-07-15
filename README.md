# RustNext Web Framework Examples

This repository contains example applications built with RustNext, a high-performance web framework written in Rust, inspired by Next.js. It demonstrates full-stack capabilities including declarative UI components, API routes, form handling, and more.

## Table of Contents

1.  [Prerequisites](#prerequisites)
2.  [Project Structure](#project-structure)
3.  [Setup and Running Examples](#setup-and-running-examples)
    *   [Running the Project Dashboard App](#running-the-project-dashboard-app)
    *   [Running the Product Catalog App](#running-the-product-catalog-app)
4.  [Configuration](#configuration)
5.  [Features Demonstrated](#features-demonstrated)

## Prerequisites

Before you begin, ensure you have the following installed:

*   **Rust and Cargo:** If you don't have Rust installed, you can get it from [rustup.rs](https://rustup.rs/).
    \`\`\`bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    \`\`\`
    Follow the on-screen instructions. This will install `rustc` (the Rust compiler) and `cargo` (Rust's package manager and build tool).

## Project Structure

The core RustNext framework code resides in the `src/` directory. The examples are located in the `examples/` directory.

\`\`\`
rustnext/
├── src/
│   ├── api/             # API route handling
│   ├── auth/            # Authentication utilities
│   ├── cache/           # Caching mechanisms
│   ├── config/          # Application configuration
│   ├── compression/     # HTTP compression middleware
│   ├── database/        # Database integration (optional)
│   ├── dev/             # Development utilities
│   ├── error/           # Custom error handling
│   ├── file_upload/     # File upload utilities
│   ├── forms/           # Form parsing and handling
│   ├── handler/         # Request handler traits
│   ├── logging/         # Logging setup
│   ├── middleware/      # HTTP middleware (e.g., AuthGuard, RateLimiter)
│   ├── request/         # HTTP Request abstraction
│   ├── response/        # HTTP Response builder
│   ├── router/          # Routing system
│   ├── session/         # Session management
│   ├── static_files/    # Static file serving
│   ├── ui/              # Declarative UI component system
│   └── lib.rs           # Main library entry point
├── examples/
│   ├── project_dashboard_app.rs # Example: Project Management Dashboard
│   ├── product_catalog_app.rs   # Example: Product Catalog Application
│   └── ...other_examples.rs
├── Cargo.toml           # Project dependencies and metadata
├── Cargo.lock           # Exact dependency versions
└── config.toml          # Application configuration file
\`\`\`

## Setup and Running Examples

1.  **Clone the repository:**
    \`\`\`bash
    git clone https://github.com/zaoinc/rustnext.git
    cd rustnext
    \`\`\`

2.  **Build the project (optional, but good for checking dependencies):**
    \`\`\`bash
    cargo build
    \`\`\`

### Running the Project Dashboard App

This example demonstrates a simple project management dashboard with CRUD operations for projects and tasks, form handling, and a multi-entity data model.

1.  **Run the application:**
    \`\`\`bash
    cargo run --example project_dashboard_app
    \`\`\`
    You should see output similar to this:
    \`\`\`
    🔧 Configuration loaded:
       Server: 127.0.0.1:3000
       Features: compression=false, metrics=false, logging=true
       API endpoints: /api/projects, /api/projects/:id, /api/projects/:id/tasks, /api/projects/:project_id/tasks/:task_id/toggle, /api/projects/:project_id/tasks/:task_id/delete

    🎯 Features Demonstrated:
       ✅ Enhanced UI/UX with modern CSS styling
       ✅ Multi-entity data model (Projects & Tasks)
       ✅ CRUD operations via API routes
       ✅ Forms for creating Projects and Tasks
       ✅ POST-redirect-GET pattern for form submissions
       ✅ Display of error/success messages after redirects

    🚀 RustNext Project Dashboard running at http://127.0.0.1:3000
    📝 Available routes:
       http://127.0.0.1:3000/           - Project dashboard home page
       http://127.0.0.1:3000/projects/new - Create new project form
       http://127.0.0.1:3000/projects/1  - Individual project detail page
       http://127.0.0.1:3000/about      - About page
       API endpoints: /api/projects, /api/projects/:id, /api/projects/:id/tasks, /api/projects/:project_id/tasks/:task_id/toggle, /api/projects/:project_id/tasks/:task_id/delete
    \`\`\`

2.  **Access the application:**
    Open your web browser and navigate to `http://127.0.0.1:3000`.

3.  **Usage:**
    *   **Dashboard:** View a list of existing projects.
    *   **New Project:** Click "New Project" in the navigation to create a new project using the form.
    *   **View Details:** Click "View Details" on a project card to see its description and associated tasks.
    *   **Add Task:** On the project detail page, use the "Add New Task" form to add tasks to that project.
    *   **Toggle Task/Delete Task:** On the project detail page, you can mark tasks as complete/pending or delete them.
    *   **About:** Learn more about the application's features.

    **Note:** All data in this example is stored in-memory and will be reset when the application is restarted.

### Running the Product Catalog App

This example showcases a simple product catalog with CRUD operations for products, including forms for adding and editing products.

1.  **Run the application:**
    \`\`\`bash
    cargo run --example product_catalog_app
    \`\`\`
    You should see output similar to this:
    \`\`\`
    🔧 Configuration loaded:
       Server: 127.0.0.1:3000
       Features: compression=false, metrics=false, logging=true
       API endpoints: /api/products, /api/products/:id, /api/products/:id/update, /api/products/:id/delete

    🎯 Features Demonstrated:
       ✅ Enhanced UI/UX with modern CSS styling
       ✅ Product Catalog data model
       ✅ CRUD operations via API routes
       ✅ Forms for creating and updating Products
       ✅ POST-redirect-GET pattern for form submissions
       ✅ Display of error/success messages after redirects

    🚀 RustNext Product Catalog running at http://127.0.0.1:3000
    📝 Available routes:
       http://127.0.0.1:3000/           - Product listing home page
       http://127.0.0.1:3000/products/new - Add new product form
       http://127.0.0.1:3000/products/1  - Individual product detail page
       http://127.0.0.1:3000/products/1/edit - Edit product form
       http://127.0.0.1:3000/about      - About page
       API endpoints: /api/products, /api/products/:id, /api/products/:id/update, /api/products/:id/delete
    \`\`\`

2.  **Access the application:**
    Open your web browser and navigate to `http://127.0.0.1:3000`.

3.  **Usage:**
    *   **Products Listing:** View a list of available products.
    *   **Add Product:** Click "Add Product" in the navigation to add a new product to the catalog.
    *   **View Details:** Click "View Details" on a product card to see its full description and details.
    *   **Edit Product:** On the product detail page, click "Edit Product" to modify its details.
    *   **Delete Product:** On the product detail page, click "Delete Product" to remove it from the catalog.
    *   **About:** Learn more about the application's features.

    **Note:** All data in this example is stored in-memory and will be reset when the application is restarted.

## Configuration

The `config.toml` file in the root directory allows you to configure various aspects of the application, such as the server host and port, and enable/disable certain features.

\`\`\`toml
# config.toml
[server]
host = "127.0.0.1"
port = 3000

[features]
compression = false
metrics = false
logging = true
\`\`\`

You can modify these values to change the server's listening address or enable/disable features like compression and metrics (if implemented in the core framework).

## Features Demonstrated

Both the Project Dashboard and Product Catalog examples showcase the following capabilities of the RustNext framework:

*   **Enhanced UI/UX with modern CSS styling:** Basic styling is applied to make the applications presentable.
*   **Multi-entity data models:** Demonstrates managing different types of data (Projects/Tasks, Products).
*   **CRUD operations via API routes:** Full Create, Read, Update, and Delete functionality exposed through RESTful API endpoints.
*   **Forms for data entry:** HTML forms are used for user input, which are processed by API routes.
*   **POST-redirect-GET pattern for form submissions:** Ensures robust form handling, preventing duplicate submissions and allowing for clear success/error messages.
*   **Display of error/success messages after redirects:** User feedback is provided through URL query parameters after form submissions or actions.
*   **In-memory data storage:** For simplicity, data is stored in static `Mutex<Vec<T>>` structures. This can be extended with a database integration.
# rustnext
