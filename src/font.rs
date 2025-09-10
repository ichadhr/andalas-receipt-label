use crate::error::{ShipLabelError, ShipLabelResult};
use krilla::text::Font;
use skrifa::{FontRef, MetadataProvider, instance::{Size, LocationRef}};

/// Font manager for handling embedded Google Fonts
#[derive(Debug, Clone)]
pub struct FontManager {
    /// Regular text font (Roboto Thin)
    pub regular: Font,
    /// Bold text font (Roboto Regular)
    pub bold: Font,
    /// Brand display font (Merriweather Bold)
    pub brand: Font,
}

impl FontManager {
    /// Create a new FontManager with embedded Google Fonts
    pub fn new() -> ShipLabelResult<Self> {
        // Load Roboto Condensed Light (much lighter weight)
        let regular_data = include_bytes!("assets/fonts/Roboto/static/Roboto-Regular.ttf");
        // let regular_data = include_bytes!("assets/fonts/Hevetica/Helvetica.ttf");
        let regular = Font::new(regular_data.to_vec().into(), 0)
            .ok_or_else(|| ShipLabelError::Font("Failed to load Roboto Regular font".to_string()))?;

        // Load Roboto Bold
        let bold_data = include_bytes!("assets/fonts/Roboto/static/Roboto-Bold.ttf");
        // let bold_data = include_bytes!("assets/fonts/Hevetica/Helvetica-Bold.ttf");
        let bold = Font::new(bold_data.to_vec().into(), 0)
            .ok_or_else(|| ShipLabelError::Font("Failed to load Roboto Bold font".to_string()))?;

        // Load Merriweather Bold (brand font)
        let brand_data = include_bytes!("assets/fonts/Merriweather/static/Merriweather_24pt-Bold.ttf");
        let brand = Font::new(brand_data.to_vec().into(), 0)
            .ok_or_else(|| ShipLabelError::Font("Failed to load Merriweather Bold font".to_string()))?;

        Ok(Self {
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

    /// Calculate actual text width using glyph advances from skrifa
    pub fn measure_text_accurate(&self, text: &str, font_size: f32, use_bold: bool) -> f32 {
        // Use embedded font data directly since we know the font files
        let font_bytes: &[u8] = if use_bold {
            include_bytes!("assets/fonts/Roboto/static/Roboto-Bold.ttf")
        } else {
            include_bytes!("assets/fonts/Roboto/static/Roboto-Regular.ttf")
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
    fn measure_text_fallback(&self, text: &str, font_size: f32, use_bold: bool) -> f32 {
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
        Self::new().expect("Failed to load default fonts")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_manager_creation() {
        let _font_manager = FontManager::new().unwrap();
        // Verify fonts are loaded (we can't test much more without krilla internals)
        assert!(true); // If we get here, fonts loaded successfully
    }

    #[test]
    fn test_font_manager_default() {
        let _font_manager = FontManager::default();
        // Should not panic
        assert!(true);
    }

    #[test]
    fn test_font_accessors() {
        let font_manager = FontManager::new().unwrap();

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

}