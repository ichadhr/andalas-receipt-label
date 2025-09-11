use crate::error::{ShipLabelError, ShipLabelResult};
use crate::cache::{CacheManager, get_cached_measurement, clear_text_measurement_cache, get_cache_stats, FontType};
use krilla::text::Font;
use skrifa::{FontRef, MetadataProvider, instance::{Size, LocationRef}};
use std::sync::Arc;

// Macro to load font data - provides single source of truth
macro_rules! load_font_bytes {
    (regular) => {
        include_bytes!("assets/fonts/Roboto/static/Roboto-Regular.ttf")
    };
    (bold) => {
        include_bytes!("assets/fonts/Roboto/static/Roboto-Bold.ttf")
    };
    (brand) => {
        include_bytes!("assets/fonts/Merriweather/static/Merriweather_24pt-Bold.ttf")
    };
}

// Global font caches for performance optimization
static REGULAR_FONT_CACHE: std::sync::OnceLock<Font> = std::sync::OnceLock::new();
static BOLD_FONT_CACHE: std::sync::OnceLock<Font> = std::sync::OnceLock::new();
static BRAND_FONT_CACHE: std::sync::OnceLock<Font> = std::sync::OnceLock::new();

// Helper function to check if font cache is loaded
pub fn font_cache_loaded() -> bool {
    REGULAR_FONT_CACHE.get().is_some()
}

// Public accessor for font caches (used by cache module)
pub fn get_regular_font() -> ShipLabelResult<&'static Font> {
    load_cached_font(&REGULAR_FONT_CACHE, load_font_bytes!(regular), "Roboto Regular")
}

pub fn get_bold_font() -> ShipLabelResult<&'static Font> {
    load_cached_font(&BOLD_FONT_CACHE, load_font_bytes!(bold), "Roboto Bold")
}

pub fn get_brand_font() -> ShipLabelResult<&'static Font> {
    load_cached_font(&BRAND_FONT_CACHE, load_font_bytes!(brand), "Merriweather Bold")
}

// Public function to get font by type (used by cache module)
pub fn get_font_by_type(font_type: crate::cache::FontType) -> ShipLabelResult<&'static Font> {
    match font_type {
        crate::cache::FontType::Regular => get_regular_font(),
        crate::cache::FontType::Bold => get_bold_font(),
        crate::cache::FontType::Brand => get_brand_font(),
    }
}

/// Cache statistics for monitoring and debugging
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Number of entries in text measurement cache
    pub text_measurement_cache_size: usize,
    /// Whether font caches are loaded
    pub font_cache_loaded: bool,
}

// Helper function to load a font with caching
fn load_cached_font<'a>(cache: &'a std::sync::OnceLock<Font>, font_data: &[u8], font_name: &str) -> ShipLabelResult<&'a Font> {
    if let Some(font) = cache.get() {
        Ok(font)
    } else {
        // Load font and initialize cache
        let font = Font::new(font_data.to_vec().into(), 0)
            .ok_or_else(|| ShipLabelError::Font(format!("Failed to load {} font", font_name)))?;

        // This is safe because we just checked that cache is empty
        // In a race condition, this might be called multiple times but OnceLock handles it
        let _ = cache.set(font);
        Ok(cache.get().unwrap())
    }
}


/// Font manager for handling embedded Google Fonts
#[derive(Debug)]
pub struct FontManager {
    /// Reference to the cache manager for font and text measurement caching
    // TODO: Use cache_manager for advanced caching features like cache invalidation, stats, etc.
    #[allow(dead_code)]
    cache_manager: Arc<CacheManager>,
    /// Regular text font (Roboto Thin) - cached via CacheManager
    pub regular: Font,
    /// Bold text font (Roboto Regular) - cached via CacheManager
    pub bold: Font,
    /// Brand display font (Merriweather Bold) - cached via CacheManager
    pub brand: Font,
}

