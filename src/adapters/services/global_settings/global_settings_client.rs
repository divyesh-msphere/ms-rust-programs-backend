use crate::config::GlobalSettingsConfig;
use crate::domain::library_catalog::{CatalogError, Library, LibraryCatalog, LibraryOption};
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tracing::warn;
use uuid::Uuid;

/// A cached library may be served for this many multiples of the soft TTL while global-settings
/// is unreachable, so a short outage never blanks the page.
const HARD_TTL_FACTOR: u32 = 40;

/// Reads libraries from global-settings, tuned for the page-load path:
///
///   * a cached library is returned from memory with no network call;
///   * once it is older than the soft TTL it is STILL returned immediately, and refreshed in
///     the background (stale-while-revalidate), so no reader ever waits on a refresh;
///   * only a library that has never been read (or is past the hard TTL) costs one network call.
pub struct GlobalSettingsClient {
    inner: Arc<Inner>,
}

struct Inner {
    http: reqwest::Client,
    base_url: String,
    enabled: bool,
    soft_ttl: Duration,
    hard_ttl: Duration,
    headers: Vec<(String, String)>,
    cache: RwLock<HashMap<Uuid, Entry>>,
}

struct Entry {
    library: Arc<Library>,
    fetched: Instant,
    /// Set while a background refresh is running, so only one runs per library.
    refreshing: AtomicBool,
}

#[derive(Debug, PartialEq, Eq)]
enum Freshness {
    Fresh,
    /// Served as is, and refreshed in the background.
    Stale,
    /// Too old to serve: read it again before answering.
    Expired,
}

fn freshness(age: Duration, soft: Duration, hard: Duration) -> Freshness {
    if age < soft {
        Freshness::Fresh
    } else if age < hard {
        Freshness::Stale
    } else {
        Freshness::Expired
    }
}

impl GlobalSettingsClient {
    pub fn new(config: &GlobalSettingsConfig) -> Self {
        let soft_ttl = Duration::from_secs(config.cache_seconds.max(1));
        let headers = [("x-client-id", &config.client_id), ("x-client-secret", &config.client_secret)]
            .into_iter()
            .filter(|(_, value)| !value.is_empty())
            .map(|(name, value)| (name.to_string(), value.clone()))
            .collect();

        let http = reqwest::Client::builder()
            .timeout(Duration::from_millis(config.timeout_ms.max(100)))
            .pool_idle_timeout(Duration::from_secs(60))
            .build()
            .expect("failed to build the global-settings HTTP client");

        Self {
            inner: Arc::new(Inner {
                http,
                base_url: config.base_url.trim_end_matches('/').to_string(),
                enabled: config.enabled,
                soft_ttl,
                hard_ttl: soft_ttl * HARD_TTL_FACTOR,
                headers,
                cache: RwLock::new(HashMap::new()),
            }),
        }
    }
}

impl Inner {
    fn store(&self, library: Arc<Library>) {
        let entry = Entry { library: library.clone(), fetched: Instant::now(), refreshing: AtomicBool::new(false) };
        self.cache.write().unwrap_or_else(|e| e.into_inner()).insert(library.id, entry);
    }

    fn forget(&self, library_id: Uuid) {
        self.cache.write().unwrap_or_else(|e| e.into_inner()).remove(&library_id);
    }

    async fn fetch(&self, library_id: Uuid) -> Result<Arc<Library>, CatalogError> {
        let url = format!("{}/libraries/{library_id}?include_options=true", self.base_url);
        let mut request = self.http.get(&url).header("accept", "application/json");
        for (name, value) in &self.headers {
            request = request.header(name.as_str(), value.as_str());
        }

        let response = request.send().await.map_err(|err| CatalogError::Unavailable(err.to_string()))?;
        let status = response.status();
        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(CatalogError::NotFound);
        }
        if !status.is_success() {
            return Err(CatalogError::Unavailable(format!("global-settings answered {status}")));
        }

        let body: Value = response.json().await.map_err(|err| CatalogError::Unavailable(err.to_string()))?;
        Ok(Arc::new(parse_library(&body, library_id)))
    }

    /// Reads a library and caches it; a library that no longer exists is evicted.
    async fn load(&self, library_id: Uuid) -> Result<Arc<Library>, CatalogError> {
        match self.fetch(library_id).await {
            Ok(library) => {
                self.store(library.clone());
                Ok(library)
            }
            Err(CatalogError::NotFound) => {
                self.forget(library_id);
                Err(CatalogError::NotFound)
            }
            Err(err) => Err(err),
        }
    }
}

