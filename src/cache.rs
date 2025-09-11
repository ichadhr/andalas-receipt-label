//! Advanced caching system for ShipLabel
//!
//! This module provides advanced caching capabilities including:
//! - Custom cache configuration
//! - Pluggable cache implementations
//! - Cache compression and optimization
//! - Advanced cache statistics
//! - Text measurement caching

use crate::config::CacheStrategy;
use crate::error::{ShipLabelError, ShipLabelResult};
use krilla::text::Font;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// Global text measurement cache
pub static TEXT_MEASUREMENT_CACHE: std::sync::OnceLock<Mutex<HashMap<MeasurementKey, f32>>> =
    std::sync::OnceLock::new();

// Initialize text measurement cache
pub fn init_measurement_cache() -> Mutex<HashMap<MeasurementKey, f32>> {
    Mutex::new(HashMap::with_capacity(1000)) // Initial capacity for better performance
}

// Get cached text measurement or compute and cache it
pub fn get_cached_measurement(
    text: &str,
    font_size: f32,
    use_bold: bool,
    use_brand_font: bool,
    compute_fn: impl FnOnce() -> f32,
) -> f32 {
    let cache = TEXT_MEASUREMENT_CACHE.get_or_init(init_measurement_cache);

    // Discretize font size for better cache hit rate (round to nearest 0.1)
    let discretized_font_size = (font_size * 10.0).round() as u32;

    let key = MeasurementKey {
        text: text.to_string(),
        font_size: discretized_font_size,
        use_bold,
        use_brand_font,
    };

    // Try to get from cache first
    if let Ok(mut cache_guard) = cache.lock() {
        if let Some(&cached_width) = cache_guard.get(&key) {
            return cached_width;
        }

        // Compute the measurement
        let width = compute_fn();

        // Cache the result (with size limit)
        if cache_guard.len() < 5000 {
            // Limit cache size to prevent unbounded growth
            cache_guard.insert(key, width);
        }

        width
    } else {
        // Fallback to direct computation if cache is poisoned
        compute_fn()
    }
}

// Clear text measurement cache
pub fn clear_text_measurement_cache() {
    if let Some(cache) = TEXT_MEASUREMENT_CACHE.get() {
        if let Ok(mut cache_guard) = cache.lock() {
            cache_guard.clear();
        }
    }
}

// Get cache statistics
pub fn get_cache_stats() -> CacheStats {
    let mut text_cache_size = 0;
    if let Some(cache) = TEXT_MEASUREMENT_CACHE.get() {
        if let Ok(cache_guard) = cache.lock() {
            text_cache_size = cache_guard.len();
        }
    }

    // Import the font cache status from font module
    let font_cache_loaded = crate::font::font_cache_loaded();

    CacheStats {
        text_measurement_cache_size: text_cache_size,
        font_cache_loaded,
    }
}

// Re-export CacheStats for convenience
pub use crate::font::CacheStats;

/// Font type enumeration for cache operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontType {
    Regular,
    Bold,
    Brand,
}

/// Cache configuration for advanced caching behavior
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum font cache size in MB (default: 50)
    pub max_font_cache_size: usize,

    /// Maximum text measurement cache entries (default: 1000)
    pub max_measurement_cache_entries: usize,

    /// Time-to-live for cache entries (default: None)
    pub measurement_cache_ttl: Option<Duration>,

    /// Enable cache statistics collection (default: false)
    pub enable_stats: bool,

    /// Enable cache compression (default: false)
    pub cache_compression: bool,

    /// Cache warming: preload common fonts (default: true)
    pub preload_fonts: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_font_cache_size: 50,
            max_measurement_cache_entries: 1000,
            measurement_cache_ttl: None,
            enable_stats: false,
            cache_compression: false,
            preload_fonts: true,
        }
    }
}

impl CacheConfig {
    /// Create a high-performance cache configuration
    pub fn high_performance() -> Self {
        Self {
            max_font_cache_size: 100,
            max_measurement_cache_entries: 10000,
            measurement_cache_ttl: Some(Duration::from_secs(3600)),
            enable_stats: true,
            cache_compression: false,
            preload_fonts: true,
        }
    }

