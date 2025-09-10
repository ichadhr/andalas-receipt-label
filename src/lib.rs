//! # ShipLabel
//!
//! A Rust library for generating shipping label PDFs, ported from PHP TcPdfLib.
//!
//! This library provides functionality to create shipping labels with:
//! - Dynamic table rendering
//! - QR code generation
//! - Multiple label support
//! - Cut guidelines
//! - Custom fonts and styling


pub mod config;
pub mod error;
pub mod font;
pub mod label;
pub mod qr;
pub mod renderer;
pub mod table;
pub mod text;

pub use config::*;
pub use error::*;
pub use font::*;
pub use label::*;
pub use qr::*;
pub use renderer::*;
pub use table::*;
pub use text::*;

/// Main ShipLabel library structure for generating shipping label PDFs
///
/// ShipLabel provides a high-level interface for creating shipping labels with:
/// - Configurable page layouts and dimensions
/// - Embedded Google Fonts (Roboto, Merriweather)
/// - QR code generation and embedding
/// - Type-safe configuration system
///
/// # Examples
///
/// ## Basic Usage
/// ```
/// use pdf::ShipLabel;
///
/// // Create with default configuration
/// let shiplabel = ShipLabel::new()?;
/// println!("Page size: {}mm x {}mm",
///          shiplabel.config().page_width,
///          shiplabel.config().page_height);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// ## Custom Configuration
/// ```
/// use pdf::{ShipLabel, Config};
///
/// let mut config = Config::new();
/// config.page_width = 120.0;
/// config.page_height = 180.0;
/// config.debug = true;
///
/// let shiplabel = ShipLabel::with_config(config)?;
/// assert_eq!(shiplabel.config().page_width, 120.0);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct ShipLabel {
    config: Config,
    font_manager: FontManager,
    document: krilla::Document,
}

impl std::fmt::Debug for ShipLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShipLabel")
            .field("config", &self.config)
            .field("font_manager", &"<FontManager>")
            .field("document", &"<KrillaDocument - not debuggable>")
            .finish()
    }
}

