use crate::model::WordCloudSession;
use dioxus::prelude::*;
#[cfg(feature = "server")]
use std::collections::HashMap;

#[cfg(feature = "server")]
use std::fs;
#[cfg(feature = "server")]
use std::sync::{LazyLock, RwLock};

#[cfg(feature = "server")]
const DATA_DIR: &str = "./data";
#[cfg(feature = "server")]
const DATA_FILE: &str = "./data/wordclouds.json";
#[cfg(feature = "server")]
const MAX_SESSIONS: usize = 2000;

#[cfg(feature = "server")]
#[derive(Default, serde::Serialize, serde::Deserialize)]
struct StorageData {
    sessions: HashMap<String, WordCloudSession>,
}

#[cfg(feature = "server")]
struct LocalStore {
    sessions: HashMap<String, WordCloudSession>,
}

#[cfg(feature = "server")]
impl LocalStore {
    fn new() -> Self {
        let mut store = Self {
            sessions: HashMap::new(),
        };
        store.load();
        store
    }

    fn load(&mut self) {
        if let Ok(content) = fs::read_to_string(DATA_FILE) {
            if let Ok(data) = serde_json::from_str::<StorageData>(&content) {
                self.sessions = data.sessions;
            }
        }
    }

    fn save(&self) {
        let _ = fs::create_dir_all(DATA_DIR);
        let data = StorageData {
            sessions: self.sessions.clone(),
        };
        if let Ok(json) = serde_json::to_string_pretty(&data) {
            let _ = fs::write(DATA_FILE, json);
        }
    }

    fn evict_oldest_if_needed(&mut self) {
        while self.sessions.len() >= MAX_SESSIONS {
            // Find session with oldest created_at
            if let Some((oldest_code, _)) = self
                .sessions
                .iter()
                .min_by_key(|(_, sess)| sess.created_at)
                .map(|(k, v)| (k.clone(), v.clone()))
            {
                self.sessions.remove(&oldest_code);
            } else {
                break;
            }
        }
    }
}

#[cfg(feature = "server")]
static STORE: LazyLock<RwLock<LocalStore>> = LazyLock::new(|| RwLock::new(LocalStore::new()));

#[cfg(feature = "server")]
fn get_current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Get existing cloud or create a new one with the given code (1-8 digits)
#[post("/api/get_or_create_cloud")]
pub async fn get_or_create_cloud(code: String) -> Result<WordCloudSession, ServerFnError> {
    #[cfg(feature = "server")]
    {
        let clean_code = code.trim().to_string();
        if clean_code.is_empty() || clean_code.len() > 8 {
            return Err(ServerFnError::new("Il codice deve essere compreso tra 1 e 8 cifre"));
        }

        let mut store = STORE.write().map_err(|e| ServerFnError::new(e.to_string()))?;
        if let Some(session) = store.sessions.get(&clean_code) {
            return Ok(session.clone());
        }

        store.evict_oldest_if_needed();
        let now = get_current_timestamp();
        let session = WordCloudSession::new(clean_code.clone(), now);
        store.sessions.insert(clean_code, session.clone());
        store.save();

        Ok(session)
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = code;
        Err(ServerFnError::new("Disponibile solo su server"))
    }
}

/// Generate a new unique word cloud ID (1 to 8 digits) and create it.
/// If ID pool is full, evicts the oldest cloud.
#[post("/api/create_new_cloud")]
pub async fn create_new_cloud() -> Result<String, ServerFnError> {
    #[cfg(feature = "server")]
    {
        use rand::RngExt;
        let mut store = STORE.write().map_err(|e| ServerFnError::new(e.to_string()))?;
        store.evict_oldest_if_needed();

        let mut rng = rand::rng();
        // Generate random numeric code between 4 and 6 digits by default for convenience,
        // or up to 8 digits if occupied
        let mut chosen_code = String::new();
        for _ in 0..100 {
            let candidate: u32 = rng.random_range(1000..999999);
            let code_str = candidate.to_string();
            if !store.sessions.contains_key(&code_str) {
                chosen_code = code_str;
                break;
            }
        }

        // Fallback: 7-8 digits if collisions occurred
        if chosen_code.is_empty() {
            for _ in 0..100 {
                let candidate: u32 = rng.random_range(1_000_000..99_999_999);
                let code_str = candidate.to_string();
                if !store.sessions.contains_key(&code_str) {
                    chosen_code = code_str;
                    break;
                }
            }
        }

        // Extreme fallback: evict oldest explicitly
        if chosen_code.is_empty() {
            if let Some((oldest_code, _)) = store
                .sessions
                .iter()
                .min_by_key(|(_, sess)| sess.created_at)
                .map(|(k, v)| (k.clone(), v.clone()))
            {
                store.sessions.remove(&oldest_code);
                chosen_code = oldest_code;
            } else {
                chosen_code = "1001".to_string();
            }
        }

        let now = get_current_timestamp();
        let session = WordCloudSession::new(chosen_code.clone(), now);
        store.sessions.insert(chosen_code.clone(), session);
        store.save();

        Ok(chosen_code)
    }
    #[cfg(not(feature = "server"))]
    {
        Err(ServerFnError::new("Disponibile solo su server"))
    }
}

