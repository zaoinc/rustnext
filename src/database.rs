#[cfg(feature = "database")] // Conditional compilation
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use once_cell::sync::OnceCell;
use log::{info, warn}; // New import for logging

#[cfg(feature = "database")]
#[derive(Clone)]
pub struct Database {
    pool: Arc<Pool<Postgres>>,
}

#[cfg(feature = "database")]
impl Database {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;
        
        Ok(Database {
            pool: Arc::new(pool),
        })
    }

    pub async fn execute(&self, query: &str) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(query).execute(&*self.pool).await?;
        Ok(result.rows_affected())
    }

    pub async fn fetch_one(&self, query: &str) -> Result<sqlx::postgres::PgRow, sqlx::Error> {
        sqlx::query(query).fetch_one(&*self.pool).await
    }

    pub async fn fetch_all(&self, query: &str) -> Result<Vec<sqlx::postgres::PgRow>, sqlx::Error> {
        sqlx::query(query).fetch_all(&*self.pool).await
    }
}

#[cfg(feature = "database")]
static GLOBAL_DATABASE: OnceCell<Database> = OnceCell::new();

#[cfg(feature = "database")]
pub async fn init_database(database_url: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let db = Database::new(database_url).await?;
    if GLOBAL_DATABASE.set(db).is_err() {
        warn!("Database already initialized, ignoring new initialization.");
    } else {
        info!("Database pool initialized.");
    }
    Ok(())
}

#[cfg(feature = "database")]
pub fn get_database() -> Option<&'static Database> {
    GLOBAL_DATABASE.get()
}

// Dummy implementations if database feature is not enabled
#[cfg(not(feature = "database"))]
pub struct Database;
#[cfg(not(feature = "database"))]
impl Database {
    pub async fn new(_database_url: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Err("Database feature not enabled".into())
    }
}
#[cfg(not(feature = "database"))]
pub async fn init_database(_database_url: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    log::warn!("Attempted to initialize database, but 'database' feature is not enabled.");
    Ok(())
}
#[cfg(not(feature = "database"))]
pub fn get_database() -> Option<&'static Database> {
    None
}