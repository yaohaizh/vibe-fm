"""
Generate Vibe File Manager icon - Modern Purple/Violet Gradient Style
Creates a sleek, modern icon with glassmorphism aesthetics
"""

from PIL import Image, ImageDraw, ImageFilter
import math


def lerp_color(c1, c2, t):
    """Linear interpolation between two colors"""
    return tuple(int(c1[i] + (c2[i] - c1[i]) * t) for i in range(len(c1)))


def create_gradient_image(width, height, color1, color2, direction="diagonal"):
    """Create a gradient image"""
    img = Image.new("RGBA", (width, height), (0, 0, 0, 0))

    for y in range(height):
        for x in range(width):
            if direction == "diagonal":
                t = (x + y) / (width + height)
            elif direction == "vertical":
                t = y / height
            elif direction == "horizontal":
                t = x / width
            else:
                t = (x + y) / (width + height)

            color = lerp_color(color1, color2, t)
            img.putpixel((x, y), color)

    return img


def draw_rounded_rect_aa(img, x1, y1, x2, y2, radius, fill_color):
    """Draw anti-aliased rounded rectangle with better quality"""
    # Create at 2x resolution for anti-aliasing
    scale = 2
    w = int((x2 - x1) * scale)
    h = int((y2 - y1) * scale)
    r = int(radius * scale)

    if w <= 0 or h <= 0:
        return

    temp = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    draw = ImageDraw.Draw(temp)

    # Draw rounded rectangle
    draw.rounded_rectangle([0, 0, w - 1, h - 1], radius=r, fill=fill_color)

    # Scale down with anti-aliasing
    try:
        temp = temp.resize((int(x2 - x1), int(y2 - y1)), Image.Resampling.LANCZOS)
    except AttributeError:
        temp = temp.resize((int(x2 - x1), int(y2 - y1)), Image.LANCZOS)

    # Paste onto main image
    img.paste(temp, (int(x1), int(y1)), temp)