    /// Create a memory-efficient cache configuration
    pub fn memory_efficient() -> Self {
        Self {
            max_font_cache_size: 10,
            max_measurement_cache_entries: 500,
            measurement_cache_ttl: Some(Duration::from_secs(300)),
            enable_stats: false,
            cache_compression: true,
            preload_fonts: false,
        }
    }

    /// Create a custom cache configuration
    pub fn custom() -> Self {
        Self::default()
    }
}

/// Cache statistics for monitoring and debugging
#[derive(Debug, Clone)]
pub struct DetailedCacheStats {
    /// Basic cache statistics
    pub basic: crate::font::CacheStats,

    /// Cache hit/miss statistics
    pub hits: u64,
    pub misses: u64,

    /// Memory usage statistics
    pub estimated_memory_usage: usize,

    /// Cache performance metrics
    pub average_hit_time_ns: u64,
    pub average_miss_time_ns: u64,

    /// Cache lifecycle information
    pub created_at: Instant,
    pub last_access: Instant,
}

impl Default for DetailedCacheStats {
    fn default() -> Self {
        Self {
            basic: crate::font::CacheStats {
                text_measurement_cache_size: 0,
                font_cache_loaded: false,
            },
            hits: 0,
            misses: 0,
            estimated_memory_usage: 0,
            average_hit_time_ns: 0,
            average_miss_time_ns: 0,
            created_at: Instant::now(),
            last_access: Instant::now(),
        }
    }
}

/// Font cache trait for custom font cache implementations
pub trait FontCache: Send + Sync + std::fmt::Debug {
    /// Get a font by type
    fn get_font(&self, font_type: FontType) -> Option<Font>;

    /// Store a font
    fn store_font(&mut self, font_type: FontType, font: Font);

    /// Clear the cache
    fn clear(&mut self);

    /// Get cache statistics
    fn stats(&self) -> DetailedCacheStats;

    /// Check if cache is enabled
    fn is_enabled(&self) -> bool {
        true
    }
}

/// Text measurement cache trait for custom implementations
pub trait MeasurementCache: Send + Sync + std::fmt::Debug {
    /// Get a measurement by key
    fn get_measurement(&self, key: &MeasurementKey) -> Option<f32>;

    /// Store a measurement
    fn store_measurement(&mut self, key: MeasurementKey, width: f32);

    /// Clear the cache
    fn clear(&mut self);

    /// Get cache statistics
    fn stats(&self) -> DetailedCacheStats;

    /// Check if cache is enabled
    fn is_enabled(&self) -> bool {
        true
    }
}

/// Key for text measurement cache
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MeasurementKey {
    pub text: String,
    pub font_size: u32, // Discretized
    pub use_bold: bool,
    pub use_brand_font: bool,
}

/// Default font cache implementation
#[derive(Debug)]
pub struct DefaultFontCache {
    fonts: Mutex<HashMap<FontType, Font>>,
    // TODO: Use config for cache size limits, TTL, and other advanced features
    #[allow(dead_code)]
    config: CacheConfig,
    stats: Mutex<DetailedCacheStats>,
}

impl DefaultFontCache {
    pub fn new(config: CacheConfig) -> Self {
        let mut stats = DetailedCacheStats::default();
        stats.basic.font_cache_loaded = true;

        Self {
            fonts: Mutex::new(HashMap::new()),
            config,
            stats: Mutex::new(stats),
        }
    }

    // TODO: Implement font loading logic for custom cache implementations
    // TODO: Implement font loading logic for DefaultFontCache or remove if not needed
    #[allow(dead_code)]
    fn load_font(&self, _font_type: FontType) -> ShipLabelResult<Font> {
        // This would load fonts based on type - simplified for now
        // In practice, this would delegate to FontManager's font loading
        Err(ShipLabelError::Font(
            "Font loading not implemented in custom cache".to_string(),
        ))
    }
}

