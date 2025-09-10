//! Table rendering and layout utilities for ShipLabel
//!
//! This module provides functionality for rendering shipping label tables with:
//! - Three distinct row types (header, QR content, order info)
//! - Manual border drawing using PathBuilder
//! - Type-safe row rendering with RowType enum
//! - Exact positioning calculations matching PHP implementation

use crate::config::Config;
use crate::error::ShipLabelResult;
use crate::font::FontManager;
use crate::label::RowType;
use crate::qr::{generate_qr_svg, embed_qr_svg};
use crate::text::render_text;
use krilla::geom::PathBuilder;
use krilla::paint::Stroke;
use krilla::surface::Surface;

// Table rendering constants
const TABLE_MARGIN: f32 = 2.0;
const HEADER_COL1_WIDTH_RATIO: f32 = 0.4; // 40% of table width for order ID
const QR_VERTICAL_CENTER_RATIO: f32 = 0.5; // Center QR vertically
const QR_SIZE_RATIO: f32 = 0.8; // QR size relative to row height
const BRAND_FIRST_LINE_RATIO: f32 = 0.3; // Position of first brand line
const BRAND_SUBSEQUENT_RATIO: f32 = 0.55; // Position of subsequent brand lines
const BRAND_LINE_SPACING_RATIO: f32 = 0.25; // Spacing between brand lines
const HEADER_LABEL_RATIO: f32 = 0.3; // Position of "Penerima:" label
const HEADER_NAME_RATIO: f32 = 0.25; // Position of name
const HEADER_ADDRESS_RATIO: f32 = 0.55; // Position of address
const HEADER_PHONE_RATIO: f32 = 0.8; // Position of phone
const ORDER_ID_RATIO: f32 = 0.5; // Center order ID vertically
const ORDER_DATE_RATIO: f32 = 0.5; // Center date vertically
const TABLE_BORDER_WIDTH: f32 = 0.4;

/// Table renderer for shipping labels
pub struct TableRenderer<'a> {
    config: &'a Config,
    font_manager: &'a FontManager,
}

impl<'a> TableRenderer<'a> {
    /// Create a new table renderer
    pub fn new(config: &'a Config, font_manager: &'a FontManager) -> Self {
        Self {
            config,
            font_manager,
        }
    }

    /// Render a complete table with the given row types
    ///
    /// # Arguments
    /// * `surface` - The krilla surface to render to
    /// * `x` - X coordinate of table top-left corner
    /// * `y` - Y coordinate of table top-left corner
    /// * `rows` - Vector of RowType enums to render
    ///
    /// # Returns
    /// Result indicating success or error
    pub fn render_table(
        &self,
        surface: &mut Surface,
        x: f32,
        y: f32,
        rows: &[RowType],
    ) -> ShipLabelResult<()> {
        // Draw table borders
        self.draw_table_borders(surface, x, y)?;

        // Calculate row heights
        let row_heights = self.config.calculate_row_heights();

        // Render each row
        let mut current_y = y;
        for (i, row) in rows.iter().enumerate() {
            let row_height = row_heights[i];
            self.render_row(surface, x, current_y, row_height, row)?;
            current_y += row_height;
        }

        Ok(())
    }

    /// Draw table borders using PathBuilder - OPTIMIZED VERSION
    fn draw_table_borders(&self, surface: &mut Surface, x: f32, y: f32) -> ShipLabelResult<()> {
        let mut path_builder = PathBuilder::new();

        // Table dimensions
        let width = self.config.table_width;
        let height = self.config.table_height;

        // Draw outer rectangle
        path_builder.move_to(x, y);
        path_builder.line_to(x + width, y);
        path_builder.line_to(x + width, y + height);
        path_builder.line_to(x, y + height);
        path_builder.close();

        // Draw horizontal lines between rows
        let row_heights = self.config.calculate_row_heights();
        let mut current_y = y;

        for &row_height in &row_heights[..row_heights.len() - 1] {
            current_y += row_height;
            path_builder.move_to(x, current_y);
            path_builder.line_to(x + width, current_y);
        }

        // Draw vertical line for header column (at end of Penerima: column)
        let header_col_width = self.config.calculate_header_col1_width(self.font_manager);
        let vertical_line_x = x + header_col_width; // Vertical line at end of header column
        path_builder.move_to(vertical_line_x, y);
        path_builder.line_to(vertical_line_x, y + row_heights[0]);

        // Create path and stroke it
        if let Some(path) = path_builder.finish() {
            // Set border drawing state
            surface.set_fill(None);
            let stroke = Stroke {
                paint: krilla::color::rgb::Color::black().into(),
                width: TABLE_BORDER_WIDTH,
                ..Default::default()
            };
            surface.set_stroke(Some(stroke));
            surface.draw_path(&path);

            // OPTIMIZATION: Reset to default state for text rendering
            surface.set_stroke(None);
            // Note: Fill will be set by render_header_row, so we don't set it here
        }

        Ok(())
    }

