#[cfg(not(target_arch = "wasm32"))]
use std::{
    collections::{HashMap, VecDeque},
    env,
    sync::{Mutex, OnceLock},
};

#[cfg(not(target_arch = "wasm32"))]
use fastembed::{EmbeddingModel, TextEmbedding, TextInitOptions};

#[cfg(not(target_arch = "wasm32"))]
use super::cache::SmallModelCache;

/// Text embedding language selector.
///
/// This is intentionally small (English/Chinese) to match the project's current
/// needs. Use `TextEmbeddingModel` if you want explicit model selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextEmbeddingLanguage {
    English,
    Chinese,
}

impl TextEmbeddingLanguage {
    /// Pick a default model for each language.
    ///
    /// English defaults to BGESmallENV15; Chinese defaults to BGESmallZHV15.
    pub const fn default_model(self) -> TextEmbeddingModel {
        match self {
            TextEmbeddingLanguage::English => TextEmbeddingModel::BgeSmallEnV15,
            TextEmbeddingLanguage::Chinese => TextEmbeddingModel::BgeSmallZhV15,
        }
    }
}

/// Text embedding models backed by fastembed.
///
/// Variants map directly to `fastembed::EmbeddingModel` so we can switch models
/// without leaking fastembed types into other crates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextEmbeddingModel {
    BgeSmallEnV15,
    BgeBaseEnV15,
    BgeLargeEnV15,
    BgeSmallZhV15,
    BgeLargeZhV15,
    ClipVitB32,
}

impl TextEmbeddingModel {
    /// Embedding dimension for each model (from fastembed model list).
    pub const fn dim(self) -> usize {
        match self {
            TextEmbeddingModel::BgeSmallEnV15 => 384,
            TextEmbeddingModel::BgeBaseEnV15 => 768,
            TextEmbeddingModel::BgeLargeEnV15 => 1024,
            TextEmbeddingModel::BgeSmallZhV15 => 512,
            TextEmbeddingModel::BgeLargeZhV15 => 1024,
            TextEmbeddingModel::ClipVitB32 => 512,
        }
    }

    /// Map to the underlying fastembed enum.
    #[cfg(not(target_arch = "wasm32"))]
    fn to_fastembed(self) -> EmbeddingModel {
        match self {
            TextEmbeddingModel::BgeSmallEnV15 => EmbeddingModel::BGESmallENV15,
            TextEmbeddingModel::BgeBaseEnV15 => EmbeddingModel::BGEBaseENV15,
            TextEmbeddingModel::BgeLargeEnV15 => EmbeddingModel::BGELargeENV15,
            TextEmbeddingModel::BgeSmallZhV15 => EmbeddingModel::BGESmallZHV15,
            TextEmbeddingModel::BgeLargeZhV15 => EmbeddingModel::BGELargeZHV15,
            TextEmbeddingModel::ClipVitB32 => EmbeddingModel::ClipVitB32,
        }
    }
}

/// Default language/model used by `embed_text`.
pub const DEFAULT_TEXT_LANGUAGE: TextEmbeddingLanguage = TextEmbeddingLanguage::English;
pub const DEFAULT_TEXT_MODEL: TextEmbeddingModel = DEFAULT_TEXT_LANGUAGE.default_model();

/// Dimension for English text embeddings stored in LanceDB.
///
/// IMPORTANT: If you change the default English model, update your LanceDB
/// schema and rebuild the tables to match the new vector dimension.
pub const TEXT_VECTOR_DIM_EN: usize = TextEmbeddingLanguage::English.default_model().dim();

/// Dimension for Chinese text embeddings stored in LanceDB.
///
/// IMPORTANT: If you change the default Chinese model, update your LanceDB
/// schema and rebuild the tables to match the new vector dimension.
pub const TEXT_VECTOR_DIM_ZH: usize = TextEmbeddingLanguage::Chinese.default_model().dim();

#[cfg(not(target_arch = "wasm32"))]
static FASTEMBED_TEXT_MODEL: OnceLock<Mutex<SmallModelCache<TextEmbeddingModel, TextEmbedding>>> =
    OnceLock::new();
