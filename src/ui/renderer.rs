use crate::ui::Element;
use crate::Response;
use serde_json::Value;
use once_cell::sync::OnceCell; // New import

pub struct Renderer;

impl Renderer {
    pub fn new() -> Self {
        Renderer
    }

    pub fn render_to_html(&self, element: &Element) -> String {
        match element.tag.as_str() {
            "text" => {
                if let Some(text) = &element.text {
                    html_escape::encode_text(text).to_string()
                } else {
                    String::new()
                }
            }
            _ => {
                let mut html = format!("<{}", element.tag);
                let mut inner_html_content: Option<String> = None; // New: To hold raw HTML content

                // Add attributes
                for (key, value) in &element.props {
                    if key == "_raw_html" { // Special handling for raw HTML content
                        if let Value::String(s) = value {
                            inner_html_content = Some(s.clone());
                        }
                    } else {
                        let attr_value = match value {
                            Value::String(s) => s.clone(),
                            Value::Number(n) => n.to_string(),
                            Value::Bool(b) => b.to_string(),
                            _ => value.to_string(),
                        };
                        html.push_str(&format!(" {}=\"{}\"", key, html_escape::encode_double_quoted_attribute(&attr_value)));
                    }
                }
                
                html.push('>');
                
                // Add children or raw HTML content
                if let Some(raw_html) = inner_html_content {
                    html.push_str(&raw_html); // Insert raw HTML directly
                } else {
                    for child in &element.children {
                        html.push_str(&self.render_to_html(child));
                    }
                }
                
                html.push_str(&format!("</{}>", element.tag));
                html
            }
        }
    }