/// Fetch cloud session data for live polling
#[post("/api/get_cloud")]
pub async fn get_cloud(code: String) -> Result<Option<WordCloudSession>, ServerFnError> {
    #[cfg(feature = "server")]
    {
        let clean_code = code.trim();
        let store = STORE.read().map_err(|e| ServerFnError::new(e.to_string()))?;
        Ok(store.sessions.get(clean_code).cloned())
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = code;
        Err(ServerFnError::new("Disponibile solo su server"))
    }
}

/// Submit a word from a participant into the cloud
#[post("/api/submit_word")]
pub async fn submit_word(code: String, word: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        let clean_code = code.trim().to_string();
        let trimmed_word = word.trim();
        if trimmed_word.is_empty() {
            return Err(ServerFnError::new("La parola non può essere vuota"));
        }
        if trimmed_word.chars().count() > 35 {
            return Err(ServerFnError::new("La parola deve contenere massimo 35 caratteri"));
        }

        // Normalize: lowercase or capitalize
        let normalized = trimmed_word.to_lowercase();

        let mut store = STORE.write().map_err(|e| ServerFnError::new(e.to_string()))?;
        let now = get_current_timestamp();

        let session = store
            .sessions
            .entry(clean_code.clone())
            .or_insert_with(|| WordCloudSession::new(clean_code, now));

        let entry = session.words.entry(normalized).or_insert(0);
        *entry += 1;
        session.updated_at = now;

        store.save();
        Ok(())
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = (code, word);
        Err(ServerFnError::new("Disponibile solo su server"))
    }
}

/// Reset all words for a given word cloud session
#[post("/api/reset_cloud")]
pub async fn reset_cloud(code: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        let clean_code = code.trim();
        let mut store = STORE.write().map_err(|e| ServerFnError::new(e.to_string()))?;
        if let Some(session) = store.sessions.get_mut(clean_code) {
            session.words.clear();
            session.updated_at = get_current_timestamp();
            store.save();
        }
        Ok(())
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = code;
        Err(ServerFnError::new("Disponibile solo su server"))
    }
}

/// Remove a single word from the cloud
#[post("/api/delete_word")]
pub async fn delete_word(code: String, word: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        let clean_code = code.trim();
        let normalized = word.trim().to_lowercase();
        let mut store = STORE.write().map_err(|e| ServerFnError::new(e.to_string()))?;
        if let Some(session) = store.sessions.get_mut(clean_code) {
            session.words.remove(&normalized);
            session.words.remove(word.trim());
            session.updated_at = get_current_timestamp();
            store.save();
        }
        Ok(())
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = (code, word);
        Err(ServerFnError::new("Disponibile solo su server"))
    }
}

#[cfg(test)]
#[cfg(feature = "server")]
mod server_tests {
    use super::*;

    #[test]
    fn test_store_eviction() {
        let mut store = LocalStore { sessions: HashMap::new() };
        for i in 1..=5 {
            let s = WordCloudSession::new(format!("code{}", i), i as u64);
            store.sessions.insert(format!("code{}", i), s);
        }
        assert_eq!(store.sessions.len(), 5);

        // Simulate eviction down to 3
        while store.sessions.len() > 3 {
            let oldest = store.sessions.iter().min_by_key(|(_, sess)| sess.created_at).map(|(k, _)| k.clone()).unwrap();
            store.sessions.remove(&oldest);
        }

        assert_eq!(store.sessions.len(), 3);
        assert!(!store.sessions.contains_key("code1"));
        assert!(!store.sessions.contains_key("code2"));
        assert!(store.sessions.contains_key("code3"));
        assert!(store.sessions.contains_key("code4"));
        assert!(store.sessions.contains_key("code5"));
    }
}
