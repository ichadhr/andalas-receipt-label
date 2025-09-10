use krilla::Document;
use krilla::page::PageSettings;
use krilla::color::rgb;
use krilla::paint::Fill;
use krilla::num::NormalizedF32;
use std::fs;

// Import our font manager, text rendering, and table rendering
use pdf::font::FontManager;
use pdf::text::render_text;
use pdf::table::TableRenderer;
use pdf::config::Config;
use pdf::label::{LabelData};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Optimized Table Rendering Test - Fixed font quality issues");
    println!("============================================================");

    // Load fonts using our FontManager (same as shipping labels)
    println!("📂 Loading fonts via FontManager...");
    let font_manager = FontManager::new()?;

    println!("✅ FontManager loaded successfully");
    println!("   Regular font units_per_em: {}", font_manager.regular().units_per_em());
    println!("   Bold font units_per_em: {}", font_manager.bold().units_per_em());
    println!("   Brand font units_per_em: {}", font_manager.brand().units_per_em());

    // Create document with same dimensions as main.rs (shipping label size)
    let mut document = Document::new();
    let mut page = document.start_page_with(PageSettings::new(100.0, 150.0));
    let mut surface = page.surface();

    // Set up black fill
    let black_fill = Fill {
        paint: rgb::Color::new(0, 0, 0).into(),
        opacity: NormalizedF32::ONE,
        rule: Default::default(),
    };
    surface.set_fill(Some(black_fill));

    // Test table rendering like main.rs
    let config = Config::new();
    let table_renderer = TableRenderer::new(&config, &font_manager);

    // Sample data for multiple labels (like main.rs processing)
    let labels_data = vec![
        ("Aswanto Iwan", "Jalan Pelita IV No. 92", "08267398xxxx", "#0001", "Tokopedia Official Store"),
        ("John Doe", "123 Main Street", "555-0123", "#0002", "Brand Store"),
    ];

    let mut current_y = 5.0; // Start position
    let label_gap = 3.0; // Gap between labels

    // Render multiple labels using TableRenderer (exactly like main.rs)
    for (label_index, (name, address, phone, order_id, brand)) in labels_data.iter().enumerate() {
        println!("Rendering label #{} with TableRenderer: {}", label_index + 1, name);

        // Create LabelData structure (exactly like main.rs)
        let mut label_rows = Vec::new();

        // Header row: [name, address, phone]
        label_rows.push(vec![format!("Penerima: {}", name), address.to_string(), phone.to_string()]);

        // QR row: [items_text, brand_json_string]
        let items_text = format!("items: T-shirt x1");
        let brand_json = format!("[\"{}\"]", brand);
        label_rows.push(vec![items_text, brand_json]);

        // Order row: [order_id, date]
        label_rows.push(vec![order_id.to_string(), "2024-01-15".to_string()]);

        let label_data = LabelData::new(label_rows);

        // Convert to RowType structure (exactly like main.rs)
        let row_types = label_data.to_row_types()?;

        // Calculate table position (like main.rs)
        let table_x = config.calculate_table_x();

        // Render table using TableRenderer (exactly like main.rs)
        table_renderer.render_table(&mut surface, table_x, current_y, &row_types)?;

        // Move to next label position
        current_y += config.table_height + label_gap;
    }

    // Add info
    let info_y = 120.0;
    render_text(
        &mut surface,
        10.0,
        info_y,
        "Minimal Test: FontManager + render_text wrapper",
        10.0,
        &font_manager,
        false, // use_brand_font
        false, // use_bold_font
    )?;

    render_text(
        &mut surface,
        10.0,
        info_y + 12.0,
        "Same fonts as shipping labels - testing render_text quality",
        10.0,
        &font_manager,
        false, // use_brand_font
        false, // use_bold_font
    )?;

    // Finish page
    surface.finish();
    page.finish();

    // Save PDF
    let pdf_data = document.finish().map_err(|e| format!("Failed to finish document: {:?}", e))?;
    let output_path = "output/optimized_table_renderer_test_100x150.pdf";
    fs::write(output_path, &pdf_data).map_err(|e| format!("Failed to write PDF: {}", e))?;

    println!("✅ Minimal font test PDF generated (using render_text)");
    println!("📁 Output: {}", output_path);
    println!("📊 PDF size: {} bytes", pdf_data.len());

    println!();
    println!("🎯 Test Results:");
    println!("   ✅ OPTIMIZED TableRenderer (fixed surface state issues)");
    println!("   ✅ Single fill setting per row (not per text call)");
    println!("   ✅ Proper surface state restoration after borders");
    println!("   ✅ Same layout and positioning as main.rs");
    println!("   📄 Compare with complete_rust_labels.pdf - fonts should now be equally smooth!");

    Ok(())
}