impl ShipLabel {
    /// Create a new ShipLabel instance with default configuration
    ///
    /// # Examples
    /// ```
    /// use pdf::ShipLabel;
    ///
    /// let shiplabel = ShipLabel::new()?;
    /// assert_eq!(shiplabel.config().page_width, 100.0);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new() -> ShipLabelResult<Self> {
        Self::with_config(Config::new())
    }

    /// Create a new ShipLabel instance with custom configuration
    ///
    /// # Examples
    /// ```
    /// use pdf::{ShipLabel, Config};
    ///
    /// let mut config = Config::new();
    /// config.page_width = 120.0;
    /// config.page_height = 180.0;
    ///
    /// let shiplabel = ShipLabel::with_config(config)?;
    /// assert_eq!(shiplabel.config().page_width, 120.0);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn with_config(config: Config) -> ShipLabelResult<Self> {
        let font_manager = FontManager::new()?;
        let document = krilla::Document::new();

        Ok(Self {
            config,
            font_manager,
            document,
        })
    }

    /// Get the current configuration
    ///
    /// # Examples
    /// ```
    /// use pdf::ShipLabel;
    ///
    /// let shiplabel = ShipLabel::new()?;
    /// let config = shiplabel.config();
    /// println!("Page width: {}mm", config.page_width);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Update the configuration
    ///
    /// # Examples
    /// ```
    /// use pdf::{ShipLabel, Config};
    ///
    /// let mut shiplabel = ShipLabel::new()?;
    /// let mut new_config = shiplabel.config().clone();
    /// new_config.debug = true;
    ///
    /// shiplabel.set_config(new_config);
    /// assert!(shiplabel.config().debug);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn set_config(&mut self, config: Config) {
        self.config = config;
    }

    /// Get the font manager for accessing embedded fonts
    ///
    /// # Examples
    /// ```
    /// use pdf::ShipLabel;
    ///
    /// let shiplabel = ShipLabel::new()?;
    /// let font_manager = shiplabel.font_manager();
    ///
    /// // Access different font variants
    /// let regular_font = font_manager.regular();
    /// let bold_font = font_manager.bold();
    /// let brand_font = font_manager.brand();
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn font_manager(&self) -> &FontManager {
        &self.font_manager
    }

    /// Get the underlying krilla document for advanced PDF operations
    ///
    /// # Examples
    /// ```
    /// use pdf::ShipLabel;
    ///
    /// let shiplabel = ShipLabel::new()?;
    /// let document = shiplabel.document();
    ///
    /// // Access krilla document for advanced operations
    /// // (Note: This is for advanced users familiar with krilla)
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn document(&self) -> &krilla::Document {
        &self.document
    }

    /// Get mutable access to the document for advanced operations
    ///
    /// # Examples
    /// ```
    /// use pdf::ShipLabel;
    ///
    /// let mut shiplabel = ShipLabel::new()?;
    /// let document = shiplabel.document_mut();
    ///
    /// // Perform advanced krilla operations
    /// // (Note: This is for advanced users familiar with krilla)
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn document_mut(&mut self) -> &mut krilla::Document {
        &mut self.document
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test data constants to reduce duplication
    const TEST_QR_CONTENT: &str = "https://example.com/test";
    const TEST_UNICODE_CONTENT: &str = "José María";

    #[test]
    fn test_shiplabel_creation() {
        let shiplabel = ShipLabel::new().unwrap();
        assert_eq!(shiplabel.config().page_width, 100.0);
        assert_eq!(shiplabel.config().page_height, 150.0);
    }

    #[test]
    fn test_shiplabel_with_config() {
        let mut config = Config::new();
        config.debug = true;
        let shiplabel = ShipLabel::with_config(config).unwrap();
        assert!(shiplabel.config().debug);
    }

    #[test]
    fn test_shiplabel_debug_implementation() {
        let shiplabel = ShipLabel::new().unwrap();
        let debug_str = format!("{:?}", shiplabel);
        assert!(debug_str.contains("ShipLabel"));
    }

    #[test]
    fn test_basic_krilla_integration() {
        use krilla::geom::Size;
        let _document = krilla::Document::new();
        let size = Size::from_wh(100.0, 150.0).expect("Valid size");
        assert_eq!(size.width(), 100.0);
        assert_eq!(size.height(), 150.0);
    }

    #[test]
    fn test_font_manager_integration() {
        let shiplabel = ShipLabel::new().unwrap();
        let font_manager = shiplabel.font_manager();

        // Test all font access methods
        let regular = font_manager.regular();
        let bold = font_manager.bold();
        let brand = font_manager.brand();

        // Verify fonts are different objects
        assert!(!std::ptr::eq(regular, bold));
        assert!(!std::ptr::eq(regular, brand));

        // Test get_font method
        assert!(!std::ptr::eq(font_manager.get_font(false), font_manager.get_font(true)));
    }

    #[test]
    fn test_configuration_calculations() {
        let config = Config::new();
        let row_heights = config.calculate_row_heights();
        let table_x = config.calculate_table_x();

        // Basic validation
        assert_eq!(row_heights.len(), 3);
        assert!(table_x >= 0.0);
        assert!(table_x < config.page_width);

        // Row heights should sum to table height
        let total_height: f32 = row_heights.iter().sum();
        assert!((total_height - config.table_height).abs() < 0.001);
    }

    #[test]
    fn test_qr_integration() {
        let shiplabel = ShipLabel::new().unwrap();
        let svg = generate_qr_svg(TEST_QR_CONTENT).unwrap();

        // Verify SVG generation
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));

        // Test QR size calculation
        let qr_size = shiplabel.config().table_height * shiplabel.config().qr_size_ratio;
        assert!(qr_size > 0.0 && qr_size <= shiplabel.config().table_height);
    }

    #[test]
    fn test_edge_cases() {
        // Test various QR content edge cases
        let long_content = "Long content".repeat(10);
        let test_cases = vec![
            "".to_string(),
            TEST_UNICODE_CONTENT.to_string(),
            "a".to_string(),
            "1234567890".to_string(),
            "!@#$%^&*()".to_string(),
            long_content
        ];

        for content in test_cases {
            assert!(generate_qr_svg(&content).is_ok(),
                "QR generation failed for: {}", content);
        }

        // Test extreme config values
        let mut config = Config::new();
        config.page_width = 10.0;
        config.page_height = 10.0;
        config.table_width = 5.0;
        config.table_height = 5.0;
        config.font_size = 1.0;

        // Should not panic
        let _ = config.calculate_row_heights();
        let _ = config.calculate_table_x();
    }

    #[test]
    fn test_data_format_validation() {
        // Test parsing sample JSON format
        let sample_data = "[
            [
                [\"John Doe\", \"123 Main St\", \"555-0123\"],
                [\"items: T-shirt x2\", [\"Brand Name\", \"Website\"]],
                [\"#0001\", \"2024-01-01\"]
            ]
        ]";

        let parsed: serde_json::Value = serde_json::from_str(sample_data).unwrap();
        let labels = parsed.as_array().unwrap();
        assert_eq!(labels.len(), 1);

        let label = &labels[0];
        let rows = label.as_array().unwrap();
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].as_array().unwrap().len(), 3); // Header
        assert_eq!(rows[1].as_array().unwrap().len(), 2); // QR content
        assert_eq!(rows[2].as_array().unwrap().len(), 2); // Order info
    }

    #[test]
    fn test_unicode_handling() {
        let unicode_data = format!("[\"{TEST_UNICODE_CONTENT}\", \"Test Address\", \"555-0123\"]");
        let parsed: serde_json::Value = serde_json::from_str(&unicode_data).unwrap();
        let name = parsed[0].as_str().unwrap();
        assert_eq!(name, TEST_UNICODE_CONTENT);
    }

    #[test]
    fn test_performance_basics() {
        use std::time::Instant;

        // Font loading performance
        let start = Instant::now();
        let _font_manager = FontManager::new().unwrap();
        assert!(start.elapsed().as_millis() < 100);

        // QR generation performance
        let start = Instant::now();
        let _ = generate_qr_svg("Test content").unwrap();
        assert!(start.elapsed().as_millis() < 50);

        // Config calculation performance (100 iterations)
        let start = Instant::now();
        for _ in 0..100 {
            let config = Config::new();
            let _ = config.calculate_row_heights();
            let _ = config.calculate_table_x();
        }
        assert!(start.elapsed().as_millis() < 1);
    }

    #[test]
    fn test_memory_usage() {
        let shiplabel = ShipLabel::new().unwrap();
        let size = std::mem::size_of_val(&shiplabel);
        assert!(size < 2048, "ShipLabel too large: {} bytes", size);
    }
}