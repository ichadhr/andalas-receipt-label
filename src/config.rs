use serde::{Deserialize, Serialize};

/// Main configuration structure for ShipLabel PDF generation
///
/// Config provides type-safe configuration for all aspects of shipping label generation
/// including page dimensions, table layouts, fonts, and QR code settings.
///
/// # Examples
///
/// ## Default Configuration
/// ```
/// use pdf::Config;
///
/// let config = Config::new();
/// assert_eq!(config.page_width, 100.0);
/// assert_eq!(config.page_height, 150.0);
/// assert_eq!(config.table_width, 96.0);
/// ```
///
/// ## Custom Layout Configuration
/// ```
/// use pdf::Config;
///
/// let mut config = Config::new();
/// config.page_width = 120.0;      // Wider page
/// config.page_height = 180.0;     // Taller page
/// config.table_width = 110.0;     // Wider table
/// config.font_size = 10.0;        // Larger font
/// config.debug = true;            // Enable debug mode
/// ```
///
/// ## QR Code Configuration
/// ```
/// use pdf::Config;
///
/// let mut config = Config::new();
/// config.qr_size_ratio = 0.9;     // 90% of table height
/// config.qr_border = 4;           // Thicker border
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Page width in millimeters (default: 100.0)
    pub page_width: f32,
    /// Page height in millimeters (default: 150.0)
    pub page_height: f32,

    /// Table width in millimeters (default: 96.0)
    pub table_width: f32,
    /// Table height in millimeters (default: 70.5)
    pub table_height: f32,
    /// Gap between tables in millimeters (default: 4.5)
    pub table_gap: f32,
    /// Top margin in millimeters (default: 2.0)
    pub margin_top: f32,
    /// Side margin in millimeters (default: 2.0)
    pub margin_side: f32,
    /// Header column 1 width in millimeters (default: 18.0)
    pub header_col1_width: f32,

    /// Regular font size in points (default: 6.0)
    pub font_size: f32,
    /// Brand font size in points (default: 8.0)
    pub brand_font_size: f32,

    /// QR code size ratio relative to table height (default: 0.8)
    pub qr_size_ratio: f32,
    /// QR code border thickness (default: 2)
    pub qr_border: i32,

    /// Row height ratios [header, qr_content, order_info] (default: [0.4, 0.5, 0.1])
    pub row_height_ratios: [f32; 3],
    /// Enable debug mode for additional logging (default: false)
    pub debug: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            page_width: 100.0,
            page_height: 150.0,
            table_width: 96.0,
            table_height: 70.5,
            table_gap: 4.5,
            margin_top: 2.0,
            margin_side: 2.0,
            header_col1_width: 22.0,
            font_size: 4.0,
            brand_font_size: 6.0,
            qr_size_ratio: 0.8,
            qr_border: 2,
            row_height_ratios: [0.4, 0.5, 0.1],
            debug: false,
        }
    }
}

impl Config {
    /// Create a new configuration with defaults
    ///
    /// # Examples
    /// ```
    /// use pdf::Config;
    ///
    /// let config = Config::new();
    /// assert_eq!(config.page_width, 100.0);
    /// assert_eq!(config.page_height, 150.0);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate row heights in mm based on table height and percentages
    ///
    /// Returns a vector of three heights corresponding to [header, qr_content, order_info] rows.
    ///
    /// # Examples
    /// ```
    /// use pdf::Config;
    ///
    /// let config = Config::new();
    /// let heights = config.calculate_row_heights();
    ///
    /// assert_eq!(heights.len(), 3);
    /// assert!((heights[0] - 28.2).abs() < 0.1); // 0.4 * 70.5
    /// assert!((heights[1] - 35.25).abs() < 0.1); // 0.5 * 70.5
    /// assert!((heights[2] - 7.05).abs() < 0.1); // 0.1 * 70.5
    /// ```
    pub fn calculate_row_heights(&self) -> Vec<f32> {
        self.row_height_ratios
            .iter()
            .map(|&ratio| ratio * self.table_height)
            .collect()
    }

