use moka::future::Cache as MokaCache;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::{HashMap, VecDeque};
use std::time::Duration;
use std::env;
use serde_json::Value as JsonValue;

#[derive(Debug, Clone)]
pub struct OrderedCache {
    caches: HashMap<String, Arc<MokaCache<String, JsonValue>>>,
    orders: HashMap<String, Arc<Mutex<VecDeque<String>>>>,
    // Track recently added items to prevent duplicates
    recently_added: HashMap<String, Arc<MokaCache<String, ()>>>,
}

impl OrderedCache {
    pub fn new(aliases: Vec<String>) -> Self {
        let ttl_seconds: u64 = env::var("CACHE_TTL")
            .ok()
            .and_then(|val| val.parse::<u64>().ok())
            .unwrap_or(300);


        let recently_added_ttl_seconds: u64 = 200; // around 3 minutes default

        let mut caches = HashMap::new();
        let mut orders = HashMap::new();
        let mut recently_added = HashMap::new();

        for alias in aliases {
            let cache = Arc::new(
                MokaCache::builder()
                    .max_capacity(10_000)
                    .time_to_live(Duration::from_secs(ttl_seconds))
                    .build()
            );
            let order = Arc::new(Mutex::new(VecDeque::new()));

            // Cache for recently added items
            let added_cache = Arc::new(
                MokaCache::builder()
                    .max_capacity(10_000)
                    .time_to_live(Duration::from_secs(recently_added_ttl_seconds))
                    .build()
            );

            caches.insert(alias.clone(), cache);
            orders.insert(alias.clone(), order);
            recently_added.insert(alias, added_cache);
        }

        Self { caches, orders, recently_added }
    }

    pub async fn insert(&self, alias: &str, key: String, value: JsonValue) -> Result<(), &'static str> {
        let cache = self.caches.get(alias).ok_or("Alias not found")?;
        let order = self.orders.get(alias).ok_or("Alias not found")?;
        let recently_added_cache = self.recently_added.get(alias).ok_or("Alias not found")?;

        // Check if this key was recently added
        if recently_added_cache.get(&key).await.is_some() {
            // Skip insertion if it was recently added
            return Ok(());
        }

        // Add to recently added cache first to prevent race conditions
        recently_added_cache.insert(key.clone(), ()).await;

        // Add to main cache
        cache.insert(key.clone(), value).await;

        let mut order = order.lock().await;
        if let Some(pos) = order.iter().position(|k| k == &key) {
            order.remove(pos);
        }
        order.push_back(key);

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn get(&self, alias: &str, key: &str) -> Option<JsonValue> {
        self.caches.get(alias)?.get(key).await
    }

    pub async fn remove_oldest(&self, alias: &str, n: usize) -> Result<Vec<(String, JsonValue)>, &'static str> {
        let cache = self.caches.get(alias).ok_or("Alias not found")?;
        let order = self.orders.get(alias).ok_or("Alias not found")?;

        let mut order = order.lock().await;
        let mut removed = Vec::new();

        while removed.len() < n && !order.is_empty() {
            if let Some(key) = order.pop_front() {
                if let Some(value) = cache.get(&key).await {
                    removed.push((key.clone(), value));
                    cache.invalidate(&key).await;
                }
            }
        }

        Ok(removed)
    }

    #[allow(dead_code)]
    pub async fn remove_newest(&self, alias: &str, n: usize) -> Result<Vec<(String, JsonValue)>, &'static str> {
        let cache = self.caches.get(alias).ok_or("Alias not found")?;
        let order = self.orders.get(alias).ok_or("Alias not found")?;

        let mut order = order.lock().await;
        let mut removed = Vec::new();

        while removed.len() < n && !order.is_empty() {
            if let Some(key) = order.pop_back() {
                if let Some(value) = cache.get(&key).await {
                    removed.push((key.clone(), value));
                    cache.invalidate(&key).await;
                }
            }
        }

        Ok(removed)
    }

    // Check if an alias exists
    pub fn has_alias(&self, alias: &str) -> bool {
        self.caches.contains_key(alias)
    }

    // Get all aliases
    #[allow(dead_code)]
    pub fn get_aliases(&self) -> Vec<String> {
        self.caches.keys().cloned().collect()
    }
}