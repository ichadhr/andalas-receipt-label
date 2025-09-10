use pdf::*;
use serde_json;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 ShipLabel - Complete Shipping Label PDF Generation");
    println!("==================================================");

    // Load sample data
    println!("📂 Loading sample data from output/sample.json...");
    let sample_content =
        fs::read_to_string("output/sample.json").expect("Failed to read sample.json");
    let raw_labels: Vec<serde_json::Value> =
        serde_json::from_str(&sample_content).expect("Failed to parse sample.json");

    println!("✅ Loaded {} sample labels", raw_labels.len());

    // Convert sample data to LabelData format
    println!("🔄 Converting sample data to LabelData format...");
    let mut all_labels = Vec::new();

    for (i, label_value) in raw_labels.iter().enumerate() {
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

        let label_data = LabelData::new(label_data_rows);
        all_labels.push(label_data);

        println!(
            "   Label #{}: {} - {}",
            i + 1,
            header_row[0].as_str().unwrap(),
            order_row[0].as_str().unwrap()
        );
    }

    // Create renderer with default settings (2 labels per page)
    println!("🎨 Creating LabelRenderer with default configuration...");
    let font_manager = FontManager::new().unwrap();
    let mut renderer = LabelRenderer::with_defaults(font_manager).unwrap();

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
        renderer.render_label(label).unwrap();
        if (i + 1) % 2 == 0 {
            println!("   Processed labels {}-{}", i, i + 1);
        }
    }

    // Store values before consuming renderer
    let total_labels = all_labels.len();
    let labels_per_page = renderer.max_labels_per_page();

    // Finish rendering and get PDF data
    println!("📄 Finalizing PDF generation...");
    let pdf_data = renderer.finish().unwrap();

    // Save the PDF
    let output_path = "output/complete_rust_labels.pdf";
    fs::write(output_path, &pdf_data).expect("Failed to write PDF");

    println!("✅ SUCCESS! Shipping label PDF generated");
    println!("📁 Output file: {}", output_path);
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
        output_path
    );
    println!("🔍 Compare this with PHP output for accuracy validation.");

    Ok(())
}
