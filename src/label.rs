//! Label data structures and parsing for ShipLabel
//!
//! This module provides the core data structures for representing shipping label data,
//! including the LabelData structure and RowType enum for different table row types.

use crate::error::{ShipLabelError, ShipLabelResult};
use serde::{Deserialize, Serialize};

/// Represents a complete shipping label with all its data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelData(pub Vec<Vec<String>>);

/// Enum representing the three different types of table rows in a shipping label
#[derive(Debug, Clone, PartialEq)]
pub enum RowType {
    /// Header row containing recipient information (name, address, phone)
    Header(Vec<String>),
    /// QR content row containing item details and brand information
    QrContent(String, Vec<String>),
    /// Order information row containing order ID and date
    OrderInfo(String, String),
}

impl LabelData {
    /// Create a new LabelData from a vector of rows
    pub fn new(rows: Vec<Vec<String>>) -> Self {
        Self(rows)
    }

    /// Get the rows as a slice
    pub fn rows(&self) -> &[Vec<String>] {
        &self.0
    }

    /// Convert LabelData to RowType variants for rendering
    ///
    /// # Returns
    /// A vector of RowType enums representing the parsed label data
    ///
    /// # Errors
    /// Returns an error if the data structure doesn't match expected format
    pub fn to_row_types(&self) -> ShipLabelResult<Vec<RowType>> {
        if self.0.len() != 3 {
            return Err(ShipLabelError::Data(
                format!("Expected 3 rows, got {}", self.0.len())
            ));
        }

        let mut row_types = Vec::new();

        // Parse header row (recipient info)
        if self.0[0].len() < 3 {
            return Err(ShipLabelError::Data(
                "Header row must have at least 3 fields (name, address, phone)".to_string()
            ));
        }
        row_types.push(RowType::Header(self.0[0].clone()));

        // Parse QR/content row (items and brand info)
        if self.0[1].len() != 2 {
            return Err(ShipLabelError::Data(
                "QR row must have exactly 2 fields (items, brand_info)".to_string()
            ));
        }

        // The second field should be an array of brand lines
        // For now, we'll handle it as a single string and parse brand info separately
        let items_text = self.0[1][0].clone();
        let brand_info = self.0[1][1].clone();

        // Parse brand info - this could be a JSON array or formatted text
        let brand_lines = Self::parse_brand_info(&brand_info)?;
        row_types.push(RowType::QrContent(items_text, brand_lines));

        // Parse order info row (order ID and date)
        if self.0[2].len() != 2 {
            return Err(ShipLabelError::Data(
                "Order row must have exactly 2 fields (order_id, date)".to_string()
            ));
        }
        row_types.push(RowType::OrderInfo(
            self.0[2][0].clone(),
            self.0[2][1].clone(),
        ));

        Ok(row_types)
    }

    /// Parse brand information from various formats
    ///
    /// Supports both JSON array format and formatted text
    fn parse_brand_info(brand_info: &str) -> ShipLabelResult<Vec<String>> {
        // First try to parse as JSON array
        if let Ok(parsed) = serde_json::from_str::<Vec<String>>(brand_info) {
            return Ok(parsed);
        }

        // If JSON parsing fails, try to parse as formatted text
        // Look for common separators like newlines or commas
        if brand_info.contains('\n') {
            Ok(brand_info.split('\n').map(|s| s.trim().to_string()).collect())
        } else if brand_info.contains(',') {
            Ok(brand_info.split(',').map(|s| s.trim().to_string()).collect())
        } else {
            // Single line, return as one item
            Ok(vec![brand_info.to_string()])
        }
    }

    /// Validate the label data structure
    pub fn validate(&self) -> ShipLabelResult<()> {
        // Check for empty data
        if self.0.is_empty() {
            return Err(ShipLabelError::Data("Label data cannot be empty".to_string()));
        }

        // Validate row count
        self.validate_row_count()?;

        // Validate each row structure
        self.validate_header_row()?;
        self.validate_qr_row()?;
        self.validate_order_row()?;

        Ok(())
    }

    /// Validate that we have exactly 3 rows
    fn validate_row_count(&self) -> ShipLabelResult<()> {
        if self.0.len() != 3 {
            return Err(ShipLabelError::Data(
                format!("Expected 3 rows, got {}", self.0.len())
            ));
        }
        Ok(())
    }

    /// Validate header row has required fields
    fn validate_header_row(&self) -> ShipLabelResult<()> {
        if self.0[0].len() < 3 {
            return Err(ShipLabelError::Data(
                "Header row must have at least name, address, and phone".to_string()
            ));
        }
        Ok(())
    }

    /// Validate QR row has required fields
    fn validate_qr_row(&self) -> ShipLabelResult<()> {
        if self.0[1].len() != 2 {
            return Err(ShipLabelError::Data(
                "QR row must have items text and brand information".to_string()
            ));
        }
        Ok(())
    }

    /// Validate order row has required fields
    fn validate_order_row(&self) -> ShipLabelResult<()> {
        if self.0[2].len() != 2 {
            return Err(ShipLabelError::Data(
                "Order row must have order ID and date".to_string()
            ));
        }
        Ok(())
    }
}