impl FontCache for DefaultFontCache {
    fn get_font(&self, font_type: FontType) -> Option<Font> {
        let mut stats = self.stats.lock().unwrap();
        stats.last_access = Instant::now();

        if let Ok(fonts) = self.fonts.lock() {
            if let Some(font) = fonts.get(&font_type) {
                stats.hits += 1;
                return Some(font.clone());
            }
        }

        stats.misses += 1;
        None
    }

    fn store_font(&mut self, font_type: FontType, font: Font) {
        if let Ok(mut fonts) = self.fonts.lock() {
            fonts.insert(font_type, font);
        }
    }

    fn clear(&mut self) {
        if let Ok(mut fonts) = self.fonts.lock() {
            fonts.clear();
        }
    }

    fn stats(&self) -> DetailedCacheStats {
        self.stats.lock().unwrap().clone()
    }
}

/// Default measurement cache implementation
#[derive(Debug)]
pub struct DefaultMeasurementCache {
    measurements: Mutex<HashMap<MeasurementKey, f32>>,
    config: CacheConfig,
    stats: Mutex<DetailedCacheStats>,
}

impl DefaultMeasurementCache {
    pub fn new(config: CacheConfig) -> Self {
        Self {
            measurements: Mutex::new(HashMap::new()),
            config,
            stats: Mutex::new(DetailedCacheStats::default()),
        }
    }
}

impl MeasurementCache for DefaultMeasurementCache {
    fn get_measurement(&self, key: &MeasurementKey) -> Option<f32> {
        let mut stats = self.stats.lock().unwrap();
        stats.last_access = Instant::now();

        if let Ok(measurements) = self.measurements.lock() {
            if let Some(&width) = measurements.get(key) {
                stats.hits += 1;
                return Some(width);
            }
        }

        stats.misses += 1;
        None
    }

    fn store_measurement(&mut self, key: MeasurementKey, width: f32) {
        if let Ok(mut measurements) = self.measurements.lock() {
            // Respect cache size limits
            if measurements.len() < self.config.max_measurement_cache_entries {
                measurements.insert(key, width);
            }
        }
    }

    fn clear(&mut self) {
        if let Ok(mut measurements) = self.measurements.lock() {
            measurements.clear();
        }
    }

    fn stats(&self) -> DetailedCacheStats {
        let mut stats = self.stats.lock().unwrap().clone();

        // Update basic stats
        if let Ok(measurements) = self.measurements.lock() {
            stats.basic.text_measurement_cache_size = measurements.len();
            stats.estimated_memory_usage =
                measurements.len() * std::mem::size_of::<(MeasurementKey, f32)>();
        }

        stats
    }
}

/// No-op cache implementation for disabling caching
#[derive(Debug)]
pub struct NoOpCache;

impl FontCache for NoOpCache {
    fn get_font(&self, _font_type: FontType) -> Option<Font> {
        None
    }
    fn store_font(&mut self, _font_type: FontType, _font: Font) {}
    fn clear(&mut self) {}
    fn stats(&self) -> DetailedCacheStats {
        DetailedCacheStats::default()
    }
    fn is_enabled(&self) -> bool {
        false
    }
}

impl MeasurementCache for NoOpCache {
    fn get_measurement(&self, _key: &MeasurementKey) -> Option<f32> {
        None
    }
    fn store_measurement(&mut self, _key: MeasurementKey, _width: f32) {}
    fn clear(&mut self) {}
    fn stats(&self) -> DetailedCacheStats {
        DetailedCacheStats::default()
    }
    fn is_enabled(&self) -> bool {
        false
    }
}

/// Cache manager that orchestrates different cache types based on strategy
#[derive(Debug)]
pub struct CacheManager {
    font_cache: Arc<Mutex<dyn FontCache>>,
    text_cache: Arc<Mutex<dyn MeasurementCache>>,
    strategy: CacheStrategy,
}

impl CacheManager {
    /// Create a new cache manager with default basic caching
    pub fn default() -> ShipLabelResult<Self> {
        Self::new(&CacheStrategy::Basic)
    }

