use crate::state::SessionState;

/// Returns the tray icon Image for the given session state.
///
/// Loads pre-rendered PNG icons from the icons/ directory.
/// Falls back to a programmatically generated circle icon if files are missing.
pub fn icon_for_state(state: &SessionState) -> Result<tauri::image::Image<'static>, Box<dyn std::error::Error>> {
    let name = match state {
        SessionState::Idle => "idle",
        SessionState::Working => "working",
        SessionState::NeedsInput => "needs-input",
        SessionState::Error => "error",
    };

    // Try loading pre-rendered PNG icons at various sizes
    for size in &["32", "16", "128", "256"] {
        let png_path = format!("icons/{name}-{size}.png");
        if let Ok(data) = std::fs::read(&png_path) {
            // Parse PNG to get dimensions
            if let Ok((w, h)) = parse_png_dimensions(&data) {
                let img = tauri::image::Image::new_owned(data, w, h);
                return Ok(img);
            }
        }
    }

    // Fallback: generate a simple colored circle at runtime
    generate_fallback_icon(state)
}

/// Generate a minimal fallback icon as RGBA pixels.
fn generate_fallback_icon(state: &SessionState) -> Result<tauri::image::Image<'static>, Box<dyn std::error::Error>> {
    let (r, g, b) = match state {
        SessionState::Idle => (0x22, 0xC5, 0x5E),     // Green
        SessionState::Working => (0xEA, 0xB3, 0x08),   // Yellow
        SessionState::NeedsInput => (0xEF, 0x44, 0x44), // Red
        SessionState::Error => (0xEF, 0x44, 0x44),     // Red
    };

    let size: u32 = 32;
    let center = (size as f32) / 2.0;
    let radius = (size as f32) / 2.0 - 1.5;

    let mut rgba = Vec::with_capacity((size * size * 4) as usize);

    for y in 0..size {
        for x in 0..size {
            let dx = (x as f32) - center;
            let dy = (y as f32) - center;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist <= radius {
                rgba.push(r);
                rgba.push(g);
                rgba.push(b);
                rgba.push(255);
            } else {
                rgba.push(0);
                rgba.push(0);
                rgba.push(0);
                rgba.push(0);
            }
        }
    }

    Ok(tauri::image::Image::new_owned(rgba, size, size))
}

/// Parse PNG dimensions from the IHDR chunk.
/// PNG header: 8 bytes signature + 4 bytes length + 4 bytes "IHDR" + 4 bytes width + 4 bytes height
fn parse_png_dimensions(data: &[u8]) -> Result<(u32, u32), Box<dyn std::error::Error>> {
    if data.len() < 24 {
        return Err("PNG data too short".into());
    }
    // Skip 8-byte signature, then 4-byte length, then "IHDR" (4 bytes)
    // Then width (4 bytes big-endian) and height (4 bytes big-endian)
    let width = u32::from_be_bytes([data[16], data[17], data[18], data[19]]);
    let height = u32::from_be_bytes([data[20], data[21], data[22], data[23]]);
    Ok((width, height))
}