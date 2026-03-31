//! ShipLabel Caching Demonstration
//!
//! This example demonstrates the different caching strategies available in ShipLabel.

use andalas_receipt_label::*;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 ShipLabel Caching Demonstration");
    println!("==================================");

    // Example 1: Default caching (automatic optimization)
    println!("\n📚 Example 1: Default Library Caching");
    println!("-----------------------------------");
    demo_default_caching()?;

    // Example 2: Advanced caching with custom settings
    println!("\n⚡ Example 2: Advanced Caching");
    println!("----------------------------");
    demo_advanced_caching()?;

    // Example 3: No caching (minimal memory usage)
    println!("\n🚫 Example 3: No Caching");
    println!("-----------------------");
    demo_no_caching()?;

    println!("\n✅ Caching demonstration completed!");
    Ok(())
}

fn demo_default_caching() -> ShipLabelResult<()> {
    // Create ShipLabel with default caching - no configuration needed!
    let shiplabel = ShipLabel::new()?;

    // The library automatically uses optimized caching
    println!("✅ Created ShipLabel with automatic caching");
    println!("   Cache strategy: {:?}", shiplabel.config().caching);

    // Create a renderer to generate labels
    let font_manager = FontManager::new()?;
    let mut renderer = LabelRenderer::with_defaults(font_manager)?;

    // Generate some sample labels to demonstrate caching in action
    let labels = create_sample_labels(3);
    let start = Instant::now();

    // Render labels using the renderer
    for label in &labels {
        renderer.render_label(label)?;
    }
    let pdf_data = renderer.finish()?;

    let duration = start.elapsed();

    println!("   Generated {} labels in {:?}", labels.len(), duration);
    println!("   PDF size: {} bytes", pdf_data.len());

    Ok(())
}

fn demo_advanced_caching() -> ShipLabelResult<()> {
    // Configure advanced caching
    let shiplabel = ShipLabel::with_cache_config(CacheConfig::high_performance())?;

    println!("✅ Created ShipLabel with advanced caching");
    println!("   Cache strategy: {:?}", shiplabel.config().caching);

    // Show cache statistics
    let stats = shiplabel.cache_manager().stats();
    println!("   Initial cache state:");
    println!(
        "     Text measurements cached: {}",
        stats.text_measurement_cache_size
    );
    println!("     Font cache loaded: {}", stats.font_cache_loaded);

    // Create renderer and generate labels to populate cache
    let font_manager = FontManager::new()?;
    let mut renderer = LabelRenderer::with_defaults(font_manager)?;
    let labels = create_sample_labels(5);

    for label in &labels {
        renderer.render_label(label)?;
    }
    let pdf_data = renderer.finish()?;

    // Show updated statistics
    let updated_stats = shiplabel.cache_manager().stats();
    println!("   After processing:");
    println!(
        "     Text measurements cached: {}",
        updated_stats.text_measurement_cache_size
    );

    println!("   PDF size: {} bytes", pdf_data.len());

    Ok(())
}

fn demo_no_caching() -> ShipLabelResult<()> {
    // Create ShipLabel without any caching
    let shiplabel = ShipLabel::without_caching()?;

    println!("✅ Created ShipLabel without caching");
    println!("   Cache strategy: {:?}", shiplabel.config().caching);

    // Create renderer and generate labels - this will be slower but use minimal memory
    let font_manager = FontManager::new()?;
    let mut renderer = LabelRenderer::with_defaults(font_manager)?;
    let labels = create_sample_labels(2);
    let start = Instant::now();

    for label in &labels {
        renderer.render_label(label)?;
    }
    let pdf_data = renderer.finish()?;

    let duration = start.elapsed();

    println!("   Generated {} labels in {:?}", labels.len(), duration);
    println!("   PDF size: {} bytes", pdf_data.len());
    println!("   Memory usage: Minimal (no cache overhead)");

    Ok(())
}

fn create_sample_labels(count: usize) -> Vec<LabelData> {
    let mut labels = Vec::new();

    for i in 0..count {
        let rows = vec![
            vec![
                format!("Recipient {}", i + 1),
                format!("Address Line {}", i + 1),
                format!("Phone {}", i + 1),
            ],
            vec![
                format!("Items: Product {}, Product {}", i + 1, i + 2),
                format!("[\"Brand {}\", \"Website {}\"]", i + 1, i + 1),
            ],
            vec![format!("#00{}", i + 1), format!("2024-01-{:02}", i + 1)],
        ];

        labels.push(LabelData::new(rows));
    }

    labels
}
