"""
Generate placeholder icons for Greenlight.

Creates:
- App icons (32x32, 128x128, 128x128@2x, icon.ico)
- State-specific tray icons (idle, working, needs-input, error, unknown)
  at 16x16, 32x32, 128x128, 256x256 for HiDPI support
"""
import struct
import zlib
import os

ICON_DIR = os.path.join(os.path.dirname(__file__), "icons")

# Colors for each state (RGB)
STATE_COLORS = {
    "idle":        (0x22, 0xC5, 0x5E),  # Green
    "working":     (0xEA, 0xB3, 0x08),  # Yellow
    "needs-input": (0xEF, 0x44, 0x44),  # Red (blinking)
    "error":       (0xEF, 0x44, 0x44),  # Red (steady)
    "unknown":     (0x6B, 0x72, 0x80),  # Gray
    "app":         (0x22, 0xC5, 0x5E),  # Green for app icon (uses idle color as default)
}

def create_circle_png(size, rgba_color, ring_width=1):
    """Create a PNG image of a colored circle on a transparent background."""
    width = height = size
    center = size / 2.0
    radius = (size / 2.0) - 1.5  # Leave margin for ring
    inner_radius = radius - ring_width

    # Generate RGBA pixel data
    pixels = []
    r, g, b = rgba_color[:3]
    a = rgba_color[3] if len(rgba_color) > 3 else 255

    for y in range(height):
        row_data = b'\x00'  # Filter byte (None)
        for x in range(width):
            dx = x - center + 0.5
            dy = y - center + 0.5
            dist = (dx * dx + dy * dy) ** 0.5

            if dist <= radius:
                # Inside circle
                if dist > inner_radius:
                    # Ring area - slightly darker
                    row_data += bytes([max(0, r - 30), max(0, g - 30), max(0, b - 30), a])
                else:
                    # Fill area - with subtle inner glow (no white highlight)
                    # Inner glow offset for 3D effect
                    glow_dx = dx / radius
                    glow_dy = dy / radius
                    glow_intensity = max(0, 1.0 - (glow_dx * glow_dx + glow_dy * glow_dy) * 0.3)
                    row_data += bytes([
                        min(255, int(r * (0.85 + 0.15 * glow_intensity))),
                        min(255, int(g * (0.85 + 0.15 * glow_intensity))),
                        min(255, int(b * (0.85 + 0.15 * glow_intensity))),
                        a
                    ])
            else:
                # Transparent
                row_data += b'\x00\x00\x00\x00'

        pixels.append(row_data)

    raw_data = b''.join(pixels)
    return create_png(width, height, raw_data, has_alpha=True)


def create_png(width, height, raw_data, has_alpha=False):
    """Create a valid PNG file from raw pixel data."""
    # PNG header
    signature = b'\x89PNG\r\n\x1a\n'

    # IHDR chunk
    bit_depth = 8
    color_type = 6 if has_alpha else 2  # RGBA or RGB
    ihdr_data = struct.pack('>IIBBBBB', width, height, bit_depth, color_type, 0, 0, 0)
    ihdr_crc = zlib.crc32(b'IHDR' + ihdr_data) & 0xFFFFFFFF
    ihdr = struct.pack('>I', 13) + b'IHDR' + ihdr_data + struct.pack('>I', ihdr_crc)

    # IDAT chunk (compressed image data)
    compressed = zlib.compress(raw_data)
    idat_crc = zlib.crc32(b'IDAT' + compressed) & 0xFFFFFFFF
    idat = struct.pack('>I', len(compressed)) + b'IDAT' + compressed + struct.pack('>I', idat_crc)

    # IEND chunk
    iend_crc = zlib.crc32(b'IEND') & 0xFFFFFFFF
    iend = struct.pack('>I', 0) + b'IEND' + struct.pack('>I', iend_crc)

    return signature + ihdr + idat + iend


def create_ico(sizes_and_images):
    """Create an ICO file from multiple PNG images."""
    # ICO header
    count = len(sizes_and_images)
    header = struct.pack('<HHH', 0, 1, count)

    # Calculate offsets
    header_size = 6 + count * 16
    offset = header_size

    directory = b''
    png_data = b''

    for size, png_bytes in sizes_and_images:
        # Directory entry
        w = size if size < 256 else 0
        h = size if size < 256 else 0
        directory += struct.pack('<BBBBHHII', w, h, 0, 0, 1, 32, len(png_bytes), offset)
        png_data += png_bytes
        offset += len(png_bytes)

    return header + directory + png_data


def main():
    os.makedirs(ICON_DIR, exist_ok=True)

    # Generate state-specific icons
    sizes = [16, 32, 128, 256]
    for state_name, color in STATE_COLORS.items():
        if state_name == "app":
            continue
        for size in sizes:
            rgba = color + (255,)
            png_data = create_circle_png(size, rgba)
            filename = f"{state_name}-{size}.png"
            filepath = os.path.join(ICON_DIR, filename)
            with open(filepath, 'wb') as f:
                f.write(png_data)
            print(f"Created {filename}")

    # Generate app icons (using idle color as default)
    app_color = STATE_COLORS["app"] + (255,)
    for size in [32, 128, 256]:
        png_data = create_circle_png(size, app_color)
        if size == 256:
            filename = "128x128@2x.png"
        else:
            filename = f"{size}x{size}.png"
        filepath = os.path.join(ICON_DIR, filename)
        with open(filepath, 'wb') as f:
            f.write(png_data)
        print(f"Created {filename}")

    # Generate icon.png (1024x1024 for app store)
    png_data = create_circle_png(1024, app_color)
    filepath = os.path.join(ICON_DIR, "icon.png")
    with open(filepath, 'wb') as f:
        f.write(png_data)
    print("Created icon.png")

    # Generate icon.ico (Windows)
    ico_sizes = [(32, create_circle_png(32, app_color)),
                  (128, create_circle_png(128, app_color)),
                  (256, create_circle_png(256, app_color))]
    ico_data = create_ico(ico_sizes)
    filepath = os.path.join(ICON_DIR, "icon.ico")
    with open(filepath, 'wb') as f:
        f.write(ico_data)
    print("Created icon.ico")

    # Generate icon.icns placeholder (macOS) - just copy the 256x256 PNG
    # Real icns requires special format, but Tauri can use PNG
    png_256 = create_circle_png(256, app_color)
    filepath = os.path.join(ICON_DIR, "icon.icns")
    # For now, just use the PNG (Tauri handles conversion)
    with open(filepath, 'wb') as f:
        f.write(png_256)
    print("Created icon.icns (PNG placeholder)")

    print("\nAll icons generated!")


if __name__ == "__main__":
    main()