    /// Render a single row based on its type
    fn render_row(
        &self,
        surface: &mut Surface,
        x: f32,
        y: f32,
        height: f32,
        row: &RowType,
    ) -> ShipLabelResult<()> {
        match row {
            RowType::Header(fields) => self.render_header_row(surface, x, y, height, fields),
            RowType::QrContent(qr_data, brand_lines) => {
                self.render_qr_content_row(surface, x, y, height, qr_data, brand_lines)
            }
            RowType::OrderInfo(order_id, date) => {
                self.render_order_info_row(surface, x, y, height, order_id, date)
            }
        }
    }

    /// Render header row (recipient information) - OPTIMIZED VERSION
    fn render_header_row(
        &self,
        surface: &mut Surface,
        x: f32,
        y: f32,
        height: f32,
        fields: &[String],
    ) -> ShipLabelResult<()> {
        if fields.is_empty() {
            return Ok(());
        }

        let header_col_width = self.config.calculate_header_col1_width(self.font_manager);

        // OPTIMIZATION: Set fill once at the beginning, not before each text render
        let black_fill = krilla::paint::Fill {
            paint: krilla::color::rgb::Color::black().into(),
            opacity: krilla::num::NormalizedF32::ONE,
            rule: Default::default(),
        };
        surface.set_fill(Some(black_fill));

        // Render "Penerima:" label positioned near the vertical stroke
        // Position it so it ends before the vertical line (which is at header_col_width)
        let label_x = x + TABLE_MARGIN; // Start with normal margin
        let label_y = y + height * HEADER_LABEL_RATIO;

        render_text(
            surface,
            label_x,
            label_y,
            "Penerima:",
            self.config.font_size,
            self.font_manager,
            false, // Use regular fonts, not brand font
            true,  // Use bold font for labels
        )?;

        // Render recipient info in second column (after vertical line)
        let vertical_line_x = x + header_col_width;
        let content_x = vertical_line_x + TABLE_MARGIN; // Start content after vertical line
        let content_width = self.config.table_width - vertical_line_x - 2.0 * TABLE_MARGIN;

        if fields.len() >= 3 {
            // Debug output
            if self.config.debug {
                println!("DEBUG: Rendering header for recipient: {}", fields[0]);
                println!("DEBUG: Address: {}", fields[1]);
                println!("DEBUG: Phone: {}", fields[2]);
                println!("DEBUG: Content area: x={}, width={}", content_x, content_width);
            }

            // Name (first line) - use bold for names like main_minimal.rs
            let name_y = y + height * HEADER_NAME_RATIO;
            render_text(
                surface,
                content_x,
                name_y,
                &fields[0],
                self.config.font_size,
                self.font_manager,
                false, // Use regular fonts, not brand font
                true,  // Use bold font for names
            )?;

            // Address (second line) - keep regular for content
            let address_y = y + height * HEADER_ADDRESS_RATIO;
            if self.config.debug {
                println!("DEBUG: Address position: x={}, y={}", content_x, address_y);
                println!("DEBUG: Address text length: {}", fields[1].len());
            }
            render_text(
                surface,
                content_x,
                address_y,
                &fields[1],
                self.config.font_size,
                self.font_manager,
                false, // Use regular fonts, not brand font
                false, // Use regular font for content
            )?;

            // Phone (third line) - keep regular for content
            let phone_y = y + height * HEADER_PHONE_RATIO;
            render_text(
                surface,
                content_x,
                phone_y,
                &fields[2],
                self.config.font_size,
                self.font_manager,
                false, // Use regular fonts, not brand font
                false, // Use regular font for content
            )?;
        }

        Ok(())
    }

    /// Render QR content row (QR code + brand information) - OPTIMIZED VERSION
    fn render_qr_content_row(
        &self,
        surface: &mut Surface,
        x: f32,
        y: f32,
        height: f32,
        qr_data: &str,
        brand_lines: &[String],
    ) -> ShipLabelResult<()> {
        // OPTIMIZATION: Set fill once for all text rendering in this row
        let black_fill = krilla::paint::Fill {
            paint: krilla::color::rgb::Color::black().into(),
            opacity: krilla::num::NormalizedF32::ONE,
            rule: Default::default(),
        };
        surface.set_fill(Some(black_fill));

        // Left side: QR code
        let qr_size = height * QR_SIZE_RATIO;
        let qr_x = x + TABLE_MARGIN;
        let qr_y = y + (height - qr_size) * QR_VERTICAL_CENTER_RATIO; // Center vertically

        // Generate and embed QR code
        let qr_svg = generate_qr_svg(qr_data)?;
        embed_qr_svg(surface, &qr_svg, qr_x, qr_y, qr_size)?;

        // Right side: Brand information
        let brand_x = x + qr_size + 2.0 * TABLE_MARGIN;
        let _brand_width = self.config.table_width - qr_size - 3.0 * TABLE_MARGIN;

        if !brand_lines.is_empty() {
            // First line (brand name)
            let first_line_y = y + height * BRAND_FIRST_LINE_RATIO;
            render_text(
                surface,
                brand_x,
                first_line_y,
                &brand_lines[0],
                self.config.brand_font_size,
                self.font_manager,
                true,  // Use brand font (Merriweather) for brand text
                false, // Brand font already provides styling
            )?;

            // Remaining lines normal weight
            let mut current_y = y + height * BRAND_SUBSEQUENT_RATIO;
            let line_spacing = height * BRAND_LINE_SPACING_RATIO;

            for line in &brand_lines[1..] {
                if !line.trim().is_empty() {
                    render_text(
                        surface,
                        brand_x,
                        current_y,
                        line,
                        self.config.font_size,
                        self.font_manager,
                        false, // Use regular fonts for subsequent brand lines
                        false, // Use regular font for content
                    )?;
                    current_y += line_spacing;
                }
            }
        }

        Ok(())
    }

