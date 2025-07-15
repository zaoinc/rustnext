

```markdown
# RustNext Web Framework

![Rust](https://img.shields.io/badge/Rust-1.70+-blue)
![License](https://img.shields.io/badge/License-MIT-green)

A high-performance web framework for Rust inspired by Next.js, featuring:

- 🚀 Full-stack capabilities in a single codebase
- ⚡ Blazing fast performance
- 🧩 Declarative UI components
- 🔄 Built-in API routes
- 📝 Form handling utilities

## Table of Contents

- [Features](#-features)
- [Quick Start](#-quick-start)
- [Examples](#-examples)
- [Project Structure](#-project-structure)
- [Configuration](#-configuration)
- [Contributing](#-contributing)
- [License](#-license)

## ✨ Features

| Feature               | Description                                                                 |
|-----------------------|-----------------------------------------------------------------------------|
| **Type-safe APIs**    | Compile-time validated routes and handlers                                  |
| **UI Components**     | Reusable, declarative views with server-side rendering                      |
| **Zero Config**       | Sensible defaults with minimal setup required                               |
| **Middleware**        | Built-in auth, logging, and compression                                    |
| **Form Handling**     | Easy form parsing and validation                                           |
| **File Uploads**      | Streaming file upload support                                              |

## 🚀 Quick Start

1. Install Rust (if needed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. Create a new project:
   ```bash
   cargo new my_rustnext_app
   cd my_rustnext_app
   ```

3. Add RustNext to your dependencies:
   ```toml
   [dependencies]
   rustnext = "0.1"
   ```

4. Create your first component in `src/main.rs`:
   ```rust
   use rustnext::prelude::*;

   #[component]
   fn App() -> Markup {
       html! {
           h1 { "Hello RustNext!" }
       }
   }

   fn main() {
       RustNext::new()
           .route("/", App)
           .run();
   }
   ```

5. Run your app:
   ```bash
   cargo run
   ```

## 📚 Examples

### Project Dashboard

```bash
cargo run --example project_dashboard_app
```

**Demonstrates:**
- CRUD operations for projects and tasks
- Form validation
- Multi-entity relationships
- POST-redirect-GET pattern

### Product Catalog

```bash
cargo run --example product_catalog_app
```

**Demonstrates:**
- Product management
- Edit/delete workflows
- Search and filtering
- Pagination

Access both examples at: `http://localhost:3000`

## 📂 Project Structure

```
my_rustnext_app/
├── src/
│   ├── components/      # UI components
│   ├── pages/           # Page layouts
│   ├── api/             # API routes
│   └── main.rs          # Application entry
├── static/              # Static assets
├── Cargo.toml
└── config.toml          # Configuration
```

## ⚙️ Configuration

Edit `config.toml`:

```toml
[server]
host = "0.0.0.0"
port = 3000

[database]
url = "postgres://user:pass@localhost/db" # Optional

[features]
logging = true
compression = true
```

## 🤝 Contributing

We welcome contributions! Please:

1. Fork the repository
2. Create a feature branch
3. Submit a PR

See our [Contribution Guidelines](CONTRIBUTING.md) for details.

## 📜 License

MIT © [sazalo]
```