#[cfg(not(target_arch = "wasm32"))]
static FASTEMBED_TEXT_MODEL_CACHE_LIMIT: OnceLock<usize> = OnceLock::new();
#[cfg(not(target_arch = "wasm32"))]
static FASTEMBED_QUERY_VECTOR_CACHE: OnceLock<Mutex<QueryEmbeddingLruCache>> = OnceLock::new();
#[cfg(not(target_arch = "wasm32"))]
static FASTEMBED_QUERY_CACHE_LIMIT: OnceLock<usize> = OnceLock::new();
#[cfg(not(target_arch = "wasm32"))]
const DEFAULT_MAX_CACHED_TEXT_MODELS: usize = 3;
#[cfg(not(target_arch = "wasm32"))]
const DEFAULT_QUERY_CACHE_CAPACITY: usize = 128;
#[cfg(not(target_arch = "wasm32"))]
const MAX_CACHED_QUERY_TEXT_BYTES: usize = 2048;
#[cfg(not(target_arch = "wasm32"))]
const DEFAULT_TEXT_EMBED_BATCH_SIZE: usize = 64;

#[cfg(not(target_arch = "wasm32"))]
struct QueryEmbeddingLruCache {
    max_entries: usize,
    order: VecDeque<(TextEmbeddingModel, String)>,
    values: HashMap<(TextEmbeddingModel, String), Vec<f32>>,
}

#[cfg(not(target_arch = "wasm32"))]
impl QueryEmbeddingLruCache {
    fn new(max_entries: usize) -> Self {
        Self {
            max_entries: max_entries.max(1),
            order: VecDeque::new(),
            values: HashMap::new(),
        }
    }

    fn get(&mut self, model: TextEmbeddingModel, text: &str) -> Option<Vec<f32>> {
        let key = (model, text.to_string());
        let vector = self.values.get(&key)?.clone();
        self.touch(&key);
        Some(vector)
    }

    fn insert(&mut self, model: TextEmbeddingModel, text: String, vector: Vec<f32>) {
        let key = (model, text);
        if self.values.contains_key(&key) {
            self.values.insert(key.clone(), vector);
            self.touch(&key);
            return;
        }
        while self.values.len() >= self.max_entries {
            if let Some(oldest) = self.order.pop_front() {
                if self.values.remove(&oldest).is_some() {
                    break;
                }
            } else {
                break;
            }
        }
        self.order.push_back(key.clone());
        self.values.insert(key, vector);
    }

    fn touch(&mut self, key: &(TextEmbeddingModel, String)) {
        if let Some(pos) = self.order.iter().position(|existing| existing == key) {
            self.order.remove(pos);
        }
        self.order.push_back(key.clone());
    }
}

/// Generate a semantic embedding for text using the default language/model.
///
/// Use `embed_text_with_language` or `embed_text_with_model` if you need a
/// specific language or model.
pub fn embed_text(text: &str) -> anyhow::Result<Vec<f32>> {
    embed_text_with_language(text, DEFAULT_TEXT_LANGUAGE)
}

/// Generate a semantic embedding for text using a language-specific default
/// model.
pub fn embed_text_with_language(
    text: &str,
    language: TextEmbeddingLanguage,
) -> anyhow::Result<Vec<f32>> {
    embed_text_with_model(text, language.default_model())
}

/// Generate a semantic embedding for text using an explicit model selection.
pub fn embed_text_with_model(text: &str, model: TextEmbeddingModel) -> anyhow::Result<Vec<f32>> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        fastembed_embedding(text, model)
    }

    #[cfg(target_arch = "wasm32")]
    {
        let _ = text;
        let _ = model;
        anyhow::bail!("text embedding is not supported on wasm32")
    }
}

/// Generate semantic embeddings for a batch of texts using the default
/// language/model in a single batched ONNX forward pass.
pub fn embed_texts(texts: &[String]) -> anyhow::Result<Vec<Vec<f32>>> {
    embed_texts_with_language(texts, DEFAULT_TEXT_LANGUAGE)
}

/// Generate semantic embeddings for a batch of texts using a language-specific
/// default model in a single batched ONNX forward pass.
pub fn embed_texts_with_language(
    texts: &[String],
    language: TextEmbeddingLanguage,
) -> anyhow::Result<Vec<Vec<f32>>> {
    embed_texts_with_model(texts, language.default_model())
}

/// Generate semantic embeddings for a batch of texts using an explicit model
/// selection in a single batched ONNX forward pass.
pub fn embed_texts_with_model(
    texts: &[String],
    model: TextEmbeddingModel,
) -> anyhow::Result<Vec<Vec<f32>>> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        fastembed_embeddings_batch(texts, model)
    }

    #[cfg(target_arch = "wasm32")]
    {
        let _ = texts;
        let _ = model;
        anyhow::bail!("text embedding is not supported on wasm32")
    }
}