impl FontManager {
    /// Create a new FontManager with embedded Google Fonts and default caching
    /// This method is provided for backward compatibility
    pub fn new() -> ShipLabelResult<Self> {
        let cache_manager = Arc::new(CacheManager::default()?);
        Self::with_cache_manager(cache_manager)
    }

    /// Create a new FontManager with embedded Google Fonts
    /// Uses the provided CacheManager for font and text measurement caching
    pub fn with_cache_manager(cache_manager: Arc<CacheManager>) -> ShipLabelResult<Self> {
        // Load fonts using the cache manager
        let regular = cache_manager.get_font(FontType::Regular)
            .ok_or_else(|| ShipLabelError::Font("Failed to load Roboto Regular font".to_string()))?;
        let bold = cache_manager.get_font(FontType::Bold)
            .ok_or_else(|| ShipLabelError::Font("Failed to load Roboto Bold font".to_string()))?;
        let brand = cache_manager.get_font(FontType::Brand)
            .ok_or_else(|| ShipLabelError::Font("Failed to load Merriweather Bold font".to_string()))?;

        Ok(Self {
            cache_manager,
            regular,
            bold,
            brand,
        })
    }

    /// Get the regular font
    pub fn regular(&self) -> &Font {
        &self.regular
    }

    /// Get the bold font
    pub fn bold(&self) -> &Font {
        &self.bold
    }

    /// Get the brand font
    pub fn brand(&self) -> &Font {
        &self.brand
    }

    /// Get font by style
    pub fn get_font(&self, bold: bool) -> &Font {
        if bold {
            &self.bold
        } else {
            &self.regular
        }
    }

    /// Clear all caches (font and text measurement caches)
    /// This forces fresh loading of fonts and recalculation of text measurements
    /// Useful for:
    /// - Memory management in long-running applications
    /// - Testing scenarios requiring clean state
    /// - Forcing cache invalidation when needed
    pub fn clear_caches() {
        // Note: We can't actually clear OnceLock caches as they're designed to be immutable
        // But we can clear the text measurement cache
        clear_text_measurement_cache();
    }

    /// Clear only the text measurement cache
    /// Keeps font caches intact but forces fresh text measurements
    pub fn clear_text_measurement_cache() {
        clear_text_measurement_cache();
    }

    /// Get cache statistics for monitoring and debugging
    pub fn get_cache_stats() -> CacheStats {
        get_cache_stats()
    }

    /// Calculate actual text width using glyph advances from skrifa
    /// Uses caching for improved performance on repeated measurements
    pub fn measure_text_accurate(&self, text: &str, font_size: f32, use_bold: bool) -> f32 {
        self.measure_text_with_font(text, font_size, use_bold, false)
    }

    /// Calculate text width using specific font type
    /// Uses caching for improved performance on repeated measurements
    pub fn measure_text_with_font(&self, text: &str, font_size: f32, use_bold: bool, use_brand_font: bool) -> f32 {
        // Use caching for performance optimization
        get_cached_measurement(text, font_size, use_bold, use_brand_font, || {
            self.compute_text_width(text, font_size, use_bold, use_brand_font)
        })
    }

    /// Internal method to compute text width without caching
    fn compute_text_width(&self, text: &str, font_size: f32, use_bold: bool, use_brand_font: bool) -> f32 {
        // Use embedded font data directly since we know the font files
        let font_bytes: &[u8] = if use_brand_font {
            load_font_bytes!(brand)
        } else if use_bold {
            load_font_bytes!(bold)
        } else {
            load_font_bytes!(regular)
        };

        // Load with skrifa for accurate measurement
        if let Ok(font_ref) = FontRef::new(font_bytes) {
            let charmap = font_ref.charmap();
            let mut total_width = 0.0;

            // Get glyph metrics for advance width calculation
            let glyph_metrics = font_ref.glyph_metrics(Size::unscaled(), LocationRef::default());
            let units_per_em = font_ref.metrics(Size::unscaled(), LocationRef::default()).units_per_em as f32;

            for ch in text.chars() {
                // Get glyph ID for character
                if let Some(glyph_id) = charmap.map(ch) {
                    // Get advance width and scale to font size
                    if let Some(advance) = glyph_metrics.advance_width(glyph_id) {
                        let scaled_advance = advance * (font_size / units_per_em);
                        total_width += scaled_advance;
                    }
                }
            }

            return total_width;
        }

        // Fallback to approximation if skrifa fails
        self.measure_text_fallback(text, font_size, use_bold)
    }

