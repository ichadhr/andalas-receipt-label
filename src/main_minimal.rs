use krilla::Document;
use krilla::page::PageSettings;
use krilla::geom::Point;
use krilla::color::rgb;
use krilla::paint::Fill;
use krilla::num::NormalizedF32;
use std::fs;

// Import our font manager
use pdf::font::FontManager;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Minimal Font Test - Using FontManager + Krilla Internal Rendering");
    println!("================================================================");

    // Load fonts using our FontManager (same as shipping labels)
    println!("📂 Loading fonts via FontManager...");
    let font_manager = FontManager::new()?;

    println!("✅ FontManager loaded successfully");
    println!("   Regular font units_per_em: {}", font_manager.regular().units_per_em());
    println!("   Bold font units_per_em: {}", font_manager.bold().units_per_em());
    println!("   Brand font units_per_em: {}", font_manager.brand().units_per_em());

    // Create document with shipping label dimensions
    let mut document = Document::new();
    let mut page = document.start_page_with(PageSettings::new(200.0, 300.0));
    let mut surface = page.surface();

    // Set up black fill
    let black_fill = Fill {
        paint: rgb::Color::new(0, 0, 0).into(),
        opacity: NormalizedF32::ONE,
        rule: Default::default(),
    };
    surface.set_fill(Some(black_fill));

    // Test shipping label content with minimal rendering
    let small_font = 6.0;  // Same as shipping labels
    let brand_font = 8.0;

    // Sample shipping label content
    let recipient_name = "Aswanto Iwan";
    let address = "Jalan Pelita IV No. 92, RT 08 RW 06";
    let phone = "08267398xxxx";
    let order_id = "#0001";
    let brand_text = "Tokopedia Official Store";

    let mut y_pos = 30.0;

    // Header section - like shipping labels
    surface.draw_text(
        Point::from_xy(10.0, y_pos),
        font_manager.bold().clone(),
        small_font,
        "Penerima:",
        false,
        krilla::text::TextDirection::Auto,
    );

    y_pos += 12.0;
    surface.draw_text(
        Point::from_xy(10.0, y_pos),
        font_manager.bold().clone(),
        small_font,
        recipient_name,
        false,
        krilla::text::TextDirection::Auto,
    );

    y_pos += 12.0;
    surface.draw_text(
        Point::from_xy(10.0, y_pos),
        font_manager.regular().clone(),
        small_font,
        address,
        false,
        krilla::text::TextDirection::Auto,
    );

    y_pos += 12.0;
    surface.draw_text(
        Point::from_xy(10.0, y_pos),
        font_manager.regular().clone(),
        small_font,
        phone,
        false,
        krilla::text::TextDirection::Auto,
    );

    // Brand section
    y_pos += 20.0;
    surface.draw_text(
        Point::from_xy(10.0, y_pos),
        font_manager.brand().clone(),
        brand_font,
        brand_text,
        false,
        krilla::text::TextDirection::Auto,
    );

    // Order info
    y_pos += 20.0;
    surface.draw_text(
        Point::from_xy(10.0, y_pos),
        font_manager.bold().clone(),
        small_font,
        &format!("Order ID: {}", order_id),
        false,
        krilla::text::TextDirection::Auto,
    );

    // Test shipping label exact replica
    y_pos += 30.0;
    surface.draw_text(
        Point::from_xy(10.0, y_pos),
        font_manager.regular().clone(),
        6.0,
        "Penerima:",
        false,
        krilla::text::TextDirection::Auto,
    );

    y_pos += 12.0;
    surface.draw_text(
        Point::from_xy(10.0, y_pos),
        font_manager.regular().clone(),
        6.0,
        "Aswanto Iwan",
        false,
        krilla::text::TextDirection::Auto,
    );

    y_pos += 12.0;
    surface.draw_text(
        Point::from_xy(10.0, y_pos),
        font_manager.regular().clone(),
        6.0,
        "Jalan Pelita IV No. 92",
        false,
        krilla::text::TextDirection::Auto,
    );

    y_pos += 12.0;
    surface.draw_text(
        Point::from_xy(10.0, y_pos),
        font_manager.regular().clone(),
        6.0,
        "08267398xxxx",
        false,
        krilla::text::TextDirection::Auto,
    );

    y_pos += 20.0;
    surface.draw_text(
        Point::from_xy(10.0, y_pos),
        font_manager.brand().clone(),
        8.0,
        "Tokopedia Official Store",
        false,
        krilla::text::TextDirection::Auto,
    );

    y_pos += 20.0;
    surface.draw_text(
        Point::from_xy(10.0, y_pos),
        font_manager.regular().clone(),
        6.0,
        "Order ID: #0001",
        false,
        krilla::text::TextDirection::Auto,
    );

    // Add info
    let info_y = 250.0;
    surface.draw_text(
        Point::from_xy(10.0, info_y),
        font_manager.regular().clone(),
        10.0,
        "Minimal Test: FontManager + Krilla internal rendering",
        false,
        krilla::text::TextDirection::Auto,
    );

    surface.draw_text(
        Point::from_xy(10.0, info_y + 12.0),
        font_manager.regular().clone(),
        10.0,
        "Same fonts as shipping labels - pure Krilla typography",
        false,
        krilla::text::TextDirection::Auto,
    );

    // Finish page
    surface.finish();
    page.finish();

    // Save PDF
    let pdf_data = document.finish().map_err(|e| format!("Failed to finish document: {:?}", e))?;
    let output_path = "output/minimal_font_test.pdf";
    fs::write(output_path, &pdf_data).map_err(|e| format!("Failed to write PDF: {}", e))?;

    println!("✅ Minimal font test PDF generated");
    println!("📁 Output: {}", output_path);
    println!("📊 PDF size: {} bytes", pdf_data.len());

    println!();
    println!("🎯 Test Results:");
    println!("   ✅ Krilla internal rendering only");
    println!("   ✅ Same fonts as shipping labels");
    println!("   ✅ No custom measurement logic");
    println!("   📄 Open the PDF to see pure Krilla typography");

    Ok(())
}