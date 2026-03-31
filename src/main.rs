use andalas_receipt_label::*;
use serde_json;
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
struct FileConfig {
    input_path: PathBuf,
    output_path: PathBuf,
}

impl Default for FileConfig {
    fn default() -> Self {
        Self {
            input_path: PathBuf::from("input/sample.json"),
            output_path: PathBuf::from("output/complete_rust_labels.pdf"),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 ShipLabel - Complete Shipping Label PDF Generation");
    println!("==================================================");

    let file_config = FileConfig::default();

    // Load sample data
    println!(
        "📂 Loading sample data from {}...",
        file_config.input_path.display()
    );
    let sample_content = fs::read_to_string(&file_config.input_path)
        .map_err(|e| format!("Failed to read {}: {}", file_config.input_path.display(), e))?;

    let raw_labels: Vec<serde_json::Value> =
        serde_json::from_str(&sample_content).map_err(|e| {
            format!(
                "Failed to parse JSON from {}: {}",
                file_config.input_path.display(),
                e
            )
        })?;

    println!("✅ Loaded {} sample labels", raw_labels.len());

    // Convert sample data to LabelData format
    println!("🔄 Converting sample data to LabelData format...");
    let mut all_labels = Vec::new();

    for (i, label_value) in raw_labels.iter().enumerate() {
        let label_array = label_value
            .as_array()
            .ok_or_else(|| format!("Label {}: Expected array, got {:?}", i, label_value))?;

        // Extract the three rows
        let header_row = label_array
            .get(0)
            .and_then(|v| v.as_array())
            .ok_or_else(|| format!("Label {}: Invalid header row structure", i))?;

        let qr_row = label_array
            .get(1)
            .and_then(|v| v.as_array())
            .ok_or_else(|| format!("Label {}: Invalid QR row structure", i))?;

        let order_row = label_array
            .get(2)
            .and_then(|v| v.as_array())
            .ok_or_else(|| format!("Label {}: Invalid order row structure", i))?;

        // Convert to LabelData format
        let mut label_data_rows = Vec::new();

        // Header row: [name, address, phone]
        let header_strings: Vec<String> = header_row
            .iter()
            .map(|v| {
                v.as_str()
                    .ok_or_else(|| format!("Label {}: Header field is not a string", i))
                    .map(|s| s.to_string())
            })
            .collect::<Result<Vec<_>, _>>()?;
        label_data_rows.push(header_strings);

        // QR row: [items_text, brand_array] -> [items_text, brand_json_string]
        let items_text = qr_row
            .get(0)
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("Label {}: Invalid items text in QR row", i))?
            .to_string();

        let brand_array = qr_row
            .get(1)
            .and_then(|v| v.as_array())
            .ok_or_else(|| format!("Label {}: Invalid brand array in QR row", i))?;

        let brand_strings: Vec<String> = brand_array
            .iter()
            .map(|v| {
                v.as_str()
                    .ok_or_else(|| format!("Label {}: Brand field is not a string", i))
                    .map(|s| s.to_string())
            })
            .collect::<Result<Vec<_>, _>>()?;

        let brand_json = serde_json::to_string(&brand_strings)
            .map_err(|e| format!("Label {}: Failed to serialize brand data: {}", i, e))?;
        label_data_rows.push(vec![items_text, brand_json]);

        // Order row: [order_id, date]
        let order_strings: Vec<String> = order_row
            .iter()
            .map(|v| {
                v.as_str()
                    .ok_or_else(|| format!("Label {}: Order field is not a string", i))
                    .map(|s| s.to_string())
            })
            .collect::<Result<Vec<_>, _>>()?;
        label_data_rows.push(order_strings);

        let label_data = LabelData::new(label_data_rows);
        all_labels.push(label_data);

        let header_name = header_row
            .get(0)
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");

        let order_id = order_row
            .get(0)
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");

        println!("   Label #{}: {} - {}", i + 1, header_name, order_id);
    }

    // Create renderer with default settings (2 labels per page)
    println!("🎨 Creating LabelRenderer with default configuration...");
    let font_manager =
        FontManager::new().map_err(|e| format!("Failed to create font manager: {}", e))?;

    let mut renderer = LabelRenderer::with_defaults(font_manager)
        .map_err(|e| format!("Failed to create label renderer: {}", e))?;

    // Enable debug mode to see detailed rendering information
    let mut config = renderer.config().clone();
    config.debug = true;
    renderer.set_config(config);

    println!("📊 Configuration:");
    println!(
        "   Page size: {}mm x {}mm",
        renderer.config().page_width,
        renderer.config().page_height
    );
    println!(
        "   Table size: {}mm x {}mm",
        renderer.config().table_width,
        renderer.config().table_height
    );
    println!("   Labels per page: {}", renderer.max_labels_per_page());
    println!(
        "   Font sizes: Regular {}pt, Brand {}pt",
        renderer.config().font_size,
        renderer.config().brand_font_size
    );

    // Render all labels
    println!("🏭 Rendering {} shipping labels...", all_labels.len());
    for (i, label) in all_labels.iter().enumerate() {
        renderer
            .render_label(label)
            .map_err(|e| format!("Failed to render label {}: {}", i + 1, e))?;

        if (i + 1) % 2 == 0 {
            println!("   Processed labels {}-{}", i, i + 1);
        }
    }

    // Store values before consuming renderer
    let total_labels = all_labels.len();
    let labels_per_page = renderer.max_labels_per_page();

    // Finish rendering and get PDF data
    println!("📄 Finalizing PDF generation...");
    let pdf_data = renderer
        .finish()
        .map_err(|e| format!("Failed to finalize PDF: {}", e))?;

    // Save the PDF
    if let Some(parent) = file_config.output_path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            format!(
                "Failed to create output directory {}: {}",
                parent.display(),
                e
            )
        })?;
    }
    fs::write(&file_config.output_path, &pdf_data).map_err(|e| {
        format!(
            "Failed to write PDF to {}: {}",
            file_config.output_path.display(),
            e
        )
    })?;

    println!("✅ SUCCESS! Shipping label PDF generated");
    println!("📁 Output file: {}", file_config.output_path.display());
    println!("📊 Statistics:");
    println!("   Total labels: {}", total_labels);
    println!(
        "   Total pages: {}",
        (total_labels + labels_per_page - 1) / labels_per_page
    );
    println!("   PDF size: {} bytes", pdf_data.len());
    println!("   Labels per page: {}", labels_per_page);

    // Verify PDF structure
    let pdf_str = String::from_utf8_lossy(&pdf_data);
    if pdf_str.contains("%PDF") && pdf_str.contains("%%EOF") {
        println!("✅ PDF structure validation: PASSED");
    } else {
        println!("❌ PDF structure validation: FAILED");
    }

    println!();
    println!("🎉 ShipLabel Rust Implementation - COMPLETE!");
    println!("==========================================");
    println!("Features demonstrated:");
    println!(
        "✅ Multi-page PDF generation ({} pages)",
        (total_labels + labels_per_page - 1) / labels_per_page
    );
    println!("✅ SVG QR code integration");
    println!("✅ Dynamic font weight selection (Regular/Bold/Brand)");
    println!("✅ Dynamic font sizing");
    println!("✅ Cut guidelines between labels");
    println!("✅ Type-safe configuration system");
    println!("✅ Embedded Google Fonts (Roboto + Merriweather)");
    println!("✅ Unicode text support");
    println!("✅ Production-ready error handling");
    println!();
    println!(
        "📄 Open '{}' to view the generated shipping labels!",
        file_config.output_path.display()
    );

    Ok(())
}