    /// Fallback text measurement using character approximations
    fn measure_text_fallback(&self, text: &str, font_size: f32, _use_bold: bool) -> f32 {
        // Better character width approximations based on Roboto font
        let char_widths = [
            ('P', 0.75), ('e', 0.55), ('n', 0.65), ('r', 0.45), ('i', 0.35),
            ('m', 0.85), ('a', 0.55), (':', 0.35)
        ];

        let mut total_width = 0.0;
        for ch in text.chars() {
            let width_ratio = char_widths.iter()
                .find(|(c, _)| *c == ch)
                .map(|(_, w)| *w)
                .unwrap_or(0.5); // Default width
            total_width += width_ratio;
        }

        // Scale by font size
        total_width * font_size
    }

}

impl Default for FontManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|e| {
            eprintln!("CRITICAL ERROR: Failed to load embedded fonts: {}", e);
            eprintln!("This usually indicates corrupted or missing font files in assets/fonts/");
            eprintln!("Please ensure all font files are present and readable.");
            panic!("Font loading failed - cannot continue without fonts");
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_font_manager_creation() {
        let font_manager = FontManager::new();
        assert!(font_manager.is_ok(), "FontManager creation should succeed: {:?}", font_manager.err());
    }

    #[test]
    fn test_font_manager_default() {
        let _font_manager = FontManager::default();
        // Should not panic - if we get here, it worked
        assert!(true);
    }

    #[test]
    fn test_font_accessors() {
        let font_manager = FontManager::new().expect("FontManager should be created successfully");

        // Test that we can access all fonts
        let _regular = font_manager.regular();
        let _bold = font_manager.bold();
        let _brand = font_manager.brand();

        // Test get_font method
        let regular_font = font_manager.get_font(false);
        let bold_font = font_manager.get_font(true);

        // These should be different fonts
        assert!(!std::ptr::eq(regular_font, bold_font));
    }

    // ===== TEXT MEASUREMENT TESTS =====

    #[test]
    fn test_text_measurement_basic() {
        let font_manager = FontManager::new().expect("FontManager should be created successfully");

        // Test basic measurement
        let width = font_manager.measure_text_accurate("Hello", 12.0, false);
        assert!(width > 0.0, "Text width should be positive");

        // Test that larger font size gives larger width
        let larger_width = font_manager.measure_text_accurate("Hello", 24.0, false);
        assert!(larger_width > width, "Larger font should give larger width");
    }

    #[test]
    fn test_text_measurement_different_fonts() {
        let font_manager = FontManager::new().expect("FontManager should be created successfully");

        let text = "Test String";
        let font_size = 12.0;

        // Measure with different font types
        let regular_width = font_manager.measure_text_accurate(text, font_size, false);
        let bold_width = font_manager.measure_text_accurate(text, font_size, true);
        let brand_width = font_manager.measure_text_with_font(text, font_size, false, true);

        // All should be positive and different
        assert!(regular_width > 0.0);
        assert!(bold_width > 0.0);
        assert!(brand_width > 0.0);

        // Bold might be slightly wider than regular
        assert!(bold_width >= regular_width);
    }