/// Pre-initialize and warm up the default Chinese and English text embedding
/// models (`BgeSmallZhV15` and `BgeSmallEnV15`) so the first user query does
/// not pay cold ONNX weight-loading latency.
pub fn warmup_default_text_models() -> anyhow::Result<()> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        for model in [TextEmbeddingModel::BgeSmallZhV15, TextEmbeddingModel::BgeSmallEnV15] {
            let _ = fastembed_embedding_uncached("StaticFlow warmup", model)?;
        }
        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    {
        anyhow::bail!("text embedding warmup is not supported on wasm32")
    }
}

/// Detect language with a lightweight heuristic.
///
/// If the input contains any CJK character, we treat it as Chinese; otherwise
/// default to English. This avoids external dependencies and keeps decisions
/// local and deterministic.
pub fn detect_language(text: &str) -> TextEmbeddingLanguage {
    if text.chars().any(is_cjk) {
        TextEmbeddingLanguage::Chinese
    } else {
        TextEmbeddingLanguage::English
    }
}

fn is_cjk(ch: char) -> bool {
    matches!(
        ch as u32,
        0x4E00..=0x9FFF
            | 0x3400..=0x4DBF
            | 0x20000..=0x2A6DF
            | 0x2A700..=0x2B73F
            | 0x2B740..=0x2B81F
            | 0x2B820..=0x2CEAF
            | 0xF900..=0xFAFF
            | 0x2F800..=0x2FA1F
    )
}

#[cfg(not(target_arch = "wasm32"))]
fn fastembed_embedding(text: &str, model: TextEmbeddingModel) -> anyhow::Result<Vec<f32>> {
    let cacheable = text.len() <= MAX_CACHED_QUERY_TEXT_BYTES;
    if cacheable {
        let cache_lock = FASTEMBED_QUERY_VECTOR_CACHE
            .get_or_init(|| Mutex::new(QueryEmbeddingLruCache::new(query_cache_limit())));
        if let Ok(mut cache) = cache_lock.lock() {
            if let Some(cached) = cache.get(model, text) {
                return Ok(cached);
            }
        }
    }

    let vector = fastembed_embedding_uncached(text, model)?;

    if cacheable {
        let cache_lock = FASTEMBED_QUERY_VECTOR_CACHE
            .get_or_init(|| Mutex::new(QueryEmbeddingLruCache::new(query_cache_limit())));
        if let Ok(mut cache) = cache_lock.lock() {
            cache.insert(model, text.to_string(), vector.clone());
        }
    }

    Ok(vector)
}

