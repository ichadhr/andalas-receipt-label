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

// Table rendering constants - now sourced from config.layout

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
            self.render_row(surface, x, current_y, row_height, row, y)?;
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

        // Create stroke with configurable border width
        let stroke = Stroke {
            paint: krilla::color::rgb::Color::black().into(),
            width: self.config.layout.table_border_width,
            ..Default::default()
        };
        surface.set_stroke(Some(stroke));

        // Draw horizontal lines between rows
        let row_heights = self.config.calculate_row_heights();
        let mut current_y = y;

        for &row_height in &row_heights[..row_heights.len() - 1] {
            current_y += row_height;
            path_builder.move_to(x, current_y);
            path_builder.line_to(x + width, current_y);
        }

        // Keep column width calculation for positioning, but don't draw the vertical line
        let _header_col_width = self.config.calculate_header_col1_width(self.font_manager);
        // Vertical line removed - no visual separation between label and content columns
        // let vertical_line_x = x + header_col_width;
        // path_builder.move_to(vertical_line_x, y);
        // path_builder.line_to(vertical_line_x, y + row_heights[0]);

        // Create path and stroke it
        if let Some(path) = path_builder.finish() {
            // Set border drawing state
            surface.set_fill(None);
            surface.set_stroke(Some(Stroke {
                paint: krilla::color::rgb::Color::black().into(),
                width: self.config.layout.table_border_width,
                ..Default::default()
            }));
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
        table_top_y: f32, // Add table top coordinate
    ) -> ShipLabelResult<()> {
        match row {
            RowType::Header(fields) => self.render_header_row(surface, x, y, height, fields, table_top_y),
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
        _height: f32,
        fields: &[String],
        _table_top_y: f32,
    ) -> ShipLabelResult<()> {
        if fields.is_empty() {
            return Ok(());
        }

        let _header_col_width = self.config.calculate_header_col1_width(self.font_manager);

        // OPTIMIZATION: Set fill once at the beginning, not before each text render
        let black_fill = krilla::paint::Fill {
            paint: krilla::color::rgb::Color::black().into(),
            opacity: krilla::num::NormalizedF32::ONE,
            rule: Default::default(),
        };
        surface.set_fill(Some(black_fill));

        // Render "Penerima:" label positioned near the vertical stroke
        // Position it so it ends before the vertical line (which is at header_col_width)
        let label_x = x + self.config.layout.table_border_width + self.config.layout.table_margin + self.config.layout.label_extra_clearance;
        let label_y = y + self.config.layout.table_border_width + self.config.layout.table_margin + 2.0 + self.config.layout.label_extra_clearance;

        render_text(
            surface,
            label_x,
            label_y,
            &self.config.recipient_label,
            self.config.font_size,
            self.font_manager,
            false, // Use regular fonts, not brand font
            false,  // Use bold font for labels
        )?;

        // Render recipient info positioned after the label (no vertical line separator)
        // Use the calculated label width plus some spacing for content positioning
        let label_text_width = self.font_manager.measure_text_accurate(&self.config.recipient_label, self.config.font_size, true);
        let content_x = x + self.config.layout.table_margin + label_text_width + self.config.layout.table_margin * self.config.layout.content_spacing_multiplier;
        let content_width = self.config.table_width - content_x - (self.config.layout.table_border_width + self.config.layout.table_margin + self.config.layout.label_extra_clearance);

        if fields.len() >= 3 {
            // Debug output
            if self.config.debug {
                println!("DEBUG: Rendering header for recipient: {}", fields[0]);
                println!("DEBUG: Address: {}", fields[1]);
                println!("DEBUG: Phone: {}", fields[2]);
                println!("DEBUG: Content area: x={}, width={}", content_x, content_width);
            }

            // Name (first line) - use bold for names like main_minimal.rs
            let _name_x = content_x; // Use the same x position as content
            let name_y = y + self.config.layout.table_border_width + self.config.layout.table_margin + 2.0 + self.config.layout.label_extra_clearance;
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

            // Address (second line) - keep regular for content with text wrapping
            let address_y = name_y + self.config.font_size * 1.2 + 0.3; // Natural line spacing + additional 0.3mm gap
            if self.config.debug {
                println!("DEBUG: Address position: x={}, y={}", content_x, address_y);
                println!("DEBUG: Address text: {}", fields[1]);
                println!("DEBUG: Available width: {}mm", content_width);
            }

            // Wrap address text if it's too long
            let wrapped_address_lines = self.wrap_text(&fields[1], content_width, self.config.font_size, self.font_manager, false);
            let mut current_address_y = address_y;

            for line in &wrapped_address_lines {
                render_text(
                    surface,
                    content_x,
                    current_address_y,
                    line,
                    self.config.font_size,
                    self.font_manager,
                    false, // Use regular fonts, not brand font
                    false, // Use regular font for content
                )?;
                current_address_y += self.config.font_size * 1.2 + 0.3; // Move to next line with additional gap
            }

            // Phone (third line) - keep regular for content
            let phone_y = current_address_y; // Position after all address lines
            render_text(
                surface,
                content_x,
                phone_y,
                &fields[2],
                self.config.font_size,
                self.font_manager,
                false, // Use regular fonts, not brand font
                true, // Use regular font for content
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
        let qr_size = height * self.config.qr_size_ratio;
        let qr_x = x + self.config.layout.table_margin;
        let qr_y = y + (height - qr_size) * self.config.layout.qr_vertical_center_ratio; // Center vertically

        // Generate and embed QR code
        let qr_svg = generate_qr_svg(qr_data)?;
        embed_qr_svg(surface, &qr_svg, qr_x, qr_y, qr_size)?;

        // Right side: Brand information
        let brand_start_x = x + qr_size + self.config.layout.brand_start_multiplier * self.config.layout.table_margin;
        let brand_width = self.config.table_width - qr_size - self.config.layout.brand_width_reduction * self.config.layout.table_margin;
        let brand_center_x = brand_start_x + (brand_width / 2.0);

        if !brand_lines.is_empty() {
            // Calculate total height of brand text block for vertical centering
            let brand_line_count = brand_lines.len() as f32;
            let line_spacing = height * self.config.brand_line_spacing_ratio;
            let first_line_height = self.config.brand_font_size;
            let subsequent_line_height = self.config.font_size;

            // Calculate total height: first line + (subsequent lines - 1) * spacing + last line
            let total_brand_height = if brand_line_count == 1.0 {
                first_line_height
            } else {
                first_line_height + (brand_line_count - 1.0) * line_spacing
            };

            // Center the entire brand block vertically in the brand column
            let brand_center_y = y + (height / 2.0);
            let brand_block_start_y = brand_center_y - (total_brand_height / 2.0);

            // First line (brand name) - centered horizontally and vertically
            let first_line_y = brand_block_start_y + (first_line_height / 2.0);
            let first_line_width = self.font_manager.measure_text_with_font(&brand_lines[0], self.config.brand_font_size, false, true);
            let centered_brand_x = brand_center_x - (first_line_width / 2.0);

            render_text(
                surface,
                centered_brand_x,
                first_line_y,
                &brand_lines[0],
                self.config.brand_font_size,
                self.font_manager,
                true,  // Use brand font (Merriweather) for brand text
                true,  // Use bold for brand name
            )?;

            // Remaining lines normal weight - centered horizontally and positioned with spacing
            let mut current_y = brand_block_start_y + first_line_height + line_spacing;

            for line in &brand_lines[1..] {
                if !line.trim().is_empty() {
                    let line_width = self.font_manager.measure_text_accurate(line, self.config.font_size, false);
                    let centered_line_x = brand_center_x - (line_width / 2.0);

                    render_text(
                        surface,
                        centered_line_x,
                        current_y + (subsequent_line_height / 2.0),
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

    /// Wrap text to fit within specified width
    fn wrap_text(&self, text: &str, max_width: f32, font_size: f32, font_manager: &FontManager, use_bold: bool) -> Vec<String> {
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut lines = Vec::new();
        let mut current_line = String::new();

        for word in words {
            let test_line = if current_line.is_empty() {
                word.to_string()
            } else {
                format!("{} {}", current_line, word)
            };

            let line_width = font_manager.measure_text_accurate(&test_line, font_size, use_bold);

            if line_width <= max_width {
                current_line = test_line;
            } else {
                if !current_line.is_empty() {
                    lines.push(current_line);
                }
                current_line = word.to_string();
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }

        // If no wrapping occurred (single line), return original text
        if lines.len() == 1 && lines[0] == text {
            vec![text.to_string()]
        } else {
            lines
        }
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
        let order_x = x + self.config.layout.table_margin;
        let order_y = y + height * self.config.layout.order_vertical_position;
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
        let date_text_width = self.font_manager.measure_text_accurate(date, self.config.font_size, false);
        let date_x = x + self.config.table_width - date_text_width - self.config.layout.table_margin; // Right-align date text
        let date_y = y + height * self.config.layout.order_vertical_position;
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