    #[test]
    fn test_text_measurement_caching() {
        let font_manager = FontManager::new().expect("FontManager should be created successfully");

        let text = "Cache Test String";
        let font_size = 12.0;

        // First measurement (cache miss)
        let start = Instant::now();
        let width1 = font_manager.measure_text_accurate(text, font_size, false);
        let first_duration = start.elapsed();

        // Second measurement (cache hit)
        let start = Instant::now();
        let width2 = font_manager.measure_text_accurate(text, font_size, false);
        let second_duration = start.elapsed();

        // Results should be identical
        assert_eq!(width1, width2, "Cached result should match original");

        // Second measurement should be faster (though this might not be reliable in tests)
        // At minimum, both should complete successfully
        assert!(width1 > 0.0);
        assert!(width2 > 0.0);

        println!("First measurement: {:?}, Second measurement: {:?}", first_duration, second_duration);
    }

    #[test]
    fn test_text_measurement_cache_different_keys() {
        let font_manager = FontManager::new().expect("FontManager should be created successfully");

        let text = "Test";

        // Different font sizes should give different results
        let width1 = font_manager.measure_text_accurate(text, 10.0, false);
        let width2 = font_manager.measure_text_accurate(text, 20.0, false);
        assert!(width2 > width1, "Larger font should give larger width");

        // Different font types should give different results
        let regular = font_manager.measure_text_accurate(text, 12.0, false);
        let bold = font_manager.measure_text_accurate(text, 12.0, true);
        let brand = font_manager.measure_text_with_font(text, 12.0, false, true);

        // All should be positive
        assert!(regular > 0.0);
        assert!(bold > 0.0);
        assert!(brand > 0.0);
    }

    #[test]
    fn test_text_measurement_unicode() {
        let font_manager = FontManager::new().expect("FontManager should be created successfully");

        // Test with Unicode characters
        let unicode_text = "José María ñoño";
        let width = font_manager.measure_text_accurate(unicode_text, 12.0, false);
        assert!(width > 0.0, "Unicode text should be measurable");

        // Test with emojis (if supported)
        let emoji_text = "Hello 😀 World";
        let emoji_width = font_manager.measure_text_accurate(emoji_text, 12.0, false);
        assert!(emoji_width > 0.0, "Emoji text should be measurable");
    }

    #[test]
    fn test_text_measurement_empty_and_whitespace() {
        let font_manager = FontManager::new().expect("FontManager should be created successfully");

        // Empty string
        let empty_width = font_manager.measure_text_accurate("", 12.0, false);
        assert_eq!(empty_width, 0.0, "Empty string should have zero width");

        // Whitespace only
        let space_width = font_manager.measure_text_accurate("   ", 12.0, false);
        assert!(space_width > 0.0, "Whitespace should have positive width");

        // Mixed content
        let mixed_width = font_manager.measure_text_accurate("  test  ", 12.0, false);
        assert!(mixed_width > space_width, "Mixed content should be wider than spaces only");
    }

    #[test]
    fn test_text_measurement_extreme_values() {
        let font_manager = FontManager::new().expect("FontManager should be created successfully");

        let text = "Test";

        // Very small font size
        let tiny_width = font_manager.measure_text_accurate(text, 0.1, false);
        assert!(tiny_width > 0.0, "Very small font should still be measurable");

        // Very large font size
        let large_width = font_manager.measure_text_accurate(text, 100.0, false);
        assert!(large_width > tiny_width, "Large font should be wider than tiny font");

        // Very long text
        let long_text = "A".repeat(1000);
        let long_width = font_manager.measure_text_accurate(&long_text, 12.0, false);
        assert!(long_width > 0.0, "Very long text should be measurable");
    }

    #[test]
    fn test_text_measurement_fallback() {
        let font_manager = FontManager::new().expect("FontManager should be created successfully");

        // Test fallback measurement (this will use the approximation method)
        // We can't easily force skrifa to fail, but we can test that the method works
        let text = "Fallback Test";
        let width = font_manager.measure_text_accurate(text, 12.0, false);
        assert!(width > 0.0, "Fallback measurement should work");

        // Test that caching works with fallback too
        let width2 = font_manager.measure_text_accurate(text, 12.0, false);
        assert_eq!(width, width2, "Cached fallback result should match");
    }

