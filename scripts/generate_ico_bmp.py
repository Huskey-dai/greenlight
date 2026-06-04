"""
Generate a proper Windows ICO file with embedded BMP icons.
Creates both 32x32 and 256x256 sizes in BMP format (not PNG).
"""

import struct
import zlib

def make_circle_pixels(size, r, g, b):
    """Generate BGRA pixel array for a colored circle."""
    pixels = bytearray()
    cx, cy = size / 2.0, size / 2.0
    radius = size / 2.0 - max(1, size * 0.06)

    for y in range(size):
        for x in range(size):
            dx = x - cx + 0.5
            dy = y - cy + 0.5
            dist = (dx * dx + dy * dy) ** 0.5

            if dist <= radius:
                # BMG format: BGRA
                pixels.extend([b, g, r, 255])
            elif dist <= radius + 1.0:
                # Anti-aliased edge
                aa = max(0, 1.0 - (dist - radius))
                pixels.extend([b, g, r, int(255 * aa)])
            else:
                pixels.extend([0, 0, 0, 0])
    return pixels


def create_bmp_icon_data(size, r, g, b):
    """Create BMP image data for ICO (BITMAPINFOHEADER format)."""
    pixels = make_circle_pixels(size, r, g, b)

    # BITMAPINFOHEADER
    header = struct.pack('<IiiHHIIiiII',
        40,         # biSize (header size)
        size,       # biWidth
        size * 2,   # biHeight (doubled for AND mask)
        1,          # biPlanes
        32,         # biBitCount (32-bit ARGB)
        0,          # biCompression (BI_RGB)
        0,          # biSizeImage (can be 0 for BI_RGB)
        0,          # biXPelsPerMeter
        0,          # biYPelsPerMeter
        0,          # biClrUsed
        0           # biClrImportant
    )

    # XOR mask (color data) - rows are bottom-up in BMP
    xor_data = bytearray()
    for y in range(size - 1, -1, -1):
        row = pixels[y * size * 4 : (y + 1) * size * 4]
        xor_data.extend(row)

    # AND mask (1-bit transparency mask) - 1 = transparent
    # Since we're using 32-bit with alpha, AND mask is all zeros
    and_row_size = ((size + 31) // 32) * 4  # DWORD aligned
    and_data = bytes(and_row_size * size)

    return header + bytes(xor_data) + and_data


def create_ico(sizes_and_data):
    """Create ICO file from list of (size, bmp_data) pairs."""
    count = len(sizes_and_data)

    # ICO header
    header = struct.pack('<HHH', 0, 1, count)

    # Calculate offsets
    dir_entry_size = 16
    data_offset = 6 +count * dir_entry_size

    directory = b''
    image_data = b''

    for size, bmp_data in sizes_and_data:
        w = size if size < 256 else 0
        h = size if size < 256 else 0
        # Directory entry
        directory += struct.pack('<BBBBHHII',
            w, h,           # Width, height (0 = 256)
            0,               # Color count (0 = no palette)
            0,               # Reserved
            1,               # Color planes
            32,              # Bits per pixel
            len(bmp_data),   # Image data size
            data_offset      # Offset to image data
        )
        image_data += bmp_data
        data_offset += len(bmp_data)

    return header + directory + image_data


def main():
    import os
    icons_dir = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
                               'src-tauri', 'icons')

    # Green color for the app icon
    r, g, b = 0x22, 0xC5, 0x5E

    # Generate ICO with 32x32 and 256x256 sizes
    sizes_data = []
    for size in [32, 256]:
        bmp_data = create_bmp_icon_data(size, r, g, b)
        sizes_data.append((size, bmp_data))
        print(f'  Generated {size}x{size} BMP icon data ({len(bmp_data)} bytes)')

    ico_data = create_ico(sizes_data)
    ico_path = os.path.join(icons_dir, 'icon.ico')
    with open(ico_path, 'wb') as f:
        f.write(ico_data)
    print(f'  Written icon.ico ({len(ico_data)} bytes)')


if __name__ == '__main__':
    main()