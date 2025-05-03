use moka::future::Cache as MokaCache;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::{HashMap, VecDeque};
use std::time::Duration;
use std::env;


#[derive(Debug, Clone)]
pub struct OrderedCache {
    caches: HashMap<String, Arc<MokaCache<String, Vec<u8>>>>,
    orders: HashMap<String, Arc<Mutex<VecDeque<String>>>>,
}

impl OrderedCache {
    pub fn new(aliases: Vec<String>) -> Self {
        let ttl_seconds: u64 = env::var("CACHE_TTL")
            .unwrap_or_else(|_| "300".to_string())
            .parse()
            .unwrap_or(300);

        let mut caches = HashMap::new();
        let mut orders = HashMap::new();

        for alias in aliases {
            let cache = Arc::new(
                MokaCache::builder()
                    .max_capacity(10_000)
                    .time_to_live(Duration::from_secs(ttl_seconds))
                    .build()
            );
            let order = Arc::new(Mutex::new(VecDeque::new()));

            caches.insert(alias.clone(), cache);
            orders.insert(alias, order);
        }

        Self { caches, orders }
    }

    pub async fn insert(&self, alias: &str, key: String, value: Vec<u8>) -> Result<(), &'static str> {
        let cache = self.caches.get(alias).ok_or("Alias not found")?;
        let order = self.orders.get(alias).ok_or("Alias not found")?;

        cache.insert(key.clone(), value).await;

        let mut order = order.lock().await;
        if let Some(pos) = order.iter().position(|k| k == &key) {
            order.remove(pos);
        }
        order.push_back(key);

        Ok(())
    }

    pub async fn get(&self, alias: &str, key: &str) -> Option<Vec<u8>> {
        self.caches.get(alias)?.get(key).await
    }

    pub async fn remove_oldest(&self, alias: &str, n: usize) -> Result<Vec<(String, Vec<u8>)>, &'static str> {
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

    pub async fn remove_newest(&self, alias: &str, n: usize) -> Result<Vec<(String, Vec<u8>)>, &'static str> {
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

    // Optional: Add a new alias dynamically
    pub fn add_alias(&mut self, alias: String) {
        let ttl_seconds: u64 = env::var("CACHE_TTL")
            .unwrap_or_else(|_| "300".to_string())
            .parse()
            .unwrap_or(300);

        let cache = Arc::new(
            MokaCache::builder()
                .max_capacity(10_000)
                .time_to_live(Duration::from_secs(ttl_seconds))
                .build()
        );
        let order = Arc::new(Mutex::new(VecDeque::new()));

        self.caches.insert(alias.clone(), cache);
        self.orders.insert(alias, order);
    }

    // Check if an alias exists
    pub fn has_alias(&self, alias: &str) -> bool {
        self.caches.contains_key(alias)
    }

    // Get all aliases
    pub fn get_aliases(&self) -> Vec<String> {
        self.caches.keys().cloned().collect()
    }
}