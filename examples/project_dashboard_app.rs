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

// In-memory storage for projects and tasks
static PROJECTS: Lazy<Mutex<Vec<Project>>> = Lazy::new(|| Mutex::new(vec![
    Project {
        id: 1,
        name: "RustNext Framework Development".to_string(),
        description: "Developing a high-performance web framework in Rust, inspired by Next.js.".to_string(),
        status: "In Progress".to_string(),
        created_at: "2024-01-01".to_string(),
        tasks: vec![
            Task { id: 101, name: "Implement Router".to_string(), description: "Build a robust routing system with parameter parsing.".to_string(), completed: true, due_date: "2024-01-15".to_string() },
            Task { id: 102, name: "Create UI Component System".to_string(), description: "Design and implement a declarative UI component system.".to_string(), completed: true, due_date: "2024-01-30".to_string() },
            Task { id: 103, name: "Add Middleware Support".to_string(), description: "Integrate a flexible middleware pipeline for request processing.".to_string(), completed: false, due_date: "2024-02-10".to_string() },
            Task { id: 104, name: "Develop API Route Handlers".to_string(), description: "Enable creation of API endpoints for backend logic.".to_string(), completed: false, due_date: "2024-02-20".to_string() },
        ],
    },
    Project {
        id: 2,
        name: "Personal Website Redesign".to_string(),
        description: "Redesigning my personal portfolio website using the new RustNext framework.".to_string(),
        status: "Planned".to_string(),
        created_at: "2024-02-01".to_string(),
        tasks: vec![
            Task { id: 201, name: "Design Mockups".to_string(), description: "Create wireframes and high-fidelity mockups for the new site.".to_string(), completed: false, due_date: "2024-02-15".to_string() },
            Task { id: 202, name: "Set up RustNext Project".to_string(), description: "Initialize a new RustNext project and configure basic settings.".to_string(), completed: false, due_date: "2024-02-20".to_string() },
        ],
    },
]));

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Project {
    id: u32,
    name: String,
    description: String,
    status: String,
    created_at: String,
    tasks: Vec<Task>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Task {
    id: u32,
    name: String,
    description: String,
    completed: bool,
    due_date: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateProjectRequest {
    name: String,
    description: String,
    status: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateTaskRequest {
    name: String,
    description: String,
    due_date: String,
}

// API Handler for getting projects
struct GetProjectsHandler;

#[async_trait]
impl ApiHandler for GetProjectsHandler {
    async fn handle(&self, _req: Request) -> Result<ApiResponse, ApiError> {
        let projects = PROJECTS.lock().unwrap().clone();
        Ok(ApiResponse::ok(serde_json::to_value(projects).unwrap()))
    }
}

// API Handler for getting a single project
struct GetProjectHandler;

#[async_trait]
impl ApiHandler for GetProjectHandler {
    async fn handle(&self, req: Request) -> Result<ApiResponse, ApiError> {
        let project_id: u32 = req.param("id")
            .and_then(|id| id.parse().ok())
            .ok_or_else(|| ApiError::bad_request("Invalid project ID"))?;

        let projects = PROJECTS.lock().unwrap();
        if let Some(project) = projects.iter().find(|p| p.id == project_id) {
            Ok(ApiResponse::ok(serde_json::to_value(project).unwrap()))
        } else {
            Err(ApiError::not_found(&format!("Project with ID {} not found", project_id)))
        }
    }
}

// API Handler for creating projects
struct CreateProjectHandler;

#[async_trait]
impl ApiHandler for CreateProjectHandler {
    async fn handle(&self, mut req: Request) -> Result<ApiResponse, ApiError> {
        let form_data = req.form().await.map_err(|e| ApiError::bad_request(&format!("Failed to parse form data: {}", e)))?;
        
        let name = form_data.get("name").map(|s| s.trim()).filter(|s| !s.is_empty());
        let description = form_data.get("description").map(|s| s.trim()).filter(|s| !s.is_empty());
        let status = form_data.get("status").map(|s| s.trim()).filter(|s| !s.is_empty());

        if name.is_none() || description.is_none() || status.is_none() {
            return Err(ApiError::bad_request("Name, description, and status are required."));
        }

        let mut projects = PROJECTS.lock().unwrap();
        let new_id = projects.iter().map(|p| p.id).max().unwrap_or(0) + 1;

        let new_project = Project {
            id: new_id,
            name: name.unwrap().to_string(),
            description: description.unwrap().to_string(),
            status: status.unwrap().to_string(),
            created_at: chrono::Utc::now().format("%Y-%m-%d").to_string(),
            tasks: Vec::new(),
        };
        projects.push(new_project.clone());
        info!("New project created: {:?}", new_project);

        Ok(ApiResponse::ok(json!({"message": "Project created successfully", "project_id": new_project.id}))
            .header("Location", &format!("/projects/{}", new_project.id))
            .header("HX-Redirect", &format!("/projects/{}", new_project.id))
            .with_status(StatusCode::SEE_OTHER))
    }
}

// API Handler for creating tasks within a project
struct CreateTaskHandler;

#[async_trait]
impl ApiHandler for CreateTaskHandler {
    async fn handle(&self, mut req: Request) -> Result<ApiResponse, ApiError> {
        let project_id: u32 = req.param("id")
            .and_then(|id| id.parse().ok())
            .ok_or_else(|| ApiError::bad_request("Invalid project ID"))?;

        let form_data = req.form().await.map_err(|e| ApiError::bad_request(&format!("Failed to parse form data: {}", e)))?;
        
        let name = form_data.get("task_name").map(|s| s.trim()).filter(|s| !s.is_empty());
        let description = form_data.get("task_description").map(|s| s.trim()).filter(|s| !s.is_empty());
        let due_date = form_data.get("due_date").map(|s| s.trim()).filter(|s| !s.is_empty());

        if name.is_none() || description.is_none() || due_date.is_none() {
            return Err(ApiError::bad_request("Task name, description, and due date are required."));
        }

        let mut projects = PROJECTS.lock().unwrap();
        if let Some(project) = projects.iter_mut().find(|p| p.id == project_id) {
            let new_task_id = project.tasks.iter().map(|t| t.id).max().unwrap_or(0) + 1;
            let new_task = Task {
                id: new_task_id,
                name: name.unwrap().to_string(),
                description: description.unwrap().to_string(),
                completed: false,
                due_date: due_date.unwrap().to_string(),
            };
            project.tasks.push(new_task.clone());
            info!("New task created for project {}: {:?}", project_id, new_task);

            Ok(ApiResponse::ok(json!({"message": "Task created successfully", "task_id": new_task.id}))
                .header("Location", &format!("/projects/{}", project_id))
                .header("HX-Redirect", &format!("/projects/{}", project_id))
                .with_status(StatusCode::SEE_OTHER))
        } else {
            Err(ApiError::not_found(&format!("Project with ID {} not found", project_id)))
        }
    }
}

// API Handler for toggling task completion
struct ToggleTaskHandler;

#[async_trait]
impl ApiHandler for ToggleTaskHandler {
    async fn handle(&self, req: Request) -> Result<ApiResponse, ApiError> {
        let project_id: u32 = req.param("project_id")
            .and_then(|id| id.parse().ok())
            .ok_or_else(|| ApiError::bad_request("Invalid project ID"))?;
        let task_id: u32 = req.param("task_id")
            .and_then(|id| id.parse().ok())
            .ok_or_else(|| ApiError::bad_request("Invalid task ID"))?;

        let mut projects = PROJECTS.lock().unwrap();
        if let Some(project) = projects.iter_mut().find(|p| p.id == project_id) {
            if let Some(task) = project.tasks.iter_mut().find(|t| t.id == task_id) {
                task.completed = !task.completed;
                info!("Task {} in project {} toggled to completed={}", task_id, project_id, task.completed);
                Ok(ApiResponse::ok(json!({"message": "Task toggled successfully", "task": task})))
            } else {
                Err(ApiError::not_found(&format!("Task with ID {} not found in project {}", task_id, project_id)))
            }
        } else {
            Err(ApiError::not_found(&format!("Project with ID {} not found", project_id)))
        }
    }
}

// API Handler for deleting tasks
struct DeleteTaskHandler;

#[async_trait]
impl ApiHandler for DeleteTaskHandler {
    async fn handle(&self, req: Request) -> Result<ApiResponse, ApiError> {
        let project_id: u32 = req.param("project_id")
            .and_then(|id| id.parse().ok())
            .ok_or_else(|| ApiError::bad_request("Invalid project ID"))?;
        let task_id: u32 = req.param("task_id")
            .and_then(|id| id.parse().ok())
            .ok_or_else(|| ApiError::bad_request("Invalid task ID"))?;

        let mut projects = PROJECTS.lock().unwrap();
        if let Some(project) = projects.iter_mut().find(|p| p.id == project_id) {
            let initial_len = project.tasks.len();
            project.tasks.retain(|t| t.id != task_id);

            if project.tasks.len() < initial_len {
                info!("Task with ID {} deleted from project {}", task_id, project_id);
                Ok(ApiResponse::ok(json!({"message": "Task deleted successfully"})))
            } else {
                Err(ApiError::not_found(&format!("Task with ID {} not found in project {}", task_id, project_id)))
            }
        } else {
            Err(ApiError::not_found(&format!("Project with ID {} not found", project_id)))
        }
    }
}


// Dashboard Layout Component
component!(DashboardLayout, props => {
    let title = props.get("title").and_then(|v| v.as_str()).unwrap_or("RustNext Dashboard");
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
                                .child(a().prop("href", "/").class(if active_path == "/" { "active" } else { "" }).child(text("Dashboard")))
                                .child(a().prop("href", "/projects/new").class(if active_path == "/projects/new" { "active" } else { "" }).child(text("New Project")))
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
                        .child(text("© 2024 RustNext Project Dashboard. Built with Rust."))
                )
        )
});

// Project Card Component
component!(ProjectCard, props => {
    let project_id = props.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
    let name = props.get("name").and_then(|v| v.as_str()).unwrap_or("Untitled Project");
    let description = props.get("description").and_then(|v| v.as_str()).unwrap_or("");
    let status = props.get("status").and_then(|v| v.as_str()).unwrap_or("Unknown");
    let created_at = props.get("created_at").and_then(|v| v.as_str()).unwrap_or("");
    
    article()
        .class("card")
        .child(
            h3()
                .child(
                    a()
                        .prop("href", format!("/projects/{}", project_id))
                        .child(text(name))
                )
        )
        .child(
            p().class("text-gray-700 text-sm mt-2")
                .child(text(description))
        )
        .child(
            div()
                .class("flex justify-between items-center mt-4 text-sm")
                .child(
                    span().class("text-gray-600")
                        .child(text(&format!("Status: {}", status)))
                )
                .child(
                    span().class("text-gray-500")
                        .child(text(&format!("Created: {}", created_at)))
                )
        )
        .child(
            div()
                .class("mt-4")
                .child(
                    a()
                        .class("btn btn-secondary text-sm")
                        .prop("href", format!("/projects/{}", project_id))
                        .child(text("View Details"))
                )
        )
});

// Task Item Component
component!(TaskItem, props => {
    let project_id = props.get("project_id").and_then(|v| v.as_u64()).unwrap_or(0);
    let task_id = props.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
    let name = props.get("name").and_then(|v| v.as_str()).unwrap_or("Untitled Task");
    let description = props.get("description").and_then(|v| v.as_str()).unwrap_or("");
    let completed = props.get("completed").and_then(|v| v.as_bool()).unwrap_or(false);
    let due_date = props.get("due_date").and_then(|v| v.as_str()).unwrap_or("");

    let name_class = if completed { "line-through text-gray-500" } else { "text-gray-800 font-semibold" };
    let status_text = if completed { "Completed" } else { "Pending" };

    li()
        .class("flex flex-col sm:flex-row items-start sm:items-center justify-between p-3 border-b border-gray-200 last:border-b-0")
        .child(
            div()
                .class("flex-1 mb-2 sm:mb-0")
                .child(
                    span()
                        .class(name_class)
                        .child(text(name))
                )
                .child(
                    p().class("text-sm text-gray-600 mt-1")
                        .child(text(description))
                )
                .child(
                    span().class("text-xs text-gray-500 mt-1")
                        .child(text(&format!("Due: {}", due_date)))
                )
        )
        .child(
            div()
                .class("flex items-center gap-3")
                .child(
                    span().class("text-sm")
                        .child(text(status_text))
                )
                .child(
                    a() // Link to toggle completion
                        .prop("href", format!("/api/projects/{}/tasks/{}/toggle", project_id, task_id))
                        .prop("method", "POST") // Use POST for state change
                        .class("btn btn-secondary text-sm")
                        .child(text(if completed { "Mark Pending" } else { "Mark Complete" }))
                )
                .child(
                    a() // Link to delete task
                        .prop("href", format!("/api/projects/{}/tasks/{}/delete", project_id, task_id))
                        .prop("method", "POST") // Use POST for state change
                        .class("btn btn-danger text-sm")
                        .child(text("Delete"))
                )
        )
});

// Project Form Component
component!(ProjectForm, _props => {
    form()
        .prop("method", "POST")
        .prop("action", "/api/projects")
        .class("mt-4 p-4 border border-gray-200 rounded-md bg-gray-50")
        .child(
            div()
                .class("form-group")
                .child(label().prop("for", "name").child(text("Project Name")))
                .child(
                    input()
                        .prop("type", "text")
                        .prop("name", "name")
                        .prop("placeholder", "e.g., Website Redesign")
                        .prop("required", "true")
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
                        .prop("placeholder", "Brief description of the project...")
                        .prop("required", "true")
                        .prop("rows", "3")
                        .class("form-control")
                )
        )
        .child(
            div()
                .class("form-group")
                .child(label().prop("for", "status").child(text("Status")))
                .child(
                    Element::new("select")
                        .prop("name", "status")
                        .prop("required", "true")
                        .class("form-control")
                        .child(Element::new("option").prop("value", "").child(text("Select Status")))
                        .child(Element::new("option").prop("value", "Planned").child(text("Planned")))
                        .child(Element::new("option").prop("value", "In Progress").child(text("In Progress")))
                        .child(Element::new("option").prop("value", "Completed").child(text("Completed")))
                        .child(Element::new("option").prop("value", "On Hold").child(text("On Hold")))
                )
        )
        .child(
            button()
                .prop("type", "submit")
                .class("btn mt-2")
                .child(text("Create Project"))
        )
});

// Task Form Component
component!(TaskForm, props => {
    let project_id = props.get("project_id").and_then(|v| v.as_u64()).unwrap_or(0);
    form()
        .prop("method", "POST")
        .prop("action", format!("/api/projects/{}/tasks", project_id))
        .class("mt-4 p-4 border border-gray-200 rounded-md bg-gray-50")
        .child(
            h3().class("text-lg font-bold mb-3").child(text("Add New Task"))
        )
        .child(
            div()
                .class("form-group")
                .child(label().prop("for", "task_name").child(text("Task Name")))
                .child(
                    input()
                        .prop("type", "text")
                        .prop("name", "task_name")
                        .prop("placeholder", "e.g., Implement User Auth")
                        .prop("required", "true")
                        .class("form-control")
                )
        )
        .child(
            div()
                .class("form-group")
                .child(label().prop("for", "task_description").child(text("Description")))
                .child(
                    Element::new("textarea")
                        .prop("name", "task_description")
                        .prop("placeholder", "Detailed description of the task...")
                        .prop("required", "true")
                        .prop("rows", "2")
                        .class("form-control")
                )
        )
        .child(
            div()
                .class("form-group")
                .child(label().prop("for", "due_date").child(text("Due Date")))
                .child(
                    input()
                        .prop("type", "date")
                        .prop("name", "due_date")
                        .prop("required", "true")
                        .class("form-control")
                )
        )
        .child(
            button()
                .prop("type", "submit")
                .class("btn mt-2")
                .child(text("Add Task"))
        )
});


// Dashboard Home Page
page!(ProjectDashboardPage, req => {
    let projects_cloned = {
        PROJECTS.lock().unwrap().clone()
    };

    let mut project_cards_futures = Vec::new();
    for project in projects_cloned {
        let mut project_props = HashMap::new();
        project_props.insert("id".to_string(), json!(project.id));
        project_props.insert("name".to_string(), json!(project.name));
        project_props.insert("description".to_string(), json!(project.description));
        project_props.insert("status".to_string(), json!(project.status));
        project_props.insert("created_at".to_string(), json!(project.created_at));
        
        let component_registry_arc = get_component_registry().clone();
        project_cards_futures.push(async move {
            let component_registry = component_registry_arc.lock().await;
            component_registry.render("project_card", &project_props).await.unwrap_or_else(|| div())
        });
    }

    // Await all futures to get the rendered elements
    let project_cards = futures::future::join_all(project_cards_futures).await;

    let content = section()
        .child(h2().class("text-2xl font-bold mb-4").child(text("All Projects")))
        .child(
            div()
                .class("grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6")
                .children(project_cards)
        );

    let mut layout_props = HashMap::new();
    layout_props.insert("title".to_string(), json!("Project Dashboard"));
    layout_props.insert("children_html".to_string(), json!(get_renderer().render_to_html(&content)));
    layout_props.insert("active_path".to_string(), json!(req.uri.path()));

    // Check for messages after redirect
    if let Some(error_msg) = req.query_param("error") {
        layout_props.insert("error_message".to_string(), json!(urlencoding::decode(error_msg).unwrap_or_default()));
    }
    if let Some(success_msg) = req.query_param("success") {
        layout_props.insert("success_message".to_string(), json!(urlencoding::decode(success_msg).unwrap_or_default()));
    }
    
    let component_registry = get_component_registry().lock().await;
    let rendered_element = component_registry.render("dashboard_layout", &layout_props).await;
    rendered_element.unwrap_or_else(|| {
        div().child(text("Error rendering page"))
    })
});

// New Project Page
page!(NewProjectPage, req => {
    let mut layout_props = HashMap::new();
    layout_props.insert("title".to_string(), json!("Create New Project"));
    let project_form_element = {
        let component_registry = get_component_registry().lock().await;
        component_registry.render("project_form", &HashMap::new()).await.unwrap_or_else(|| div())
    };
    layout_props.insert("children_html".to_string(), json!(get_renderer().render_to_html(&project_form_element)));
    layout_props.insert("active_path".to_string(), json!(req.uri.path()));

    // Check for messages after redirect
    if let Some(error_msg) = req.query_param("error") {
        layout_props.insert("error_message".to_string(), json!(urlencoding::decode(error_msg).unwrap_or_default()));
    }
    if let Some(success_msg) = req.query_param("success") {
        layout_props.insert("success_message".to_string(), json!(urlencoding::decode(success_msg).unwrap_or_default()));
    }
    
    let component_registry = get_component_registry().lock().await;
    let rendered_element = component_registry.render("dashboard_layout", &layout_props).await;
    rendered_element.unwrap_or_else(|| {
        div().child(text("Error rendering page"))
    })
});

// Project Detail Page
page!(ProjectDetailPage, req => {
    let project_id: u32 = req.param("id")
        .and_then(|id| id.parse().ok())
        .unwrap_or(0);
    
    let project_option = { // Renamed to project_option to avoid confusion with project_clone
        PROJECTS.lock().unwrap().iter().find(|p| p.id == project_id).cloned()
    };

    let content = if let Some(ref project) = project_option { // Changed to `ref project` to borrow
        let mut task_items_futures = Vec::new();
        for task in project.tasks.clone() { // Clone tasks to iterate
            let mut task_props = HashMap::new();
            task_props.insert("id".to_string(), json!(task.id));
            task_props.insert("project_id".to_string(), json!(project.id)); // Pass project_id to task
            task_props.insert("name".to_string(), json!(task.name));
            task_props.insert("description".to_string(), json!(task.description));
            task_props.insert("completed".to_string(), json!(task.completed));
            task_props.insert("due_date".to_string(), json!(task.due_date));

            let component_registry_arc = get_component_registry().clone();
            task_items_futures.push(async move {
                let component_registry = component_registry_arc.lock().await;
                component_registry.render("task_item", &task_props).await.unwrap_or_else(|| div())
            });
        }
        let task_items = futures::future::join_all(task_items_futures).await;

        let mut task_form_props = HashMap::new();
        task_form_props.insert("project_id".to_string(), json!(project.id));
        let task_form_element = {
            let component_registry = get_component_registry().lock().await;
            component_registry.render("task_form", &task_form_props).await.unwrap_or_else(|| {
                div().child(text("Error rendering task form"))
            })
        };

        div()
            .child(
                article()
                    .class("card mb-6")
                    .child(h2().class("text-2xl font-bold mb-2").child(text(&project.name)))
                    .child(p().class("text-gray-700 mb-3").child(text(&project.description)))
                    .child(
                        div()
                            .class("flex justify-between items-center text-sm text-gray-600")
                            .child(span().child(text(&format!("Status: {}", project.status))))
                            .child(span().child(text(&format!("Created: {}", project.created_at))))
                    )
                    .child(
                        div()
                            .class("mt-6")
                            .child(a().class("btn btn-secondary").prop("href", "/").child(text("← Back to Dashboard")))
                    )
            )
            .child(
                section()
                    .class("card")
                    .child(h2().class("text-xl font-bold mb-4").child(text("Tasks")))
                    .child(
                        ul().children(task_items)
                    )
            )
            .child(task_form_element)
    } else {
        div()
            .class("card")
            .child(h1().child(text("Project Not Found")))
            .child(p().child(text("The requested project could not be found.")))
            .child(a().class("btn").prop("href", "/").child(text("← Back to Dashboard")))
    };
    
    let mut layout_props = HashMap::new();
    layout_props.insert("title".to_string(), json!(
        project_option.map(|p| p.name).unwrap_or_else(|| "Project Not Found".to_string())
    ));
    layout_props.insert("children_html".to_string(), json!(get_renderer().render_to_html(&content)));
    layout_props.insert("active_path".to_string(), json!(req.uri.path()));

    // Check for messages after redirect
    if let Some(error_msg) = req.query_param("error") {
        layout_props.insert("error_message".to_string(), json!(urlencoding::decode(error_msg).unwrap_or_default()));
    }
    if let Some(success_msg) = req.query_param("success") {
        layout_props.insert("success_message".to_string(), json!(urlencoding::decode(success_msg).unwrap_or_default()));
    }
    
    let component_registry = get_component_registry().lock().await;
    let rendered_element = component_registry.render("dashboard_layout", &layout_props).await;
    rendered_element.unwrap_or_else(|| {
        div().child(text("Error rendering page"))
    })
});

// About Page for Dashboard App
page!(AboutPage, req => {
    let content = section()
        .child(
            h2().class("text-2xl font-bold mb-4").child(text("About This Project Dashboard App"))
        )
        .child(
            div()
                .class("card")
                .child(
                    p().child(text("This is a simple Project Management Dashboard application built with RustNext."))
                )
                .child(
                    p().child(text("It demonstrates full-stack capabilities including: "))
                )
                .child(
                    ul()
                        .class("list-disc pl-5 space-y-2")
                        .child(li().child(text("UI Components for Projects and Tasks")))
                        .child(li().child(text("API Endpoints for managing Projects and Tasks (CRUD operations)")))
                        .child(li().child(text("In-memory data storage (for simplicity, can be extended with a database)")))
                        .child(li().child(text("Form submission and POST-redirect-GET pattern for robust data handling")))
                        .child(li().child(text("Improved UI/UX with modern CSS styling")))
                        .child(li().child(text("Error and success message display after redirects")))
                )
                .child(
                    p().class("mt-4").child(text("This application serves as a more comprehensive example of building interactive web applications with RustNext, showcasing how different parts of the framework work together."))
                )
        );

    let mut layout_props = HashMap::new();
    layout_props.insert("title".to_string(), json!("About Project Dashboard"));
    layout_props.insert("children_html".to_string(), json!(get_renderer().render_to_html(&content)));
    layout_props.insert("active_path".to_string(), json!(req.uri.path()));

    let component_registry = get_component_registry().lock().await;
    let rendered_element = component_registry.render("dashboard_layout", &layout_props).await;
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
    info!("   API endpoints: /api/projects, /api/projects/:id, /api/projects/:id/tasks, /api/projects/:project_id/tasks/:task_id/toggle, /api/projects/:project_id/tasks/:task_id/delete");
    info!("");
    info!("🎯 Features Demonstrated:");
    info!("   ✅ Enhanced UI/UX with modern CSS styling");
    info!("   ✅ Multi-entity data model (Projects & Tasks)");
    info!("   ✅ CRUD operations via API routes");
    info!("   ✅ Forms for creating Projects and Tasks");
    info!("   ✅ POST-redirect-GET pattern for form submissions");
    info!("   ✅ Display of error/success messages after redirects");
    
    // Register components
    register_component!("dashboard_layout", DashboardLayout).await?;
    register_component!("project_card", ProjectCard).await?;
    register_component!("project_form", ProjectForm).await?;
    register_component!("task_item", TaskItem).await?;
    register_component!("task_form", TaskForm).await?;
    
    // Register pages
    register_page!("/", ProjectDashboardPage).await?;
    register_page!("/projects/new", NewProjectPage).await?;
    register_page!("/projects/:id", ProjectDetailPage).await?;
    register_page!("/about", AboutPage).await?;

    // Register API routes
    api_route!(hyper::Method::GET, "/api/projects", GetProjectsHandler).await?;
    api_route!(hyper::Method::GET, "/api/projects/:id", GetProjectHandler).await?;
    api_route!(hyper::Method::POST, "/api/projects", CreateProjectHandler).await?;
    api_route!(hyper::Method::POST, "/api/projects/:id/tasks", CreateTaskHandler).await?;
    api_route!(hyper::Method::POST, "/api/projects/:project_id/tasks/:task_id/toggle", ToggleTaskHandler).await?;
    api_route!(hyper::Method::POST, "/api/projects/:project_id/tasks/:task_id/delete", DeleteTaskHandler).await?;

    // Create router
    let router = Router::new()
        .use_middleware(RateLimiter::new(100, 60))
        .get("/", |req| async move {
            let page_registry = get_page_registry().lock().await;
            let element_option = page_registry.render_page("/", &req).await;
            if let Some(element) = element_option {
                get_renderer().render_to_response(&element)
            } else {
                Err(Box::new(AppError::NotFound("Dashboard home page not found".to_string())) as Box<dyn std::error::Error + Send + Sync>)
            }
        })
        .get("/projects/new", |req| async move {
            let page_registry = get_page_registry().lock().await;
            let element_option = page_registry.render_page("/projects/new", &req).await;
            if let Some(element) = element_option {
                get_renderer().render_to_response(&element)
            } else {
                Err(Box::new(AppError::NotFound("New project page not found".to_string())) as Box<dyn std::error::Error + Send + Sync>)
            }
        })
        .get("/projects/:id", |req| async move {
            let page_registry = get_page_registry().lock().await;
            let element_option = page_registry.render_page("/projects/:id", &req).await;
            if let Some(element) = element_option {
                get_renderer().render_to_response(&element)
            } else {
                Err(Box::new(AppError::NotFound(format!("Project {} not found", req.param("id").unwrap_or(&"unknown".to_string())))) as Box<dyn std::error::Error + Send + Sync>)
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
        .post("/api/projects", |req: Request| async move {
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
                            .header("Location", &format!("/projects/new?error={}", urlencoding::encode(&error_msg)))
                            .header("HX-Redirect", &format!("/projects/new?error={}", urlencoding::encode(&error_msg))))
                    }
                }
                None => Err(Box::new(AppError::NotFound("API endpoint /api/projects (POST) not found".to_string())) as Box<dyn std::error::Error + Send + Sync>),
            }
        })
        .post("/api/projects/:id/tasks", |req: Request| async move {
            let api_registry = get_api_registry().lock().await;
            let project_id_str = req.param("id").cloned().unwrap_or_default();
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
                            .header("Location", &format!("/projects/{}?error={}", project_id_str, urlencoding::encode(&error_msg)))
                            .header("HX-Redirect", &format!("/projects/{}?error={}", project_id_str, urlencoding::encode(&error_msg))))
                    }
                }
                None => Err(Box::new(AppError::NotFound("API endpoint /api/projects/:id/tasks (POST) not found".to_string())) as Box<dyn std::error::Error + Send + Sync>),
            }
        })
        .post("/api/projects/:project_id/tasks/:task_id/toggle", |req: Request| async move {
            let api_registry = get_api_registry().lock().await;
            let project_id_str = req.param("project_id").cloned().unwrap_or_default();
            match api_registry.handle_request(req).await {
                Some(_response) => {
                    // After toggle, redirect back to project detail to show updated list
                    Ok(Response::new().status(StatusCode::SEE_OTHER).header("Location", &format!("/projects/{}", project_id_str)))
                }
                None => Err(Box::new(AppError::NotFound("API endpoint /api/projects/:project_id/tasks/:task_id/toggle (POST) not found".to_string())) as Box<dyn std::error::Error + Send + Sync>),
            }
        })
        .post("/api/projects/:project_id/tasks/:task_id/delete", |req: Request| async move {
            let api_registry = get_api_registry().lock().await;
            let project_id_str = req.param("project_id").cloned().unwrap_or_default();
            match api_registry.handle_request(req).await {
                Some(_response) => {
                    // After delete, redirect back to project detail to show updated list
                    Ok(Response::new().status(StatusCode::SEE_OTHER).header("Location", &format!("/projects/{}", project_id_str)))
                }
                None => Err(Box::new(AppError::NotFound("API endpoint /api/projects/:project_id/tasks/:task_id/delete (POST) not found".to_string())) as Box<dyn std::error::Error + Send + Sync>),
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
    
    info!("🚀 RustNext Project Dashboard running at http://{}:{}", config.server.host, config.server.port);
    info!("📝 Available routes:");
    info!("   http://{}:{}/           - Project dashboard home page", config.server.host, config.server.port);
    info!("   http://{}:{}/projects/new - Create new project form", config.server.host, config.server.port);
    info!("   http://{}:{}/projects/1  - Individual project detail page", config.server.host, config.server.port);
    info!("   http://{}:{}/about      - About page", config.server.host, config.server.port);
    info!("   API endpoints: /api/projects, /api/projects/:id, /api/projects/:id/tasks, /api/projects/:project_id/tasks/:task_id/toggle, /api/projects/:project_id/tasks/:task_id/delete");
    info!("");
    info!("🎯 Features Demonstrated:");
    info!("   ✅ Enhanced UI/UX with modern CSS styling");
    info!("   ✅ Multi-entity data model (Projects & Tasks)");
    info!("   ✅ CRUD operations via API routes");
    info!("   ✅ Forms for creating Projects and Tasks");
    info!("   ✅ POST-redirect-GET pattern for form submissions");
    info!("   ✅ Display of error/success messages after redirects");
    
    server.run().await
}