    /// Calculate table X position based on page width
    ///
    /// Centers the table horizontally on the page.
    ///
    /// # Examples
    /// ```
    /// use pdf::Config;
    ///
    /// let config = Config::new();
    /// let table_x = config.calculate_table_x();
    ///
    /// // Table should be centered: (100 - 96) / 2 = 2
    /// assert_eq!(table_x, 2.0);
    /// ```
    pub fn calculate_table_x(&self) -> f32 {
        (self.page_width - self.table_width) / 2.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_config_defaults() {
        let config = Config::new();
        assert_eq!(config.page_width, 100.0);
        assert_eq!(config.page_height, 150.0);
        assert_eq!(config.table_width, 96.0);
        assert_eq!(config.font_size, 1.0);
        assert_eq!(config.brand_font_size, 2.0);
        assert_eq!(config.qr_size_ratio, 0.8);
        assert_eq!(config.row_height_ratios, [0.4, 0.5, 0.1]);
    }

    #[test]
    fn test_calculate_row_heights() {
        let config = Config::new();
        let heights = config.calculate_row_heights();
        assert_eq!(heights.len(), 3);
        assert!((heights[0] - 28.2).abs() < 0.1); // 0.4 * 70.5
        assert!((heights[1] - 35.25).abs() < 0.1); // 0.5 * 70.5
        assert!((heights[2] - 7.05).abs() < 0.1); // 0.1 * 70.5
    }

    #[test]
    fn test_calculate_table_x() {
        let config = Config::new();
        let x = config.calculate_table_x();
        assert_eq!(x, 2.0); // (100 - 96) / 2
    }

    // ===== PROPERTY-BASED TESTS =====

    proptest::proptest! {
        #[test]
        fn test_config_calculations_are_consistent(
            page_width in 50.0..2000.0f32,
            page_height in 50.0..2000.0f32,
            table_width in 10.0..500.0f32,
            table_height in 10.0..500.0f32,
            margin_side in 0.0..50.0f32
        ) {
            // Create config with generated values
            let mut config = Config::new();
            config.page_width = page_width;
            config.page_height = page_height;
            config.table_width = table_width.min(page_width); // Ensure table fits
            config.table_height = table_height.min(page_height); // Ensure table fits
            config.margin_side = margin_side;

            // Test that calculations don't panic and produce reasonable results
            let row_heights = config.calculate_row_heights();
            let table_x = config.calculate_table_x();

            // Row heights should sum to table height
            let total_height: f32 = row_heights.iter().sum();
            prop_assert!((total_height - config.table_height).abs() < 0.001);

            // Table X should be non-negative and table should fit on page
            prop_assert!(table_x >= 0.0);
            prop_assert!(table_x + config.table_width <= config.page_width);
        }

        #[test]
        fn test_config_row_ratios_are_valid(
            ratio1 in 0.0..1.0f32,
            ratio2 in 0.0..1.0f32,
            ratio3 in 0.0..1.0f32
        ) {
            // Ensure ratios sum to approximately 1.0
            let sum = ratio1 + ratio2 + ratio3;
            if sum > 0.0 {
                let normalized = [ratio1 / sum, ratio2 / sum, ratio3 / sum];

                let mut config = Config::new();
                config.row_height_ratios = normalized;

                let row_heights = config.calculate_row_heights();
                let total_height: f32 = row_heights.iter().sum();

                // Should sum to table height
                prop_assert!((total_height - config.table_height).abs() < 0.001);
            }
        }

        #[test]
        fn test_config_extreme_values_dont_panic(
            page_width in 0.1..10000.0f32,
            page_height in 0.1..10000.0f32,
            table_width in 0.1..1000.0f32,
            table_height in 0.1..1000.0f32,
            font_size in 0.1..1000.0f32,
            qr_size_ratio in 0.0..10.0f32
        ) {
            let mut config = Config::new();
            config.page_width = page_width;
            config.page_height = page_height;
            config.table_width = table_width;
            config.table_height = table_height;
            config.font_size = font_size;
            config.qr_size_ratio = qr_size_ratio;

            // These operations should not panic
            let _row_heights = config.calculate_row_heights();
            let _table_x = config.calculate_table_x();

            // Values should be finite
            prop_assert!(config.page_width.is_finite());
            prop_assert!(config.page_height.is_finite());
            prop_assert!(config.table_width.is_finite());
            prop_assert!(config.table_height.is_finite());
            prop_assert!(config.font_size.is_finite());
        }
    }
}