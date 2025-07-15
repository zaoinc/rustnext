use crate::{Request, Response, middleware::Middleware}; // Corrected import path for Middleware
use async_trait::async_trait;
use cookie::{Cookie, CookieJar};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub data: HashMap<String, serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

impl Session {
    pub fn new(duration: chrono::Duration) -> Self {
        let now = chrono::Utc::now();
        Session {
            id: uuid::Uuid::new_v4().to_string(),
            data: HashMap::new(),
            created_at: now,
            expires_at: now + duration,
        }
    }

    pub fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        self.data.get(key).and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    pub fn set<T: Serialize>(&mut self, key: &str, value: T) -> Result<(), serde_json::Error> {
        self.data.insert(key.to_string(), serde_json::to_value(value)?);
        Ok(())
    }

    pub fn remove(&mut self, key: &str) -> Option<serde_json::Value> {
        self.data.remove(key)
    }

    pub fn is_expired(&self) -> bool {
        chrono::Utc::now() > self.expires_at
    }
}

#[async_trait]
pub trait SessionStore: Send + Sync {
    async fn get(&self, id: &str) -> Result<Option<Session>, Box<dyn std::error::Error + Send + Sync>>;
    async fn set(&self, session: Session) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn delete(&self, id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn cleanup(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

pub struct MemorySessionStore {
    sessions: Arc<tokio::sync::RwLock<HashMap<String, Session>>>,
}

impl MemorySessionStore {
    pub fn new() -> Self {
        MemorySessionStore {
            sessions: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl SessionStore for MemorySessionStore {
    async fn get(&self, id: &str) -> Result<Option<Session>, Box<dyn std::error::Error + Send + Sync>> {
        let sessions = self.sessions.read().await;
        Ok(sessions.get(id).cloned())
    }

    async fn set(&self, session: Session) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut sessions = self.sessions.write().await;
        sessions.insert(session.id.clone(), session);
        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut sessions = self.sessions.write().await;
        sessions.remove(id);
        Ok(())
    }

    async fn cleanup(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut sessions = self.sessions.write().await;
        let now = chrono::Utc::now();
        sessions.retain(|_, session| session.expires_at > now);
        Ok(())
    }
}

pub struct SessionMiddleware {
    store: Arc<dyn SessionStore>,
    cookie_name: String,
    session_duration: chrono::Duration,
}

impl SessionMiddleware {
    pub fn new(store: Arc<dyn SessionStore>) -> Self {
        SessionMiddleware {
            store,
            cookie_name: "rustnext_session".to_string(),
            session_duration: chrono::Duration::hours(24),
        }
    }

    pub fn cookie_name(mut self, name: &str) -> Self {
        self.cookie_name = name.to_string();
        self
    }

    pub fn duration(mut self, duration: chrono::Duration) -> Self {
        self.session_duration = duration;
        self
    }
}

#[async_trait]
impl Middleware for SessionMiddleware {
    async fn handle(
        &self,
        mut req: Request,
        next: Arc<dyn crate::Handler>,
    ) -> Result<Response, Box<dyn std::error::Error + Send + Sync>> {
        // Extract session ID from cookie
        let session_id = req.headers
            .get("cookie")
            .and_then(|cookie_header| cookie_header.to_str().ok())
            .and_then(|cookie_str| {
                let _jar = CookieJar::new(); // Fixed unused variable warning
                for cookie in cookie_str.split(';') {
                    if let Ok(cookie) = Cookie::parse(cookie.trim()) {
                        if cookie.name() == self.cookie_name {
                            return Some(cookie.value().to_string());
                        }
                    }
                }
                None
            });

        // Load or create session
        let session = if let Some(id) = session_id {
            match self.store.get(&id).await? {
                Some(session) if !session.is_expired() => session,
                _ => Session::new(self.session_duration),
            }
        } else {
            Session::new(self.session_duration)
        };

        // Add session to request
        req.session = Some(session.clone());

        // Process request
        let mut response = next.handle(req).await?;

        // Set session cookie
        let cookie = Cookie::build(self.cookie_name.clone(), session.id.clone())
            .http_only(true)
            .secure(false) // Set to true in production with HTTPS
            .same_site(cookie::SameSite::Lax)
            .path("/")
            .finish();

        response.headers.insert("Set-Cookie".to_string(), cookie.to_string());

        // Save session
        self.store.set(session).await?;

        Ok(response)
    }
}