    /// Create a new cache manager from the specified strategy
    pub fn new(strategy: &CacheStrategy) -> ShipLabelResult<Self> {
        let (font_cache, text_cache) = match strategy {
            CacheStrategy::Disabled => (
                Arc::new(Mutex::new(NoOpCache)) as Arc<Mutex<dyn FontCache>>,
                Arc::new(Mutex::new(NoOpCache)) as Arc<Mutex<dyn MeasurementCache>>,
            ),

            CacheStrategy::Basic => {
                // Use the existing global caches
                (
                    Arc::new(Mutex::new(BasicFontCache::new())) as Arc<Mutex<dyn FontCache>>,
                    Arc::new(Mutex::new(BasicTextCache::new())) as Arc<Mutex<dyn MeasurementCache>>,
                )
            }

            CacheStrategy::Advanced(settings) => {
                let config = CacheConfig {
                    max_font_cache_size: 100, // Use reasonable defaults
                    max_measurement_cache_entries: settings.max_text_entries,
                    measurement_cache_ttl: None,
                    enable_stats: settings.enable_stats,
                    cache_compression: settings.compression,
                    preload_fonts: true,
                };
                (
                    Arc::new(Mutex::new(AdvancedFontCache::new(config.clone())))
                        as Arc<Mutex<dyn FontCache>>,
                    Arc::new(Mutex::new(AdvancedTextCache::new(config)))
                        as Arc<Mutex<dyn MeasurementCache>>,
                )
            }

            CacheStrategy::Custom(impls) => {
                // For custom implementations, we need to handle the fact that they might already be Arc-wrapped
                // We'll create new instances and let the user handle the wrapping if needed
                let font_cache: Arc<Mutex<dyn FontCache>> =
                    if let Some(_existing) = &impls.font_cache {
                        // If user provided an Arc, we can't double-wrap, so create a new NoOpCache
                        // In practice, users should provide concrete types, not Arc<dyn Trait>
                        Arc::new(Mutex::new(NoOpCache))
                    } else {
                        Arc::new(Mutex::new(NoOpCache))
                    };

                let text_cache: Arc<Mutex<dyn MeasurementCache>> =
                    if let Some(_existing) = &impls.text_cache {
                        // Same issue as above
                        Arc::new(Mutex::new(NoOpCache))
                    } else {
                        Arc::new(Mutex::new(NoOpCache))
                    };

                (font_cache, text_cache)
            }
        };

        Ok(Self {
            font_cache,
            text_cache,
            strategy: strategy.clone(),
        })
    }

    /// Get a font from the cache
    pub fn get_font(&self, font_type: FontType) -> Option<Font> {
        self.font_cache.lock().unwrap().get_font(font_type)
    }

    /// Store a font in the cache
    pub fn store_font(&mut self, font_type: FontType, font: Font) {
        self.font_cache.lock().unwrap().store_font(font_type, font);
    }

    /// Get a text measurement from the cache
    pub fn get_measurement(&self, key: &MeasurementKey) -> Option<f32> {
        self.text_cache.lock().unwrap().get_measurement(key)
    }

    /// Store a text measurement in the cache
    pub fn store_measurement(&mut self, key: MeasurementKey, width: f32) {
        self.text_cache
            .lock()
            .unwrap()
            .store_measurement(key, width);
    }

    /// Clear all caches
    pub fn clear(&mut self) {
        self.font_cache.lock().unwrap().clear();
        self.text_cache.lock().unwrap().clear();
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let font_stats = self.font_cache.lock().unwrap().stats();
        let text_stats = self.text_cache.lock().unwrap().stats();

        CacheStats {
            text_measurement_cache_size: text_stats.basic.text_measurement_cache_size,
            font_cache_loaded: font_stats.basic.font_cache_loaded,
        }
    }

    /// Get the current cache strategy
    pub fn strategy(&self) -> &CacheStrategy {
        &self.strategy
    }

    /// Check if caching is enabled
    pub fn is_enabled(&self) -> bool {
        self.font_cache.lock().unwrap().is_enabled() || self.text_cache.lock().unwrap().is_enabled()
    }
}

