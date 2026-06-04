"""
Generate proper PNG icons for Greenlight using pure Python (no PIL dependency).
Uses the struct+zlib approach to create valid PNG files.
"""
import struct
import zlib
import os
import sys

def create_png(width, height, pixels):
    """Create a PNG from RGBA pixel data. pixels is a list of (r,g,b,a) tuples."""
    def make_chunk(chunk_type, data):
        chunk = chunk_type + data
        crc = zlib.crc32(chunk) & 0xFFFFFFFF
        return struct.pack('>I', len(data)) + chunk + struct.pack('>I', crc)

    # PNG signature
    signature = b'\x89PNG\r\n\x1a\n'

    # IHDR
    ihdr_data = struct.pack('>IIBBBBB', width, height, 8, 6, 0, 0, 0)
    ihdr = make_chunk(b'IHDR', ihdr_data)

    # IDAT - raw pixel data with filter bytes
    raw = bytearray()
    for y in range(height):
        raw.append(0)  # None filter
        for x in range(width):
            idx = (y * width + x)
            r, g, b, a = pixels[idx]
            raw.extend([r, g, b, a])

    compressed = zlib.compress(bytes(raw), 9)
    idat = make_chunk(b'IDAT', compressed)

    # IEND
    iend = make_chunk(b'IEND', b'')

    return signature + ihdr + idat + iend


def make_circle(size, r, g, b, alpha=255):
    """Generate RGBA pixels for a filled circle."""
    pixels = []
    cx, cy = size / 2.0, size / 2.0
    radius = size / 2.0 - max(1, size * 0.06)

    for y in range(size):
        for x in range(size):
            dx = x - cx + 0.5
            dy = y - cy + 0.5
            dist = (dx * dx + dy * dy) ** 0.5

            if dist <= radius:
                pixels.append((r, g, b, alpha))
            elif dist <= radius + 1.0:
                # Anti-alias edge
                aa = max(0, 1.0 - (dist - radius))
                pixels.append((r, g, b, int(alpha * aa)))
            else:
                pixels.append((0, 0, 0, 0))

    return pixels


def make_ico(sizes_data):
    """Create ICO file from (size, png_bytes) pairs."""
    count = len(sizes_data)
    # Header: 6 bytes
    header = struct.pack('<HHH', 0, 1, count)

    # Calculate offsets for each entry
    dir_size = 16 * count
    offset = 6 + dir_size

    entries = b''
    image_data = b''

    for size, png_bytes in sizes_data:
        w = size if size < 256 else 0
        h = size if size < 256 else 0
        # ICO directory entry: width(1), height(1), colors(1), reserved(1), planes(2), bpp(2), size(4), offset(4)
        entries += struct.pack('<BBBBHHII', w, h, 0, 0, 1, 32, len(png_bytes), offset)
        image_data += png_bytes
        offset += len(png_bytes)

    return header + entries + image_data


def main():
    base_dir = os.path.dirname(os.path.abspath(__file__))
    icons_dir = os.path.join(os.path.dirname(base_dir), 'src-tauri', 'icons')

    # State colors
    states = {
        'idle':        (0x22, 0xC5, 0x5E),  # Green
        'working':     (0xEA, 0xB3, 0x08),  # Yellow/amber
        'needs-input': (0xEF, 0x44, 0x44),  # Red
        'error':       (0xEF, 0x44, 0x44),  # Red (same, icon context distinguishes)
    }

    # Generate state icons at each size
    for state_name, (r, g, b) in states.items():
        for size in [16, 32, 128, 256]:
            pixels = make_circle(size, r, g, b)
            png = create_png(size, size, pixels)
            path = os.path.join(icons_dir, f'{state_name}-{size}.png')
            with open(path, 'wb') as f:
                f.write(png)
            print(f'  {state_name}-{size}.png ({len(png)} bytes)')

    # Generate app icons (using idle/green as default)
    r, g, b = 0x22, 0xC5, 0x5E

    # 32x32
    pixels = make_circle(32, r, g, b)
    png = create_png(32, 32, pixels)
    with open(os.path.join(icons_dir, '32x32.png'), 'wb') as f:
        f.write(png)
    print(f'  32x32.png ({len(png)} bytes)')

    # 128x128
    pixels = make_circle(128, r, g, b)
    png = create_png(128, 128, pixels)
    with open(os.path.join(icons_dir, '128x128.png'), 'wb') as f:
        f.write(png)
    print(f'  128x128.png ({len(png)} bytes)')

    # 128x128@2x (256x256)
    pixels = make_circle(256, r, g, b)
    png = create_png(256, 256, pixels)
    with open(os.path.join(icons_dir, '128x128@2x.png'), 'wb') as f:
        f.write(png)
    print(f'  128x128@2x.png ({len(png)} bytes)')

    # icon.png (1024x1024 for app store)
    pixels = make_circle(1024, r, g, b)
    png = create_png(1024, 1024, pixels)
    with open(os.path.join(icons_dir, 'icon.png'), 'wb') as f:
        f.write(png)
    print(f'  icon.png ({len(png)} bytes)')

    # icon.ico (Windows) - contains 32x32 and 256x256
    sizes_ico = []
    for size in [32, 256]:
        pixels = make_circle(size, r, g, b)
        png = create_png(size, size, pixels)
        sizes_ico.append((size, png))

    ico_data = make_ico(sizes_ico)
    with open(os.path.join(icons_dir, 'icon.ico'), 'wb') as f:
        f.write(ico_data)
    print(f'  icon.ico ({len(ico_data)} bytes)')

    # icon.icns (macOS) - placeholder, Tauri converts from PNG
    pixels = make_circle(256, r, g, b)
    png = create_png(256, 256, pixels)
    with open(os.path.join(icons_dir, 'icon.icns'), 'wb') as f:
        f.write(png)
    print(f'  icon.icns ({len(png)} bytes) - placeholder')

    print('\nAll icons generated successfully!')


if __name__ == '__main__':
    main()