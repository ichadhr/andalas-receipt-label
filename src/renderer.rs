//! PDF rendering orchestration and page management for ShipLabel
//!
//! This module provides the main rendering engine that:
//! - Manages PDF document creation and page handling
//! - Orchestrates table rendering across multiple pages
//! - Handles label positioning and layout
//! - Adds cut guidelines between labels
//! - Tracks available space and creates new pages as needed

use crate::config::Config;
use crate::error::ShipLabelResult;
use crate::font::FontManager;
use crate::label::LabelData;
use crate::table::TableRenderer;
use krilla::page::PageSettings;
use krilla::Document;

// Cut guideline rendering constants
const CUT_GUIDELINE_DASH_LENGTH: f32 = 4.0;
const CUT_GUIDELINE_GAP_LENGTH: f32 = 2.0;
const CUT_GUIDELINE_STROKE_WIDTH: f32 = 0.3;
const CUT_GUIDELINE_MITER_LIMIT: f32 = 2.0;

/// Main PDF renderer for shipping labels
pub struct LabelRenderer {
    config: Config,
    font_manager: FontManager,
    document: Document,
    labels_on_current_page: usize,
    page_count: usize,
    max_labels_per_page: usize,
    pending_labels: Vec<LabelData>,
}

impl LabelRenderer {
    /// Create a new label renderer
    pub fn new(config: Config, font_manager: FontManager) -> Self {
        let document = Document::new();

        Self {
            config,
            font_manager,
            document,
            labels_on_current_page: 0,
            page_count: 0,
            max_labels_per_page: 2, // Default to 2 labels per page
            pending_labels: Vec::new(),
        }
    }

    /// Create a new label renderer with default configuration
    pub fn with_defaults(font_manager: FontManager) -> ShipLabelResult<Self> {
        let config = Config::new();
        Ok(Self::new(config, font_manager))
    }

    /// Render a single label
    ///
    /// # Arguments
    /// * `label_data` - The label data to render
    ///
    /// # Returns
    /// Result indicating success or error
    pub fn render_label(&mut self, label_data: &LabelData) -> ShipLabelResult<()> {
        // Add label to pending batch
        self.pending_labels.push(label_data.clone());

        // If we have enough labels for a page, render them
        if self.pending_labels.len() >= self.max_labels_per_page {
            self.render_pending_page()?;
        }

        Ok(())
    }

    /// Render all pending labels on a new page
    fn render_pending_page(&mut self) -> ShipLabelResult<()> {
        if self.pending_labels.is_empty() {
            return Ok(());
        }

        // Create table renderer
        let table_renderer = TableRenderer::new(&self.config, &self.font_manager);

        // Create a new page
        let page_size =
            krilla::geom::Size::from_wh(self.config.page_width, self.config.page_height)
                .ok_or_else(|| {
                    crate::error::ShipLabelError::Config("Invalid page dimensions".to_string())
                })?;

        let page_settings = PageSettings::new(page_size);
        let mut page = self.document.start_page_with(page_settings);
        let mut surface = page.surface();

        let table_x = self.config.calculate_table_x();
        let mut current_y = self.config.margin_top;

        // Render each label on this page
        for (i, label_data) in self.pending_labels.iter().enumerate() {
            // Convert label data to row types
            let row_types = label_data.to_row_types()?;

            // Calculate Y position for this label
            let table_y = current_y;

            // Render the table
            table_renderer.render_table(&mut surface, table_x, table_y, &row_types)?;

            // Add cut guideline if not the last label on page
            if i < self.pending_labels.len() - 1 {
                // Position cut guideline between labels (in the middle of the gap)
                let cut_guideline_y =
                    table_y + self.config.table_height + (self.config.table_gap / 2.0);
                Self::add_cut_guideline(&mut surface, cut_guideline_y, &self.config)?;
            }

            // Update Y position for next label
            current_y += self.config.table_height + self.config.table_gap;
        }

        // Clear pending labels and update counters
        self.pending_labels.clear();
        self.labels_on_current_page = 0;
        self.page_count += 1;

        // Finish the surface and page
        surface.finish();
        page.finish();

        Ok(())
    }

    /// Render multiple labels
    ///
    /// # Arguments
    /// * `labels` - Vector of label data to render
    ///
    /// # Returns
    /// Result indicating success or error
    pub fn render_labels(&mut self, labels: &[LabelData]) -> ShipLabelResult<()> {
        for label in labels {
            self.render_label(label)?;
        }
        Ok(())
    }