impl From<Vec<Vec<String>>> for LabelData {
    fn from(rows: Vec<Vec<String>>) -> Self {
        Self(rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_label_data_creation() {
        let rows = vec![
            vec!["John Doe".to_string(), "123 Main St".to_string(), "555-0123".to_string()],
            vec!["items: T-shirt".to_string(), "[\"Brand\", \"Contact\"]".to_string()],
            vec!["#0001".to_string(), "2024-01-01".to_string()],
        ];

        let label = LabelData::new(rows);
        assert_eq!(label.rows().len(), 3);
    }

    #[test]
    fn test_to_row_types_valid_data() {
        let rows = vec![
            vec!["John Doe".to_string(), "123 Main St".to_string(), "555-0123".to_string()],
            vec!["items: T-shirt".to_string(), "[\"Brand Name\", \"Contact\"]".to_string()],
            vec!["#0001".to_string(), "2024-01-01".to_string()],
        ];

        let label = LabelData::new(rows);
        let row_types = label.to_row_types().unwrap();

        assert_eq!(row_types.len(), 3);

        match &row_types[0] {
            RowType::Header(fields) => {
                assert_eq!(fields.len(), 3);
                assert_eq!(fields[0], "John Doe");
            }
            _ => panic!("Expected Header row"),
        }

        match &row_types[1] {
            RowType::QrContent(items, brand) => {
                assert_eq!(items, "items: T-shirt");
                assert_eq!(brand, &vec!["Brand Name", "Contact"]);
            }
            _ => panic!("Expected QrContent row"),
        }

        match &row_types[2] {
            RowType::OrderInfo(order_id, date) => {
                assert_eq!(order_id, "#0001");
                assert_eq!(date, "2024-01-01");
            }
            _ => panic!("Expected OrderInfo row"),
        }
    }

    #[test]
    fn test_to_row_types_invalid_structure() {
        // Test with wrong number of rows
        let rows = vec![
            vec!["John Doe".to_string()],
        ];
        let label = LabelData::new(rows);
        assert!(label.to_row_types().is_err());

        // Test with wrong number of fields in row
        let rows = vec![
            vec!["John Doe".to_string(), "123 Main St".to_string(), "555-0123".to_string()],
            vec!["items".to_string()], // Missing brand info
            vec!["#0001".to_string(), "2024-01-01".to_string()],
        ];
        let label = LabelData::new(rows);
        assert!(label.to_row_types().is_err());
    }

    #[test]
    fn test_parse_brand_info_json() {
        let brand_json = r#"["Brand Name", "TikTok: @brand", "IG: @brand"]"#;
        let result = LabelData::parse_brand_info(brand_json).unwrap();
        assert_eq!(result, vec!["Brand Name", "TikTok: @brand", "IG: @brand"]);
    }

    #[test]
    fn test_parse_brand_info_newlines() {
        let brand_text = "Brand Name\nTikTok: @brand\nIG: @brand";
        let result = LabelData::parse_brand_info(brand_text).unwrap();
        assert_eq!(result, vec!["Brand Name", "TikTok: @brand", "IG: @brand"]);
    }

    #[test]
    fn test_parse_brand_info_commas() {
        let brand_text = "Brand Name, TikTok: @brand, IG: @brand";
        let result = LabelData::parse_brand_info(brand_text).unwrap();
        assert_eq!(result, vec!["Brand Name", "TikTok: @brand", "IG: @brand"]);
    }

    #[test]
    fn test_parse_brand_info_single_line() {
        let brand_text = "Brand Name";
        let result = LabelData::parse_brand_info(brand_text).unwrap();
        assert_eq!(result, vec!["Brand Name"]);
    }

    #[test]
    fn test_validate_valid_data() {
        let rows = vec![
            vec!["John Doe".to_string(), "123 Main St".to_string(), "555-0123".to_string()],
            vec!["items".to_string(), "brand".to_string()],
            vec!["#0001".to_string(), "2024-01-01".to_string()],
        ];

        let label = LabelData::new(rows);
        assert!(label.validate().is_ok());
    }

    #[test]
    fn test_validate_invalid_data() {
        // Empty data
        let label = LabelData::new(vec![]);
        assert!(label.validate().is_err());

        // Wrong number of rows
        let rows = vec![vec!["test".to_string()]];
        let label = LabelData::new(rows);
        assert!(label.validate().is_err());

        // Insufficient header fields
        let rows = vec![
            vec!["John Doe".to_string()], // Only name
            vec!["items".to_string(), "brand".to_string()],
            vec!["#0001".to_string(), "2024-01-01".to_string()],
        ];
        let label = LabelData::new(rows);
        assert!(label.validate().is_err());
    }

    #[test]
    fn test_from_vec_conversion() {
        let rows = vec![
            vec!["John Doe".to_string(), "123 Main St".to_string(), "555-0123".to_string()],
            vec!["items".to_string(), "brand".to_string()],
            vec!["#0001".to_string(), "2024-01-01".to_string()],
        ];

        let label: LabelData = rows.into();
        assert_eq!(label.rows().len(), 3);
    }
}