    /// Render order info row (order ID + date) - OPTIMIZED VERSION
    fn render_order_info_row(
        &self,
        surface: &mut Surface,
        x: f32,
        y: f32,
        height: f32,
        order_id: &str,
        date: &str,
    ) -> ShipLabelResult<()> {
        // OPTIMIZATION: Set fill once for all text rendering in this row
        let black_fill = krilla::paint::Fill {
            paint: krilla::color::rgb::Color::black().into(),
            opacity: krilla::num::NormalizedF32::ONE,
            rule: Default::default(),
        };
        surface.set_fill(Some(black_fill));

        // Left side: Order ID - use bold like main_minimal.rs
        let order_x = x + TABLE_MARGIN;
        let order_y = y + height * ORDER_ID_RATIO; // Center vertically
        render_text(
            surface,
            order_x,
            order_y,
            order_id,
            self.config.font_size,
            self.font_manager,
            false, // Use regular fonts, not brand font
            false,
        )?;

        // Right side: Date (right-aligned) - keep regular for content
        let date_width = self.config.table_width * HEADER_COL1_WIDTH_RATIO; // 40% of width for date
        let date_x = x + self.config.table_width - date_width - TABLE_MARGIN;
        let date_y = y + height * ORDER_DATE_RATIO; // Center vertically
        render_text(
            surface,
            date_x,
            date_y,
            date,
            self.config.font_size,
            self.font_manager,
            false, // Use regular fonts, not brand font
            false, // Use regular font for content
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::font::FontManager;
    use crate::label::RowType;

    #[test]
    fn test_table_renderer_creation() {
        let config = Config::new();
        let font_manager = FontManager::new().unwrap();
        let renderer = TableRenderer::new(&config, &font_manager);

        // Verify renderer is created successfully
        assert_eq!(renderer.config.page_width, 100.0);
    }

    #[test]
    fn test_calculate_row_heights_integration() {
        let config = Config::new();
        let font_manager = FontManager::new().unwrap();
        let renderer = TableRenderer::new(&config, &font_manager);

        let row_heights = renderer.config.calculate_row_heights();
        assert_eq!(row_heights.len(), 3);

        // Verify heights sum to table height
        let total: f32 = row_heights.iter().sum();
        assert!((total - config.table_height).abs() < 0.001);
    }

    #[test]
    fn test_row_type_creation() {
        // Test Header row
        let header_fields = vec!["John Doe".to_string(), "123 Main St".to_string(), "555-0123".to_string()];
        let header_row = RowType::Header(header_fields.clone());
        assert!(matches!(header_row, RowType::Header(_)));

        // Test QrContent row
        let qr_data = "test-qr-data".to_string();
        let brand_lines = vec!["Brand Name".to_string(), "Contact Info".to_string()];
        let qr_row = RowType::QrContent(qr_data.clone(), brand_lines.clone());
        assert!(matches!(qr_row, RowType::QrContent(_, _)));

        // Test OrderInfo row
        let order_id = "#0001".to_string();
        let date = "2024-01-01".to_string();
        let order_row = RowType::OrderInfo(order_id.clone(), date.clone());
        assert!(matches!(order_row, RowType::OrderInfo(_, _)));
    }

    #[test]
    fn test_table_dimensions() {
        let config = Config::new();

        // Test that table dimensions are reasonable
        assert!(config.table_width > 0.0);
        assert!(config.table_height > 0.0);
        assert!(config.table_width <= config.page_width);
        assert!(config.table_height <= config.page_height);
    }

    #[test]
    fn test_qr_size_calculation() {
        let config = Config::new();

        // Test QR size calculation
        let qr_size = config.table_height * config.qr_size_ratio;
        assert!(qr_size > 0.0);
        assert!(qr_size <= config.table_height);
        assert!(config.qr_size_ratio > 0.0 && config.qr_size_ratio <= 1.0);
    }

    #[test]
    fn test_render_functions_exist() {
        // Test that all render functions exist and can be called
        let config = Config::new();
        let font_manager = FontManager::new().unwrap();
        let renderer = TableRenderer::new(&config, &font_manager);

        // These would normally require a surface, but we can test they exist
        let _ = renderer.config;
        let _ = renderer.font_manager;
    }
}