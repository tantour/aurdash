use anyhow::Result;
use nucleo::{Config, Nucleo};
use raur::{Package, Raur};
use std::sync::Arc;
use tokio::sync::Mutex;

pub use raur::Package as AurPackage;

pub struct AurSearcher {
    raur: raur::Handle,
    // Cache of last search results for local fuzzy re-filter
    cache: Arc<Mutex<Vec<Package>>>,
}

impl AurSearcher {
    pub fn new() -> Self {
        Self {
            raur: raur::Handle::new(),
            cache: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Search AUR and return results, fuzzy-sorted by query
    pub async fn search(&self, query: &str) -> Result<Vec<Package>> {
        if query.len() < 2 {
            return Ok(Vec::new());
        }

        let results = self.raur.search(query).await?;
        // Sort by popularity descending
        let mut sorted = results;
        sorted.sort_by(|a, b| {
            b.popularity
                .partial_cmp(&a.popularity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted.truncate(100); // hard cap

        {
            let mut cache = self.cache.lock().await;
            *cache = sorted.clone();
        }

        Ok(sorted)
    }

    /// Re-filter cached results with nucleo fuzzy matcher (for local instant search)
    pub async fn fuzzy_filter(&self, query: &str) -> Vec<Package> {
        let cache = self.cache.lock().await.clone();
        if query.is_empty() {
            return cache;
        }

        let mut matcher = Nucleo::<String>::new(Config::DEFAULT, Arc::new(|| {}), None, 1);
        let injector = matcher.injector();

        for pkg in &cache {
            let name = pkg.name.clone();
            let _ = injector.push(name.clone(), |val, cols| {
                cols[0] = val.as_str().into();
            });
        }

        // Tick to process
        matcher.tick(10);
        let snapshot = matcher.snapshot();

        let mut result = Vec::new();
        for item in snapshot.matched_items(..) {
            if let Some(pkg) = cache.iter().find(|p| p.name == *item.data) {
                result.push(pkg.clone());
            }
        }

        if result.is_empty() {
            // Fall back to simple contains match
            cache
                .into_iter()
                .filter(|p| p.name.to_lowercase().contains(&query.to_lowercase()))
                .collect()
        } else {
            result
        }
    }
}
