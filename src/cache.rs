#[cfg(feature = "cache")] // Conditional compilation
use redis::{AsyncCommands, Client};
use std::time::Duration;
use once_cell::sync::OnceCell;
use log::{info, warn}; // New import for logging

#[cfg(feature = "cache")]
#[derive(Clone)]
pub struct Cache {
    client: Client,
}

#[cfg(feature = "cache")]
impl Cache {
    pub async fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        let client = Client::open(redis_url)?;
        Ok(Cache { client })
    }

    pub async fn get<T: for<'de> serde::Deserialize<'de>>(&self, key: &str) -> Result<Option<T>, Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.client.get_async_connection().await?;
        let value: Option<String> = conn.get(key).await?;
        
        match value {
            Some(v) => Ok(Some(serde_json::from_str(&v)?)),
            None => Ok(None),
        }
    }

    pub async fn set<T: serde::Serialize>(&self, key: &str, value: &T, ttl: Duration) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.client.get_async_connection().await?;
        let serialized = serde_json::to_string(value)?;
        conn.set_ex::<_, _, ()>(key, serialized, ttl.as_secs().try_into().unwrap()).await?;
        Ok(())
    }

    pub async fn delete(&self, key: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.client.get_async_connection().await?;
        conn.del::<_, ()>(key).await?;
        Ok(())
    }
}

#[cfg(feature = "cache")]
static GLOBAL_CACHE: OnceCell<Cache> = OnceCell::new();

#[cfg(feature = "cache")]
pub async fn init_cache(redis_url: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cache = Cache::new(redis_url).await?;
    if GLOBAL_CACHE.set(cache).is_err() {
        warn!("Cache already initialized, ignoring new initialization.");
    } else {
        info!("Cache client initialized.");
    }
    Ok(())
}

#[cfg(feature = "cache")]
pub fn get_cache() -> Option<&'static Cache> {
    GLOBAL_CACHE.get()
}

// Dummy implementations if cache feature is not enabled
#[cfg(not(feature = "cache"))]
pub struct Cache;
#[cfg(not(feature = "cache"))]
impl Cache {
    pub async fn new(_redis_url: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Err("Cache feature not enabled".into())
    }
}
#[cfg(not(feature = "cache"))]
pub async fn init_cache(_redis_url: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    log::warn!("Attempted to initialize cache, but 'cache' feature is not enabled.");
    Ok(())
}
#[cfg(not(feature = "cache"))]
pub fn get_cache() -> Option<&'static Cache> {
    None
}