    pub fn render_to_response(&self, element: &Element) -> Result<Response, Box<dyn std::error::Error + Send + Sync>> {
        let html_content = self.render_to_html(element);
        let full_html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>RustNext App</title>
    <style>
        :root {{
            --color-primary: #2a9d8f; /* Deep Teal */
            --color-primary-dark: #218377;
            --color-accent: #f4a261; /* Soft Orange */
            --color-background: #f8f9fa; /* Off-white */
            --color-text-dark: #343a40; /* Dark gray */
            --color-text-medium: #6c757d; /* Medium gray */
            --color-text-light: #adb5bd; /* Light gray */
            --color-border: #e9ecef; /* Light border */
            --color-card-bg: #ffffff;
            --color-header-bg: #2c3e50; /* Dark Blue-Gray */
            --color-footer-bg: #34495e; /* Slightly darker Blue-Gray */
        }}

        body {{ 
            font-family: 'Inter', sans-serif; /* Modern sans-serif font */
            margin: 0;
            padding: 0;
            line-height: 1.6;
            color: var(--color-text-dark);
            background-color: var(--color-background);
            -webkit-font-smoothing: antialiased;
            -moz-osx-font-smoothing: grayscale;
        }}
        
        /* Basic Reset & Box Sizing */
        *, *::before, *::after {{
            box-sizing: border-box;
        }}

        /* Layout Containers */
        .container {{ 
            max-width: 1200px; 
            margin: 0 auto; 
            padding: 20px; 
        }}
        
        /* Header & Navigation */
        .header {{ 
            background: var(--color-header-bg); 
            color: white; 
            padding: 1rem 0; 
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }}
        .nav {{ 
            display: flex; 
            gap: 1.5rem; 
            align-items: center;
        }}
        .nav a {{ 
            color: white; 
            text-decoration: none; 
            padding: 0.5rem 1rem; 
            border-radius: 6px;
            transition: background 0.2s ease;
        }}
        .nav a:hover {{ 
            background: rgba(255,255,255,0.15); 
        }}
        .nav a.active {{
            background: var(--color-primary);
            font-weight: 600;
        }}

        /* Main Content Area */
        .main {{ 
            padding: 2rem 0; 
            min-height: calc(100vh - 120px); /* Adjust based on header/footer height */
        }}

        /* Footer */
        .footer {{ 
            background: var(--color-footer-bg); 
            color: white; 
            text-align: center; 
            padding: 1rem 0; 
            font-size: 0.9rem;
        }}

        /* Cards */
        .card {{ 
            background: var(--color-card-bg); 
            border-radius: 8px; 
            box-shadow: 0 4px 12px rgba(0,0,0,0.08); 
            padding: 1.5rem; 
            margin-bottom: 1.5rem; 
            border: 1px solid var(--color-border);
        }}

        /* Buttons */
        .btn {{ 
            background: var(--color-primary); 
            color: white; 
            border: none; 
            padding: 0.75rem 1.5rem; 
            border-radius: 6px; 
            cursor: pointer; 
            text-decoration: none; 
            display: inline-block; 
            font-weight: 500;
            transition: background 0.2s ease, transform 0.1s ease;
        }}
        .btn:hover {{ 
            background: var(--color-primary-dark); 
            transform: translateY(-1px);
        }}
        .btn:active {{
            transform: translateY(0);
        }}
        .btn-secondary {{
            background: var(--color-text-medium);
        }}
        .btn-secondary:hover {{
            background: #5a6268;
        }}
        .btn-danger {{
            background: #dc3545;
        }}
        .btn-danger:hover {{
            background: #c82333;
        }}

        /* Forms */
        .form-group {{ 
            margin-bottom: 1rem; 
        }}
        .form-control {{ 
            width: 100%; 
            padding: 0.75rem; 
            border: 1px solid var(--color-border); 
            border-radius: 6px; 
            font-size: 1rem;
            color: var(--color-text-dark);
            transition: border-color 0.2s ease, box-shadow 0.2s ease;
        }}
        .form-control:focus {{
            border-color: var(--color-primary);
            outline: none;
            box-shadow: 0 0 0 3px rgba(42, 157, 143, 0.25);
        }}
        label {{
            display: block;
            margin-bottom: 0.5rem;
            font-weight: 500;
            color: var(--color-text-dark);
        }}
        textarea.form-control {{
            min-height: 100px;
            resize: vertical;
        }}

        /* Typography */
        h1, h2, h3, h4, h5, h6 {{
            color: var(--color-text-dark);
            margin-top: 0;
            margin-bottom: 1rem;
            font-weight: 700;
        }}
        h1 {{ font-size: 2.5rem; }}
        h2 {{ font-size: 2rem; }}
        h3 {{ font-size: 1.75rem; }}
        p {{
            margin-bottom: 1rem;
        }}
        .text-sm {{ font-size: 0.875rem; }}
        .text-md {{ font-size: 1rem; }}
        .text-lg {{ font-size: 1.125rem; }}
        .text-xl {{ font-size: 1.25rem; }}
        .font-semibold {{ font-weight: 600; }}
        .font-bold {{ font-weight: 700; }}
        .text-gray-500 {{ color: var(--color-text-medium); }}
        .text-gray-600 {{ color: var(--color-text-dark); }}
        .text-gray-700 {{ color: var(--color-text-dark); }}
        .text-gray-800 {{ color: var(--color-text-dark); }}
        .line-through {{ text-decoration: line-through; }}
        .leading-relaxed {{ line-height: 1.8; }}

        /* Utilities */
        .mt-1 {{ margin-top: 0.25rem; }}
        .mt-2 {{ margin-top: 0.5rem; }}
        .mt-3 {{ margin-top: 0.75rem; }}
        .mt-4 {{ margin-top: 1rem; }}
        .mt-6 {{ margin-top: 1.5rem; }}
        .mb-4 {{ margin-bottom: 1rem; }}
        .mb-6 {{ margin-bottom: 1.5rem; }}
        .p-3 {{ padding: 0.75rem; }}
        .p-4 {{ padding: 1rem; }}
        .flex {{ display: flex; }}
        .items-center {{ align-items: center; }}
        .justify-between {{ justify-content: space-between; }}
        .justify-center {{ justify-content: center; }}
        .mr-3 {{ margin-right: 0.75rem; }}
        .border-b {{ border-bottom: 1px solid var(--color-border); }}
        .last\:border-b-0:last-child {{ border-bottom: none; }}
        .bg-gray-50 {{ background-color: #f8f9fa; }} /* Light background for form */
        .rounded-md {{ border-radius: 6px; }}
        .error-message {{ 
            color: #dc3545; /* Red for errors */
            background-color: #f8d7da; /* Light red background */
            border: 1px solid #f5c6cb;
            padding: 0.75rem 1.25rem;
            margin-bottom: 1rem;
            border-radius: 6px;
            font-size: 0.9rem;
        }}
        .success-message {{
            color: #28a745; /* Green for success */
            background-color: #d4edda; /* Light green background */
            border: 1px solid #c3e6cb;
            padding: 0.75rem 1.25rem;
            margin-bottom: 1rem;
            border-radius: 6px;
            font-size: 0.9rem;
        }}

        /* Responsive adjustments */
        @media (max-width: 768px) {{
            .container {{ padding: 15px; }}
            h1 {{ font-size: 2rem; }}
            h2 {{ font-size: 1.75rem; }}
            .nav {{ flex-direction: column; align-items: flex-start; }}
            .nav a {{ width: 100%; text-align: center; }}
        }}
    </style>
</head>
<body>
    {}
</body>
</html>"#,
            html_content
        );

        Ok(Response::new().html(&full_html))
    }
}

// Global renderer instance using once_cell
static GLOBAL_RENDERER: OnceCell<Renderer> = OnceCell::new();

pub fn get_renderer() -> &'static Renderer {
    GLOBAL_RENDERER.get_or_init(Renderer::new)
}