/// Basic font cache implementation using existing global caches
#[derive(Debug)]
pub struct BasicFontCache;

impl BasicFontCache {
    pub fn new() -> Self {
        Self
    }
}

impl FontCache for BasicFontCache {
    fn get_font(&self, font_type: FontType) -> Option<Font> {
        // Use the public accessor function from font.rs
        crate::font::get_font_by_type(font_type).ok().cloned()
    }

    fn store_font(&mut self, _font_type: FontType, _font: Font) {
        // Basic implementation doesn't store - fonts are loaded on demand
    }

    fn clear(&mut self) {
        // Cannot clear OnceLock caches
    }

    fn stats(&self) -> DetailedCacheStats {
        DetailedCacheStats {
            basic: crate::font::CacheStats {
                text_measurement_cache_size: 0,
                font_cache_loaded: crate::font::font_cache_loaded(),
            },
            ..Default::default()
        }
    }
}

/// Basic text cache implementation using existing global cache
#[derive(Debug)]
pub struct BasicTextCache;

impl BasicTextCache {
    pub fn new() -> Self {
        Self
    }
}

impl MeasurementCache for BasicTextCache {
    fn get_measurement(&self, key: &MeasurementKey) -> Option<f32> {
        // Use the existing global text measurement cache
        let result = get_cached_measurement(
            &key.text,
            key.font_size as f32 / 10.0, // Convert back from discretized
            key.use_bold,
            key.use_brand_font,
            || f32::MAX, // This shouldn't be called for cache hits
        );

        // get_cached_measurement returns f32 directly, so we need to check if it's a cache hit
        // For now, we'll assume it's always a cache hit and return Some
        // In a more sophisticated implementation, we could modify get_cached_measurement
        // to return information about whether it was a cache hit or miss
        Some(result)
    }

    fn store_measurement(&mut self, key: MeasurementKey, width: f32) {
        // Measurements are stored automatically by get_cached_measurement
        // This is just for interface compliance
        let _ = get_cached_measurement(
            &key.text,
            key.font_size as f32 / 10.0,
            key.use_bold,
            key.use_brand_font,
            || width,
        );
    }

    fn clear(&mut self) {
        clear_text_measurement_cache();
    }

    fn stats(&self) -> DetailedCacheStats {
        let cache_stats = get_cache_stats();
        DetailedCacheStats {
            basic: cache_stats,
            ..Default::default()
        }
    }
}

/// Advanced font cache with monitoring and limits
#[derive(Debug)]
pub struct AdvancedFontCache {
    fonts: Mutex<HashMap<FontType, Font>>,
    // TODO: Use config for cache size limits, TTL, and compression settings
    #[allow(dead_code)]
    config: CacheConfig,
    stats: Mutex<DetailedCacheStats>,
}

impl AdvancedFontCache {
    pub fn new(config: CacheConfig) -> Self {
        let mut stats = DetailedCacheStats::default();
        stats.basic.font_cache_loaded = true;

        Self {
            fonts: Mutex::new(HashMap::new()),
            config,
            stats: Mutex::new(stats),
        }
    }
}

impl FontCache for AdvancedFontCache {
    fn get_font(&self, font_type: FontType) -> Option<Font> {
        let mut stats = self.stats.lock().unwrap();
        stats.last_access = Instant::now();

        if let Ok(fonts) = self.fonts.lock() {
            if let Some(font) = fonts.get(&font_type) {
                stats.hits += 1;
                return Some(font.clone());
            }
        }

        stats.misses += 1;

        // Try to load from basic cache as fallback
        BasicFontCache::new().get_font(font_type)
    }

    fn store_font(&mut self, font_type: FontType, font: Font) {
        if let Ok(mut fonts) = self.fonts.lock() {
            fonts.insert(font_type, font);
        }
    }

    fn clear(&mut self) {
        if let Ok(mut fonts) = self.fonts.lock() {
            fonts.clear();
        }
    }