def create_modern_icon():
    """Create a modern, gradient-based dual-pane icon"""
    sizes = [16, 24, 32, 48, 64, 128, 256]
    images = []

    # Modern purple/violet color palette
    bg_gradient_start = (99, 49, 163, 255)  # Deep purple
    bg_gradient_end = (168, 85, 247, 255)  # Vibrant violet

    panel_left_color = (255, 255, 255, 50)  # Glass effect - white with low opacity
    panel_right_color = (255, 255, 255, 35)  # Slightly more transparent

    panel_left_accent = (236, 72, 153, 255)  # Pink accent
    panel_right_accent = (139, 92, 246, 255)  # Purple accent

    line_color_left = (255, 255, 255, 100)  # Subtle white lines
    line_color_right = (255, 255, 255, 80)

    for size in sizes:
        # Create gradient background
        img = create_gradient_image(
            size, size, bg_gradient_start, bg_gradient_end, "diagonal"
        )

        # Calculate proportions
        padding = max(1, size // 12)
        corner_radius = max(2, size // 6)

        # Create a mask for rounded corners on main background
        mask = Image.new("L", (size, size), 0)
        mask_draw = ImageDraw.Draw(mask)
        mask_draw.rounded_rectangle(
            [padding, padding, size - padding, size - padding],
            radius=corner_radius,
            fill=255,
        )

        # Apply rounded corner mask to gradient
        bg_with_corners = Image.new("RGBA", (size, size), (0, 0, 0, 0))
        bg_with_corners.paste(img, (0, 0), mask)
        img = bg_with_corners

        # Add subtle inner glow/border
        draw = ImageDraw.Draw(img)

        # Inner content area
        inner_pad = max(2, size // 6)
        panel_gap = max(1, size // 14)

        panel_area_left = padding + inner_pad
        panel_area_top = padding + inner_pad
        panel_area_right = size - padding - inner_pad
        panel_area_bottom = size - padding - inner_pad

        panel_width = (panel_area_right - panel_area_left - panel_gap) // 2
        panel_height = panel_area_bottom - panel_area_top

        panel_radius = max(1, size // 12)

        # Left panel (glass effect)
        left_x1 = panel_area_left
        left_y1 = panel_area_top
        left_x2 = panel_area_left + panel_width
        left_y2 = panel_area_bottom

        draw_rounded_rect_aa(
            img, left_x1, left_y1, left_x2, left_y2, panel_radius, panel_left_color
        )

        # Right panel (glass effect)
        right_x1 = left_x2 + panel_gap
        right_y1 = panel_area_top
        right_x2 = panel_area_right
        right_y2 = panel_area_bottom

        draw_rounded_rect_aa(
            img, right_x1, right_y1, right_x2, right_y2, panel_radius, panel_right_color
        )

        # Add accent bars at top of panels (for sizes >= 24)
        if size >= 24:
            accent_height = max(1, size // 16)
            accent_margin = max(1, size // 24)

            # Left panel accent bar (pink)
            draw_rounded_rect_aa(
                img,
                left_x1 + accent_margin,
                left_y1 + accent_margin,
                left_x2 - accent_margin,
                left_y1 + accent_margin + accent_height,
                max(1, accent_height // 2),
                panel_left_accent,
            )

            # Right panel accent bar (purple)
            draw_rounded_rect_aa(
                img,
                right_x1 + accent_margin,
                right_y1 + accent_margin,
                right_x2 - accent_margin,
                right_y1 + accent_margin + accent_height,
                max(1, accent_height // 2),
                panel_right_accent,
            )

        # Add file lines (for sizes >= 32)
        if size >= 32:
            line_height = max(1, size // 32)
            line_spacing = max(2, size // 16)
            line_margin_x = max(2, size // 20)
            line_start_y = left_y1 + max(4, size // 8)

            # Redraw to add lines
            draw = ImageDraw.Draw(img)

            # Lines in left panel
            y = line_start_y
            line_count = 0
            max_lines = 4
            while y + line_height < left_y2 - line_margin_x and line_count < max_lines:
                # Vary line widths for visual interest
                line_width_factor = 0.9 if line_count % 2 == 0 else 0.6
                line_end_x = (
                    left_x1
                    + line_margin_x
                    + int((left_x2 - left_x1 - 2 * line_margin_x) * line_width_factor)
                )

                draw.rounded_rectangle(
                    [left_x1 + line_margin_x, y, line_end_x, y + line_height],
                    radius=max(1, line_height // 2),
                    fill=line_color_left,
                )
                y += line_spacing + line_height
                line_count += 1

            # Lines in right panel
            y = line_start_y
            line_count = 0
            while y + line_height < right_y2 - line_margin_x and line_count < max_lines:
                line_width_factor = 0.75 if line_count % 2 == 0 else 0.5
                line_end_x = (
                    right_x1
                    + line_margin_x
                    + int((right_x2 - right_x1 - 2 * line_margin_x) * line_width_factor)
                )

                draw.rounded_rectangle(
                    [right_x1 + line_margin_x, y, line_end_x, y + line_height],
                    radius=max(1, line_height // 2),
                    fill=line_color_right,
                )
                y += line_spacing + line_height
                line_count += 1

        # Add subtle highlight on top edge for depth (sizes >= 48)
        if size >= 48:
            highlight_color = (255, 255, 255, 30)
            highlight_height = max(1, size // 32)
            draw.rounded_rectangle(
                [
                    padding + 2,
                    padding + 1,
                    size - padding - 2,
                    padding + highlight_height + 1,
                ],
                radius=max(1, corner_radius // 2),
                fill=highlight_color,
            )

        images.append(img)

    # Save as ICO file with multiple sizes
    images[0].save(
        "assets/vibe.ico",
        format="ICO",
        sizes=[(s, s) for s in sizes],
        append_images=images[1:],
    )

    # Also save PNG versions
    images[-1].save("assets/vibe_256.png", format="PNG")
    images[4].save("assets/vibe_64.png", format="PNG")  # 64px for previews

    print(f"Created assets/vibe.ico with sizes: {sizes}")
    print(f"Created assets/vibe_256.png (256x256)")
    print(f"Created assets/vibe_64.png (64x64)")
    print("\nColor scheme: Purple/Violet gradient with glassmorphism panels")


if __name__ == "__main__":
    import os

    os.makedirs("assets", exist_ok=True)
    create_modern_icon()