    /// Finish rendering and return the PDF data
    ///
    /// # Returns
    /// PDF data as bytes
    pub fn finish(mut self) -> ShipLabelResult<Vec<u8>> {
        // Render any remaining pending labels
        if !self.pending_labels.is_empty() {
            self.render_pending_page()?;
        }

        // Finish document and return PDF data
        let pdf_data = self.document.finish().map_err(|e| {
            crate::error::ShipLabelError::Pdf(format!("Failed to finish PDF document: {:?}", e))
        })?;

        Ok(pdf_data)
    }

    /// Add a cut guideline between labels
    pub fn add_cut_guideline(
        surface: &mut krilla::surface::Surface,
        y_position: f32,
        config: &Config,
    ) -> ShipLabelResult<()> {
        // Calculate the horizontal span for the cut guideline (full page width)
        let line_start_x = 0.0; // Start from left edge of page
        let line_end_x = config.page_width; // End at right edge of page

        // Create a horizontal dashed line path
        let mut path_builder = krilla::geom::PathBuilder::new();
        path_builder.move_to(line_start_x, y_position);
        path_builder.line_to(line_end_x, y_position);

        if let Some(path) = path_builder.finish() {
            // Create dashed stroke for cut guidelines
            let dash_pattern = krilla::paint::StrokeDash {
                array: vec![CUT_GUIDELINE_DASH_LENGTH, CUT_GUIDELINE_GAP_LENGTH],
                offset: 0.0,
            };

            let stroke = krilla::paint::Stroke {
                paint: krilla::color::rgb::Color::black().into(),
                width: CUT_GUIDELINE_STROKE_WIDTH,
                miter_limit: CUT_GUIDELINE_MITER_LIMIT,
                line_cap: krilla::paint::LineCap::Square,
                line_join: krilla::paint::LineJoin::Miter,
                opacity: krilla::num::NormalizedF32::ONE,
                dash: Some(dash_pattern),
            };

            // Apply stroke and draw the path
            surface.set_stroke(Some(stroke));
            surface.draw_path(&path);
        }

        Ok(())
    }

    /// Get the current page count
    pub fn page_count(&self) -> usize {
        self.page_count
    }

    /// Get the total number of labels rendered so far
    pub fn labels_rendered(&self) -> usize {
        (self.page_count * self.max_labels_per_page) + self.labels_on_current_page
    }

    /// Get access to the underlying document (for advanced operations)
    pub fn document(&self) -> &Document {
        &self.document
    }

    /// Get mutable access to the underlying document
    pub fn document_mut(&mut self) -> &mut Document {
        &mut self.document
    }

    /// Get the current configuration
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Update the configuration
    pub fn set_config(&mut self, config: Config) {
        self.config = config;
    }