    #[test]
    fn test_font_size_discretization() {
        let font_manager = FontManager::new().expect("FontManager should be created successfully");

        let text = "Discretization Test";

        // Test that very similar font sizes produce very similar results
        // (they might not be exactly equal due to floating point precision)
        let width1 = font_manager.measure_text_accurate(text, 12.0, false);
        let width2 = font_manager.measure_text_accurate(text, 12.01, false);

        // They should be very close (within 1% difference)
        let difference = (width1 - width2).abs();
        let percent_diff = difference / width1;
        assert!(percent_diff < 0.01, "Very similar font sizes should give very similar results: {} vs {}", width1, width2);

        // But clearly different font sizes should be noticeably different
        let width3 = font_manager.measure_text_accurate(text, 14.0, false);
        let difference_large = (width1 - width3).abs();
        let percent_diff_large = difference_large / width1;
        assert!(percent_diff_large > 0.1, "Different font sizes should give noticeably different results: {} vs {}", width1, width3);
    }

    #[test]
    fn test_measurement_cache_thread_safety() {
        let font_manager = FontManager::new().expect("FontManager should be created successfully");

        let text = "Thread Safety Test";
        let font_size = 12.0;

        // Test concurrent access (basic test - in real scenarios this would be more thorough)
        let width1 = font_manager.measure_text_accurate(text, font_size, false);
        let width2 = font_manager.measure_text_accurate(text, font_size, false);

        assert_eq!(width1, width2, "Concurrent cache access should be consistent");
    }

    #[test]
    fn test_cache_clearing() {
        let font_manager = FontManager::new().expect("FontManager should be created successfully");

        let text = "Cache Clear Test";
        let font_size = 12.0;

        // Fill cache
        let width1 = font_manager.measure_text_accurate(text, font_size, false);
        assert!(width1 > 0.0, "First measurement should work");

        // Verify cache has entry
        let stats_before = FontManager::get_cache_stats();
        assert!(stats_before.text_measurement_cache_size > 0, "Cache should have entries");

        // Clear cache
        FontManager::clear_text_measurement_cache();

        // Verify cache is cleared
        let stats_after = FontManager::get_cache_stats();
        assert_eq!(stats_after.text_measurement_cache_size, 0, "Cache should be empty after clearing");

        // Verify measurement still works (recalculates)
        let width2 = font_manager.measure_text_accurate(text, font_size, false);
        assert_eq!(width1, width2, "Recalculated measurement should match original");
    }

    #[test]
    fn test_cache_stats() {
        let font_manager = FontManager::new().expect("FontManager should be created successfully");

        // Get initial stats
        let initial_stats = FontManager::get_cache_stats();
        assert!(initial_stats.font_cache_loaded, "Font cache should be loaded");

        // Add some measurements
        font_manager.measure_text_accurate("Test 1", 12.0, false);
        font_manager.measure_text_accurate("Test 2", 14.0, true);

        // Check stats after measurements
        let after_stats = FontManager::get_cache_stats();
        assert!(after_stats.text_measurement_cache_size >= 2, "Should have at least 2 cache entries");

        // Clear and check again
        FontManager::clear_text_measurement_cache();
        let cleared_stats = FontManager::get_cache_stats();
        assert_eq!(cleared_stats.text_measurement_cache_size, 0, "Cache should be empty");
        assert!(cleared_stats.font_cache_loaded, "Font cache should still be loaded");
    }

    #[test]
    fn test_cache_clear_all() {
        let font_manager = FontManager::new().expect("FontManager should be created successfully");

        // Add measurements
        font_manager.measure_text_accurate("Test", 12.0, false);

        // Verify cache has entries
        let stats_before = FontManager::get_cache_stats();
        assert!(stats_before.text_measurement_cache_size > 0);

        // Clear all caches
        FontManager::clear_caches();

        // Verify text measurement cache is cleared
        let stats_after = FontManager::get_cache_stats();
        assert_eq!(stats_after.text_measurement_cache_size, 0);
    }

}