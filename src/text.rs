//! Ultra-minimal text rendering for ShipLabel
//!
//! Only essential functions for text positioning.

use crate::error::ShipLabelResult;
use crate::font::FontManager;
use krilla::surface::Surface;

/// Simple text rendering with positioning - matches main_minimal.rs exactly
pub fn render_text(
    surface: &mut Surface,
    x: f32,
    y: f32,
    text: &str,
    font_size: f32,
    font_manager: &FontManager,
    use_brand_font: bool,
    use_bold_font: bool,
) -> ShipLabelResult<()> {
    // Set black fill for text rendering (like main_minimal.rs)
    let black_fill = krilla::paint::Fill {
        paint: krilla::color::rgb::Color::black().into(),
        opacity: krilla::num::NormalizedF32::ONE,
        rule: Default::default(),
    };
    surface.set_fill(Some(black_fill));

    // Choose font - same logic as main_minimal.rs
    let font = if use_brand_font {
        font_manager.brand().clone()
    } else if use_bold_font {
        font_manager.bold().clone()
    } else {
        font_manager.regular().clone()
    };

    // Handle newlines - same as main_minimal.rs
    let lines: Vec<&str> = text.split('\n').collect();
    let line_height = font_size;
    let mut current_y = y;

    for line in lines {
        surface.draw_text(
            krilla::geom::Point::from_xy(x, current_y),
            font.clone(),
            font_size,
            line,
            false,
            krilla::text::TextDirection::Auto,
        );
        current_y += line_height;
    }

    Ok(())
}