/// `{id, name, code, selection_mode, options: [{id, name, code, active, ...}]}`, as served by
/// `GET /libraries/{library}?include_options=true`.
fn parse_library(body: &Value, requested: Uuid) -> Library {
    let text = |value: &Value, key: &str| value.get(key).and_then(Value::as_str).unwrap_or_default().to_string();

    let options = body
        .get("options")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| {
                    let id = item.get("id").and_then(Value::as_str).and_then(|raw| Uuid::parse_str(raw).ok())?;
                    Some(LibraryOption {
                        id,
                        code: text(item, "code"),
                        name: text(item, "name"),
                        active: item.get("active").and_then(Value::as_bool).unwrap_or(true),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    Library {
        id: body.get("id").and_then(Value::as_str).and_then(|raw| Uuid::parse_str(raw).ok()).unwrap_or(requested),
        name: text(body, "name"),
        code: text(body, "code"),
        selection_mode: match text(body, "selection_mode") {
            mode if mode.is_empty() => "single".to_string(),
            mode => mode,
        },
        options,
    }
}

#[async_trait]
impl LibraryCatalog for GlobalSettingsClient {
    fn enabled(&self) -> bool {
        self.inner.enabled
    }

    async fn get(&self, library_id: Uuid) -> Result<Arc<Library>, CatalogError> {
        if !self.inner.enabled {
            return Err(CatalogError::Unavailable("global-settings integration is switched off".to_string()));
        }

        // Fast path: memory only.
        let hit = {
            let cache = self.inner.cache.read().unwrap_or_else(|e| e.into_inner());
            cache.get(&library_id).map(|entry| {
                let state = freshness(entry.fetched.elapsed(), self.inner.soft_ttl, self.inner.hard_ttl);
                let start_refresh = state == Freshness::Stale
                    && entry.refreshing.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire).is_ok();
                (entry.library.clone(), state, start_refresh)
            })
        };

        match hit {
            Some((library, Freshness::Fresh, _)) => Ok(library),
            Some((library, Freshness::Stale, start_refresh)) => {
                if start_refresh {
                    let inner = self.inner.clone();
                    tokio::spawn(async move {
                        if let Err(err) = inner.load(library_id).await {
                            warn!(%library_id, error = ?err, "background refresh of a library failed; serving the cached copy");
                            // Let the next read try again.
                            if let Some(entry) = inner.cache.read().unwrap_or_else(|e| e.into_inner()).get(&library_id) {
                                entry.refreshing.store(false, Ordering::Release);
                            }
                        }
                    });
                }
                Ok(library)
            }
            // Never read, or too old to trust: one network call, then it is cached.
            _ => self.inner.load(library_id).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn freshness_has_three_states() {
        let (soft, hard) = (Duration::from_secs(10), Duration::from_secs(400));
        assert_eq!(freshness(Duration::from_secs(0), soft, hard), Freshness::Fresh);
        assert_eq!(freshness(Duration::from_secs(9), soft, hard), Freshness::Fresh);
        assert_eq!(freshness(Duration::from_secs(10), soft, hard), Freshness::Stale);
        assert_eq!(freshness(Duration::from_secs(399), soft, hard), Freshness::Stale);
        assert_eq!(freshness(Duration::from_secs(400), soft, hard), Freshness::Expired);
    }

    #[test]
    fn parses_a_library_with_options() {
        let id = Uuid::new_v4();
        let option_id = Uuid::new_v4();
        let body = json!({
            "id": id.to_string(), "name": "Brands", "code": "brands", "selection_mode": "multi",
            "options": [
                { "id": option_id.to_string(), "name": "Apple", "code": "apple", "active": true, "Country": "US" },
                { "id": "not-a-uuid", "name": "Broken", "code": "x" },
                { "id": Uuid::new_v4().to_string(), "name": "Old", "code": "old", "active": false }
            ]
        });
        let library = parse_library(&body, Uuid::nil());
        assert_eq!(library.id, id);
        assert_eq!((library.name.as_str(), library.code.as_str(), library.selection_mode.as_str()), ("Brands", "brands", "multi"));
        assert_eq!(library.options.len(), 2, "an option without a valid id is skipped");
        assert_eq!(library.options[0].id, option_id);
        assert!(library.options[0].active);
        assert!(!library.options[1].active);
    }

    #[test]
    fn parses_a_library_without_options() {
        let requested = Uuid::new_v4();
        let library = parse_library(&json!({ "name": "Empty" }), requested);
        assert_eq!(library.id, requested);
        assert_eq!(library.selection_mode, "single");
        assert!(library.options.is_empty());
    }
}