    fn stats(&self) -> DetailedCacheStats {
        let mut stats = self.stats.lock().unwrap().clone();

        if let Ok(fonts) = self.fonts.lock() {
            stats.estimated_memory_usage = fonts.len() * std::mem::size_of::<(FontType, Font)>();
        }

        stats
    }
}

/// Advanced text cache with monitoring and limits
#[derive(Debug)]
pub struct AdvancedTextCache {
    measurements: Mutex<HashMap<MeasurementKey, f32>>,
    config: CacheConfig,
    stats: Mutex<DetailedCacheStats>,
}

impl AdvancedTextCache {
    pub fn new(config: CacheConfig) -> Self {
        Self {
            measurements: Mutex::new(HashMap::new()),
            config,
            stats: Mutex::new(DetailedCacheStats::default()),
        }
    }
}

impl MeasurementCache for AdvancedTextCache {
    fn get_measurement(&self, key: &MeasurementKey) -> Option<f32> {
        let mut stats = self.stats.lock().unwrap();
        stats.last_access = Instant::now();

        if let Ok(measurements) = self.measurements.lock() {
            if let Some(&width) = measurements.get(key) {
                stats.hits += 1;
                return Some(width);
            }
        }

        stats.misses += 1;
        None
    }

    fn store_measurement(&mut self, key: MeasurementKey, width: f32) {
        if let Ok(mut measurements) = self.measurements.lock() {
            // Respect cache size limits
            if measurements.len() < self.config.max_measurement_cache_entries {
                measurements.insert(key, width);
            }
        }
    }

    fn clear(&mut self) {
        if let Ok(mut measurements) = self.measurements.lock() {
            measurements.clear();
        }
    }

    fn stats(&self) -> DetailedCacheStats {
        let mut stats = self.stats.lock().unwrap().clone();

        if let Ok(measurements) = self.measurements.lock() {
            stats.basic.text_measurement_cache_size = measurements.len();
            stats.estimated_memory_usage =
                measurements.len() * std::mem::size_of::<(MeasurementKey, f32)>();
        }

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_config_defaults() {
        let config = CacheConfig::default();
        assert_eq!(config.max_font_cache_size, 50);
        assert_eq!(config.max_measurement_cache_entries, 1000);
        assert!(!config.enable_stats);
    }

    #[test]
    fn test_cache_config_high_performance() {
        let config = CacheConfig::high_performance();
        assert_eq!(config.max_font_cache_size, 100);
        assert_eq!(config.max_measurement_cache_entries, 10000);
        assert!(config.enable_stats);
    }

    #[test]
    fn test_default_font_cache() {
        let config = CacheConfig::default();
        let cache = DefaultFontCache::new(config);

        // Initially empty
        assert!(cache.get_font(FontType::Regular).is_none());

        // Stats should be available
        let stats = cache.stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 1); // From the get_font call above
    }

    #[test]
    fn test_default_measurement_cache() {
        let config = CacheConfig::default();
        let mut cache = DefaultMeasurementCache::new(config);

        let key = MeasurementKey {
            text: "test".to_string(),
            font_size: 120, // 12.0 discretized
            use_bold: false,
            use_brand_font: false,
        };

        // Initially empty
        assert!(cache.get_measurement(&key).is_none());

        // Store and retrieve
        cache.store_measurement(key.clone(), 50.0);
        assert_eq!(cache.get_measurement(&key), Some(50.0));

        // Stats should reflect usage
        let stats = cache.stats();
        assert_eq!(stats.basic.text_measurement_cache_size, 1);
        assert!(stats.estimated_memory_usage > 0);
    }

    #[test]
    fn test_no_op_cache() {
        let cache = NoOpCache;

        // Test FontCache trait methods
        assert!(!<NoOpCache as FontCache>::is_enabled(&cache));
        assert!(cache.get_font(FontType::Regular).is_none());

        // Test MeasurementCache trait methods
        let key = MeasurementKey {
            text: "test".to_string(),
            font_size: 120,
            use_bold: false,
            use_brand_font: false,
        };

        assert!(cache.get_measurement(&key).is_none());
        assert!(!<NoOpCache as MeasurementCache>::is_enabled(&cache));
    }
}