#[cfg(not(target_arch = "wasm32"))]
fn fastembed_embedding_uncached(text: &str, model: TextEmbeddingModel) -> anyhow::Result<Vec<f32>> {
    let lock = FASTEMBED_TEXT_MODEL
        .get_or_init(|| Mutex::new(SmallModelCache::new(text_model_cache_limit())));
    let mut guard = lock
        .lock()
        .map_err(|err| anyhow::anyhow!("text embedding mutex poisoned: {err}"))?;

    let instance = guard.get_or_try_insert_mut(model, || {
        // Model initialization is expensive; cache a small LRU set for reuse.
        let options = TextInitOptions::new(model.to_fastembed());
        TextEmbedding::try_new(options).map_err(|err| {
            anyhow::anyhow!("failed to initialize text embedding model {:?}: {err}", model)
        })
    })?;
    let mut embeddings = instance
        .embed(vec![text], None)
        .map_err(|err| anyhow::anyhow!("text embedding failed for model {:?}: {err}", model))?;
    embeddings.pop().ok_or_else(|| {
        anyhow::anyhow!("text embedding model {:?} returned empty embedding result", model)
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn fastembed_embeddings_batch(
    texts: &[String],
    model: TextEmbeddingModel,
) -> anyhow::Result<Vec<Vec<f32>>> {
    if texts.is_empty() {
        return Ok(Vec::new());
    }

    let lock = FASTEMBED_TEXT_MODEL
        .get_or_init(|| Mutex::new(SmallModelCache::new(text_model_cache_limit())));
    let mut guard = lock
        .lock()
        .map_err(|err| anyhow::anyhow!("text embedding mutex poisoned: {err}"))?;

    let instance = guard.get_or_try_insert_mut(model, || {
        let options = TextInitOptions::new(model.to_fastembed());
        TextEmbedding::try_new(options).map_err(|err| {
            anyhow::anyhow!("failed to initialize text embedding model {:?}: {err}", model)
        })
    })?;
    let embeddings = instance
        .embed(texts, Some(DEFAULT_TEXT_EMBED_BATCH_SIZE))
        .map_err(|err| {
            anyhow::anyhow!(
                "batched text embedding failed for model {:?} (batch_len={}): {err}",
                model,
                texts.len()
            )
        })?;
    if embeddings.len() != texts.len() {
        anyhow::bail!(
            "text embedding model {:?} returned {} vectors for {} inputs",
            model,
            embeddings.len(),
            texts.len()
        );
    }
    Ok(embeddings)
}

#[cfg(not(target_arch = "wasm32"))]
fn text_model_cache_limit() -> usize {
    *FASTEMBED_TEXT_MODEL_CACHE_LIMIT.get_or_init(|| {
        env::var("FASTEMBED_MAX_CACHED_TEXT_MODELS")
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .filter(|value| *value > 0)
            .unwrap_or(DEFAULT_MAX_CACHED_TEXT_MODELS)
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn query_cache_limit() -> usize {
    *FASTEMBED_QUERY_CACHE_LIMIT.get_or_init(|| {
        env::var("FASTEMBED_QUERY_CACHE_SIZE")
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .filter(|value| *value > 0)
            .unwrap_or(DEFAULT_QUERY_CACHE_CAPACITY)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embed_text_has_expected_shape() {
        let vector = embed_text("StaticFlow embeddings").expect("embed text");
        assert_eq!(vector.len(), TEXT_VECTOR_DIM_EN);
        assert!(vector.iter().any(|v| *v != 0.0));
        assert!(vector.iter().all(|v| v.is_finite()));
    }

    #[test]
    fn embed_text_with_language_matches_language_dim() {
        let vector = embed_text_with_language("中文内容", TextEmbeddingLanguage::Chinese)
            .expect("embed text");
        let expected_dim = TextEmbeddingLanguage::Chinese.default_model().dim();
        assert_eq!(vector.len(), expected_dim);
        assert!(vector.iter().any(|v| *v != 0.0));
        assert!(vector.iter().all(|v| v.is_finite()));
    }

    #[test]
    fn embed_texts_empty_slice_returns_empty_without_error() {
        let vectors =
            embed_texts_with_language(&[], TextEmbeddingLanguage::English).expect("empty batch");
        assert!(vectors.is_empty());
    }

    #[test]
    fn embed_texts_batch_matches_single_shape() {
        let inputs = vec!["first snippet".to_string(), "second snippet".to_string()];
        let vectors = embed_texts_with_language(&inputs, TextEmbeddingLanguage::English)
            .expect("batched embed");
        assert_eq!(vectors.len(), 2);
        for vec in &vectors {
            assert_eq!(vec.len(), TEXT_VECTOR_DIM_EN);
            assert!(vec.iter().all(|v| v.is_finite()));
            assert!(vec.iter().any(|v| *v != 0.0));
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn query_embedding_lru_cache_hits_and_evicts_oldest() {
        let mut cache = QueryEmbeddingLruCache::new(2);
        cache.insert(TextEmbeddingModel::BgeSmallEnV15, "q1".to_string(), vec![1.0, 2.0]);
        cache.insert(TextEmbeddingModel::BgeSmallEnV15, "q2".to_string(), vec![3.0, 4.0]);

        // Touch q1 so q2 becomes least recently used.
        assert_eq!(cache.get(TextEmbeddingModel::BgeSmallEnV15, "q1"), Some(vec![1.0, 2.0]));
        assert_eq!(cache.get(TextEmbeddingModel::BgeSmallZhV15, "q1"), None);

        cache.insert(TextEmbeddingModel::BgeSmallEnV15, "q3".to_string(), vec![5.0, 6.0]);
        assert_eq!(cache.get(TextEmbeddingModel::BgeSmallEnV15, "q2"), None);
        assert_eq!(cache.get(TextEmbeddingModel::BgeSmallEnV15, "q1"), Some(vec![1.0, 2.0]));
        assert_eq!(cache.get(TextEmbeddingModel::BgeSmallEnV15, "q3"), Some(vec![5.0, 6.0]));
    }

    #[test]
    fn fastembed_smoke_if_available() {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Ok(vector) = fastembed_embedding("fastembed smoke", DEFAULT_TEXT_MODEL) {
                assert_eq!(vector.len(), TEXT_VECTOR_DIM_EN);
                assert!(vector.iter().all(|v| v.is_finite()));
                assert!(vector.iter().any(|v| *v != 0.0));
            }
        }
    }

    #[test]
    fn detect_language_defaults_to_english() {
        let language = detect_language("Hello from StaticFlow");
        assert_eq!(language, TextEmbeddingLanguage::English);
    }

    #[test]
    fn detect_language_detects_chinese_characters() {
        let language = detect_language("你好，StaticFlow");
        assert_eq!(language, TextEmbeddingLanguage::Chinese);
    }
}