    /// Get the maximum number of labels per page
    pub fn max_labels_per_page(&self) -> usize {
        self.max_labels_per_page
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::FontManager;
    use crate::label::LabelData;

    #[test]
    fn test_renderer_creation() {
        let font_manager = FontManager::new().unwrap();
        let renderer = LabelRenderer::with_defaults(font_manager).unwrap();

        assert_eq!(renderer.page_count(), 0);
        assert_eq!(renderer.labels_rendered(), 0);
    }

    #[test]
    fn test_renderer_with_config() {
        let config = Config::new();
        let font_manager = FontManager::new().unwrap();
        let renderer = LabelRenderer::new(config.clone(), font_manager);

        assert_eq!(renderer.config().page_width, config.page_width);
        assert_eq!(renderer.config().page_height, config.page_height);
    }

    #[test]
    fn test_label_data_conversion() {
        // Test that we can convert sample data to row types
        let rows = vec![
            vec![
                "John Doe".to_string(),
                "123 Main St".to_string(),
                "555-0123".to_string(),
            ],
            vec!["items: T-shirt".to_string(), "[\"Brand Name\"]".to_string()],
            vec!["#0001".to_string(), "2024-01-01".to_string()],
        ];

        let label_data = LabelData::new(rows);
        let row_types = label_data.to_row_types().unwrap();

        assert_eq!(row_types.len(), 3);
        assert!(matches!(row_types[0], crate::label::RowType::Header(_)));
        assert!(matches!(
            row_types[1],
            crate::label::RowType::QrContent(_, _)
        ));
        assert!(matches!(
            row_types[2],
            crate::label::RowType::OrderInfo(_, _)
        ));
    }

    #[test]
    fn test_config_calculations() {
        let config = Config::new();

        // Test table positioning
        let table_x = config.calculate_table_x();
        assert!(table_x >= 0.0);
        assert!(table_x < config.page_width);

        // Test row heights
        let row_heights = config.calculate_row_heights();
        assert_eq!(row_heights.len(), 3);
        let total: f32 = row_heights.iter().sum();
        assert!((total - config.table_height).abs() < 0.001);
    }

    #[test]
    fn test_empty_label_handling() {
        // Empty label should be handled gracefully
        let empty_label = LabelData::new(vec![]);
        assert!(empty_label.to_row_types().is_err());
    }

    #[test]
    fn test_invalid_label_structure() {
        // Label with wrong number of rows
        let invalid_label = LabelData::new(vec![vec!["test".to_string()]]);
        assert!(invalid_label.to_row_types().is_err());
    }

    #[test]
    fn test_renderer_state_tracking() {
        let font_manager = FontManager::new().unwrap();
        let renderer = LabelRenderer::with_defaults(font_manager).unwrap();

        // Test initial state
        assert_eq!(renderer.labels_rendered(), 0);
        assert_eq!(renderer.page_count(), 0);
    }

    #[test]
    fn test_document_access() {
        let font_manager = FontManager::new().unwrap();
        let mut renderer = LabelRenderer::with_defaults(font_manager).unwrap();

        // Should be able to access document
        let _doc = renderer.document();
        let _doc_mut = renderer.document_mut();
    }

    #[test]
    fn test_sample_data_integration() {
        use serde_json;
        use std::fs;

        // Load sample data
        let sample_data_path = "input/sample.json";
        let sample_content =
            fs::read_to_string(sample_data_path).expect("Failed to read sample.json");

        // Parse as raw JSON first to handle the complex structure
        let raw_labels: Vec<serde_json::Value> =
            serde_json::from_str(&sample_content).expect("Failed to parse sample.json");

        assert_eq!(raw_labels.len(), 8, "Should have 8 sample labels");

        // Convert first label to LabelData format
        let first_label_value = &raw_labels[0];
        let first_label_array = first_label_value.as_array().unwrap();

        // Extract the three rows
        let header_row = first_label_array[0].as_array().unwrap();
        let qr_row = first_label_array[1].as_array().unwrap();
        let order_row = first_label_array[2].as_array().unwrap();

        // Convert to LabelData format
        let mut label_data_rows = Vec::new();

        // Header row: [name, address, phone]
        let header_strings: Vec<String> = header_row
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect();
        label_data_rows.push(header_strings);

        // QR row: [items_text, brand_array] -> [items_text, brand_json_string]
        let items_text = qr_row[0].as_str().unwrap().to_string();
        let brand_array = qr_row[1].as_array().unwrap();
        let brand_strings: Vec<String> = brand_array
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect();
        let brand_json = serde_json::to_string(&brand_strings).unwrap();
        label_data_rows.push(vec![items_text, brand_json]);

        // Order row: [order_id, date]
        let order_strings: Vec<String> = order_row
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect();
        label_data_rows.push(order_strings);

        let label_data = LabelData::new(label_data_rows);

        // Test parsing
        let row_types = label_data.to_row_types().unwrap();
        assert_eq!(row_types.len(), 3, "Should have 3 row types");

        // Verify row types
        match &row_types[0] {
            crate::label::RowType::Header(fields) => {
                assert_eq!(fields.len(), 3, "Header should have 3 fields");
                assert_eq!(fields[0], "Aswanto Iwan", "First field should be name");
            }
            _ => panic!("First row should be Header"),
        }

        match &row_types[1] {
            crate::label::RowType::QrContent(qr_data, brand_lines) => {
                assert!(qr_data.contains("items:"), "QR data should contain items");
                assert!(!brand_lines.is_empty(), "Should have brand lines");
                assert_eq!(
                    brand_lines[0], "Andalas Branded",
                    "First brand line should be correct"
                );
            }
            _ => panic!("Second row should be QrContent"),
        }

        match &row_types[2] {
            crate::label::RowType::OrderInfo(order_id, date) => {
                assert_eq!(order_id, "#0001", "Order ID should be #0001");
                assert_eq!(date, "1202025", "Date should be 1202025");
            }
            _ => panic!("Third row should be OrderInfo"),
        }

        println!("✅ Sample data integration test passed - first label parsed correctly");
    }

    #[test]
    fn test_complete_rendering_pipeline() {
        use serde_json;
        use std::fs;

        // Load sample data
        let sample_data_path = "input/sample.json";
        let sample_content =
            fs::read_to_string(sample_data_path).expect("Failed to read sample.json");

        // Parse as raw JSON first to handle the complex structure
        let raw_labels: Vec<serde_json::Value> =
            serde_json::from_str(&sample_content).expect("Failed to parse sample.json");

        // Convert first 4 labels to LabelData format for testing
        let mut test_labels = Vec::new();

        for i in 0..4 {
            let label_value = &raw_labels[i];
            let label_array = label_value.as_array().unwrap();

            // Extract the three rows
            let header_row = label_array[0].as_array().unwrap();
            let qr_row = label_array[1].as_array().unwrap();
            let order_row = label_array[2].as_array().unwrap();

            // Convert to LabelData format
            let mut label_data_rows = Vec::new();

            // Header row: [name, address, phone]
            let header_strings: Vec<String> = header_row
                .iter()
                .map(|v| v.as_str().unwrap().to_string())
                .collect();
            label_data_rows.push(header_strings);

            // QR row: [items_text, brand_array] -> [items_text, brand_json_string]
            let items_text = qr_row[0].as_str().unwrap().to_string();
            let brand_array = qr_row[1].as_array().unwrap();
            let brand_strings: Vec<String> = brand_array
                .iter()
                .map(|v| v.as_str().unwrap().to_string())
                .collect();
            let brand_json = serde_json::to_string(&brand_strings).unwrap();
            label_data_rows.push(vec![items_text, brand_json]);

            // Order row: [order_id, date]
            let order_strings: Vec<String> = order_row
                .iter()
                .map(|v| v.as_str().unwrap().to_string())
                .collect();
            label_data_rows.push(order_strings);

            test_labels.push(LabelData::new(label_data_rows));
        }

        // Create renderer
        let font_manager = FontManager::new().unwrap();
        let mut renderer = LabelRenderer::with_defaults(font_manager).unwrap();

        // Render first 2 labels (should fit on one page with 2 labels per page default)
        for i in 0..2 {
            renderer.render_label(&test_labels[i]).unwrap();
        }

        // Verify state
        assert_eq!(
            renderer.labels_rendered(),
            2,
            "Should have rendered 2 labels"
        );
        assert_eq!(renderer.page_count(), 1, "Should be on page 1");

        // Render 2 more labels (should go to page 2)
        for i in 2..4 {
            renderer.render_label(&test_labels[i]).unwrap();
        }

        // Verify state after second page
        assert_eq!(
            renderer.labels_rendered(),
            4,
            "Should have rendered 4 labels"
        );
        assert_eq!(renderer.page_count(), 2, "Should be on page 2");

        // Finish rendering
        let pdf_data = renderer.finish().unwrap();
        assert!(!pdf_data.is_empty(), "PDF data should not be empty");

        println!(
            "✅ Complete rendering pipeline test passed - generated {} bytes PDF",
            pdf_data.len()
        );
    }

    #[test]
    fn test_multi_page_rendering() {
        use serde_json;
        use std::fs;

        // Load sample data
        let sample_data_path = "input/sample.json";
        let sample_content =
            fs::read_to_string(sample_data_path).expect("Failed to read sample.json");

        // Parse as raw JSON first to handle the complex structure
        let raw_labels: Vec<serde_json::Value> =
            serde_json::from_str(&sample_content).expect("Failed to parse sample.json");

        // Convert all 8 labels to LabelData format
        let mut test_labels = Vec::new();

        for i in 0..8 {
            let label_value = &raw_labels[i];
            let label_array = label_value.as_array().unwrap();

            // Extract the three rows
            let header_row = label_array[0].as_array().unwrap();
            let qr_row = label_array[1].as_array().unwrap();
            let order_row = label_array[2].as_array().unwrap();

            // Convert to LabelData format
            let mut label_data_rows = Vec::new();

            // Header row: [name, address, phone]
            let header_strings: Vec<String> = header_row
                .iter()
                .map(|v| v.as_str().unwrap().to_string())
                .collect();
            label_data_rows.push(header_strings);

            // QR row: [items_text, brand_array] -> [items_text, brand_json_string]
            let items_text = qr_row[0].as_str().unwrap().to_string();
            let brand_array = qr_row[1].as_array().unwrap();
            let brand_strings: Vec<String> = brand_array
                .iter()
                .map(|v| v.as_str().unwrap().to_string())
                .collect();
            let brand_json = serde_json::to_string(&brand_strings).unwrap();
            label_data_rows.push(vec![items_text, brand_json]);

            // Order row: [order_id, date]
            let order_strings: Vec<String> = order_row
                .iter()
                .map(|v| v.as_str().unwrap().to_string())
                .collect();
            label_data_rows.push(order_strings);

            test_labels.push(LabelData::new(label_data_rows));
        }

        // Create renderer with 1 label per page
        let font_manager = FontManager::new().unwrap();
        let config = Config::new();
        let mut renderer = LabelRenderer::new(config, font_manager);
        renderer.max_labels_per_page = 1; // Override to 1 label per page

        // Render all 8 labels
        for label in &test_labels {
            renderer.render_label(label).unwrap();
        }

        // Verify final state
        assert_eq!(
            renderer.labels_rendered(),
            8,
            "Should have rendered all 8 labels"
        );
        assert_eq!(renderer.page_count(), 8, "Should have 8 pages");

        // Finish rendering
        let pdf_data = renderer.finish().unwrap();
        assert!(!pdf_data.is_empty(), "PDF data should not be empty");

        println!(
            "✅ Multi-page rendering test passed - 8 labels on 8 pages, {} bytes PDF",
            pdf_data.len()
        );
    }

    #[test]
    fn test_complete_label_rendering_integration() {
        use serde_json;
        use std::fs;

        // Load all sample data
        let sample_data_path = "input/sample.json";
        let sample_content =
            fs::read_to_string(sample_data_path).expect("Failed to read sample.json");

        // Parse as raw JSON first to handle the complex structure
        let raw_labels: Vec<serde_json::Value> =
            serde_json::from_str(&sample_content).expect("Failed to parse sample.json");

        // Convert all labels to LabelData format
        let mut all_labels = Vec::new();

        for label_value in &raw_labels {
            let label_array = label_value.as_array().unwrap();

            // Extract the three rows
            let header_row = label_array[0].as_array().unwrap();
            let qr_row = label_array[1].as_array().unwrap();
            let order_row = label_array[2].as_array().unwrap();

            // Convert to LabelData format
            let mut label_data_rows = Vec::new();

            // Header row: [name, address, phone]
            let header_strings: Vec<String> = header_row
                .iter()
                .map(|v| v.as_str().unwrap().to_string())
                .collect();
            label_data_rows.push(header_strings);

            // QR row: [items_text, brand_array] -> [items_text, brand_json_string]
            let items_text = qr_row[0].as_str().unwrap().to_string();
            let brand_array = qr_row[1].as_array().unwrap();
            let brand_strings: Vec<String> = brand_array
                .iter()
                .map(|v| v.as_str().unwrap().to_string())
                .collect();
            let brand_json = serde_json::to_string(&brand_strings).unwrap();
            label_data_rows.push(vec![items_text, brand_json]);

            // Order row: [order_id, date]
            let order_strings: Vec<String> = order_row
                .iter()
                .map(|v| v.as_str().unwrap().to_string())
                .collect();
            label_data_rows.push(order_strings);

            all_labels.push(LabelData::new(label_data_rows));
        }

        // Create renderer with default settings (2 labels per page)
        let font_manager = FontManager::new().unwrap();
        let mut renderer = LabelRenderer::with_defaults(font_manager).unwrap();

        // Store configuration values before consuming renderer
        let max_labels_per_page = renderer.max_labels_per_page;

        // Render all 8 labels
        for label in &all_labels {
            renderer.render_label(label).unwrap();
        }

        // Verify final state
        assert_eq!(
            renderer.labels_rendered(),
            8,
            "Should have rendered all 8 labels"
        );
        assert_eq!(
            renderer.page_count(),
            4,
            "Should have 4 pages (8 labels / 2 per page)"
        );

        // Finish rendering and get PDF data
        let pdf_data = renderer.finish().unwrap();
        assert!(!pdf_data.is_empty(), "PDF data should not be empty");
        assert!(pdf_data.len() > 1000, "PDF should be substantial in size");

        // Save the PDF for manual inspection
        let output_path = "output/complete_test_labels.pdf";
        fs::write(output_path, &pdf_data).expect("Failed to write test PDF");

        println!("✅ Complete label rendering integration test PASSED!");
        println!("   📄 Generated PDF: {}", output_path);
        println!("   📊 Total labels: {}", all_labels.len());
        println!("   📄 Total pages: 4");
        println!("   💾 PDF size: {} bytes", pdf_data.len());
        println!("   🎯 Labels per page: {}", max_labels_per_page);

        // Verify PDF structure (basic validation)
        let pdf_str = String::from_utf8_lossy(&pdf_data);
        assert!(pdf_str.contains("%PDF"), "Should be a valid PDF file");
        assert!(
            pdf_str.contains("%%EOF"),
            "PDF should have proper EOF marker"
        );

        println!("   ✅ PDF structure validation passed");
        println!("   🎉 ShipLabel library integration test COMPLETED SUCCESSFULLY!");
    }
}
