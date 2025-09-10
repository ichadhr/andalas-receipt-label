use krilla::geom::{Size, Transform};
use krilla::surface::Surface;
use krilla_svg::{SurfaceExt, SvgSettings};
use qrcode::QrCode;
use qrcode::render::svg;

/// Generate a QR code as SVG string
///
/// # Arguments
/// * `content` - The text content to encode in the QR code
///
/// # Returns
/// * `Result<String, Box<dyn std::error::Error>>` - SVG string or error
///
/// # Security
/// Input content is limited to 1024 bytes to prevent excessive memory usage
/// and potential DoS attacks through extremely large QR codes.
///
/// # Examples
/// ```
/// use pdf::generate_qr_svg;
///
/// let qr_svg = generate_qr_svg("Hello, World!")?;
/// assert!(qr_svg.contains("<svg"));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn generate_qr_svg(content: &str) -> Result<String, Box<dyn std::error::Error>> {
    const MAX_QR_CONTENT_SIZE: usize = 1024;

    if content.len() > MAX_QR_CONTENT_SIZE {
        return Err(format!(
            "QR content too large: {} bytes (max: {} bytes)",
            content.len(),
            MAX_QR_CONTENT_SIZE
        )
        .into());
    }

    let code = QrCode::new(content.as_bytes())?;
    let svg_xml = code.render::<svg::Color>().build();
    Ok(svg_xml)
}

/// Embed a QR code SVG into a krilla surface
///
/// # Arguments
/// * `surface` - The krilla surface to draw on
/// * `svg_content` - The SVG string content of the QR code
/// * `x` - X coordinate for the top-left corner
/// * `y` - Y coordinate for the top-left corner
/// * `size` - Size of the QR code (width and height)
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - Success or error
pub fn embed_qr_svg(
    surface: &mut Surface,
    svg_content: &str,
    x: f32,
    y: f32,
    size: f32,
) -> Result<(), Box<dyn std::error::Error>> {
    // Parse SVG string into usvg Tree (following krilla-svg example pattern)
    let tree = {
        let opts = usvg::Options::default();
        let data = svg_content.as_bytes();
        usvg::Tree::from_data(data, &opts)?
    };

    // Create krilla Size
    let qr_size = Size::from_wh(size, size).ok_or("Invalid size")?;

    // Create SVG settings
    let svg_settings = SvgSettings::default();

    // Create translation transform and apply it
    let translate_transform = Transform::from_translate(x, y);
    surface.push_transform(&translate_transform);

    // Draw the SVG
    surface
        .draw_svg(&tree, qr_size, svg_settings)
        .ok_or("Failed to draw SVG")?;

    // Restore transform
    surface.pop();

    Ok(())
}

/// Generate and embed QR code in one step
///
/// # Arguments
/// * `surface` - The krilla surface to draw on
/// * `content` - The text content to encode in the QR code
/// * `x` - X coordinate for the top-left corner
/// * `y` - Y coordinate for the top-left corner
/// * `size` - Size of the QR code (width and height)
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - Success or error
pub fn generate_and_embed_qr(
    surface: &mut Surface,
    content: &str,
    x: f32,
    y: f32,
    size: f32,
) -> Result<(), Box<dyn std::error::Error>> {
    let svg_content = generate_qr_svg(content)?;
    embed_qr_svg(surface, &svg_content, x, y, size)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_qr_svg() {
        let content = "Hello, World!";
        let result = generate_qr_svg(content);
        assert!(result.is_ok());

        let svg = result.unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        // QR codes encode data as geometric patterns, not as readable text
        assert!(svg.contains("rect") || svg.contains("path"));
    }

    #[test]
    fn test_generate_qr_svg_empty_content() {
        let content = "";
        let result = generate_qr_svg(content);
        assert!(result.is_ok());
    }

    #[test]
    fn test_generate_qr_svg_special_characters() {
        let content = "Special chars: àáâãäåæçèéêë";
        let result = generate_qr_svg(content);
        assert!(result.is_ok());
    }

    #[test]
    fn test_generate_qr_svg_content_size_limit() {
        // Test content exactly at the limit
        let content_1024 = "a".repeat(1024);
        let result = generate_qr_svg(&content_1024);
        assert!(result.is_ok());

        // Test content over the limit
        let content_1025 = "a".repeat(1025);
        let result = generate_qr_svg(&content_1025);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("QR content too large"));
    }
}
