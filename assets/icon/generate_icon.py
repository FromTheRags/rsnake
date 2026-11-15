"""Generates the rsnake application icon: a snake coiled around a Rust-style gear, eyeing an apple.

Usage (needs Pillow):  python assets/icon/generate_icon.py
Outputs, next to this script:
  rsnake.ico  multi-size Windows icon, used by the installer (installer\rsnake.iss) for itself,
              the shortcuts and the "Installed apps" entry
  rsnake.png  256x256 preview

Everything is drawn from the coordinates below (no font, no external artwork), on a 256x256 design
grid rendered 4x larger and then downscaled for antialiasing. The gear with its "R" is a redrawn
homage to the Rust logo (a trademark of the Rust Foundation), not the official artwork.
Small sizes use a simplified drawing (thicker outline, fewer coils and teeth, no apple).
"""

import io
import math
import struct
from pathlib import Path

from PIL import Image, ImageDraw

GRID = 256  # design grid, in units
SUPERSAMPLING = 4
ICO_SIZES = [16, 20, 24, 32, 40, 48, 64, 256]
SIMPLIFIED_UP_TO = 32  # sizes up to this one use the simplified drawing

OUTLINE = (20, 58, 30, 255)
BODY = (96, 200, 64, 255)
BODY_LIGHT = (176, 236, 120, 255)
SCALES = (54, 150, 52, 255)
TONGUE = (232, 44, 62, 255)
EYE_WHITE = (255, 255, 255, 255)
EYE_PUPIL = (22, 28, 24, 255)
RUST = (226, 100, 40, 255)
RUST_LIGHT = (250, 164, 92, 255)
RUST_OUTLINE = (84, 30, 10, 255)
APPLE = (230, 44, 52, 255)
APPLE_LIGHT = (255, 150, 150, 255)
APPLE_OUTLINE = (104, 14, 24, 255)
STEM = (110, 66, 30, 255)

CENTER = (128, 134)  # centre of the gear


def polar(angle, radius):
    """Point at `angle` degrees (clockwise from the right, as the y-axis points down) from CENTER."""
    a = math.radians(angle)
    return CENTER[0] + radius * math.cos(a), CENTER[1] + radius * math.sin(a)


def catmull_rom(points, steps=24):
    """Smooth curve through `points` (uniform Catmull-Rom spline), any number of coordinates."""
    padded = [points[0]] + points + [points[-1]]
    curve = []
    for i in range(1, len(padded) - 2):
        p0, p1, p2, p3 = padded[i - 1], padded[i], padded[i + 1], padded[i + 2]
        for step in range(steps):
            t = step / steps
            curve.append(tuple(
                0.5 * (2 * b + (c - a) * t + (2 * a - 5 * b + 4 * c - d) * t * t + (3 * b - 3 * c + d - a) * t ** 3)
                for a, b, c, d in zip(p0, p1, p2, p3)))
    curve.append(points[-1])
    return curve


class Layer:
    """Transparent layer with drawing helpers working in design-grid units."""

    def __init__(self):
        self.scale = SUPERSAMPLING
        side = GRID * self.scale
        self.image = Image.new("RGBA", (side, side), (0, 0, 0, 0))
        self.draw = ImageDraw.Draw(self.image)

    def disc(self, center, radius, color, squash=1.0):
        x, y = center
        s = self.scale
        self.draw.ellipse(
            [(x - radius) * s, (y - radius * squash) * s, (x + radius) * s, (y + radius * squash) * s],
            fill=color)

    def stroke(self, points, radius, color):
        """Round-capped thick line through `points`; `radius` is a number or a list (one per point)."""
        radii = radius if isinstance(radius, list) else [radius] * len(points)
        if len(points) == 1:
            self.disc(points[0], radii[0], color)
        for (a, ra), (b, rb) in zip(zip(points, radii), zip(points[1:], radii[1:])):
            count = max(1, int(math.dist(a, b) * self.scale / 2))
            for i in range(count + 1):
                t = i / count
                self.disc((a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t), ra + (rb - ra) * t, color)

    def polygon(self, points, color):
        self.draw.polygon([(x * self.scale, y * self.scale) for x, y in points], fill=color)


def draw_gear(layer, simplified, outline):
    """Rust-style gear: toothed ring with a bold serif "R" in the middle."""
    teeth = 12 if simplified else 18
    tip, outer, inner = (99, 84, 60) if simplified else (97, 85, 63)
    half_base, half_tip = (360 / teeth) * 0.30, (360 / teeth) * 0.13

    def draw_teeth(grow, color):
        widen = math.degrees(grow / outer)
        for i in range(teeth):
            angle = i * 360 / teeth - 90
            layer.polygon([polar(angle - half_base - widen, outer - 4), polar(angle - half_tip - widen, tip + grow),
                           polar(angle + half_tip + widen, tip + grow), polar(angle + half_base + widen, outer - 4)],
                          color)

    draw_teeth(outline, RUST_OUTLINE)
    layer.disc(CENTER, outer + outline, RUST_OUTLINE)
    draw_teeth(0, RUST)
    layer.disc(CENTER, outer, RUST)
    if not simplified:
        # Glossy highlight on the upper left of the ring
        layer.stroke([polar(a, outer - 6) for a in range(190, 300, 3)], 2.6, RUST_LIGHT)
    layer.disc(CENTER, inner, RUST_OUTLINE)
    layer.disc(CENTER, inner - outline, (0, 0, 0, 0))

    # The "R": stem, bowl, leg and serifs
    cx, cy = CENTER
    k = 1.0 if simplified else 0.84  # smaller when the coils of the snake come inside the ring
    weight = 9.5 if simplified else 6.5
    left, top, bottom = cx - 20 * k, cy - 29 * k, cy + 29 * k
    middle = cy + 1
    bowl_radius = (middle - top) / 2
    bowl_x = cx + 6 * k
    bowl = [(left, top), (bowl_x, top)]
    bowl += [(bowl_x + bowl_radius * math.cos(math.radians(a)), top + bowl_radius * (1 + math.sin(math.radians(a))))
             for a in range(-90, 91, 6)]
    bowl += [(left, middle)]
    letter = [[(left, top), (left, bottom)], bowl, [(bowl_x - 2, middle), (cx + 24 * k, bottom)],
              [(left - 9 * k, top), (left, top)], [(left - 9 * k, bottom), (left + 10 * k, bottom)],
              [(cx + 17 * k, bottom), (cx + 31 * k, bottom)]]
    for color, extra in ((RUST_OUTLINE, outline), (RUST, 0)):
        for part in letter:
            layer.stroke(part, weight + extra, color)
    if not simplified:
        layer.stroke([(left - 2, top + 3), (left - 2, bottom - 8)], 1.8, RUST_LIGHT)


def snake_spine(simplified):
    """Spine from the tail tip to the head as (x, y, depth) points.

    The body winds around the ring of the gear like a spring: `depth` is positive where it passes
    in front of the ring, negative where it passes behind. The neck and head stay in front.
    """
    controls = []
    if simplified:
        # Too small for coils: the snake simply lies around the gear, over its teeth
        controls = [(*polar(angle, 98), 1.0) for angle in range(12, 205, 16)]
    else:
        turns = 3
        start_angle, end_angle = -22, 204
        mid_ring, out_reach, in_reach = 74, 29, 17
        steps = turns * 8
        for i in range(steps + 1):
            phase = math.pi / 2 + 2 * math.pi * turns * i / steps  # starts and ends outside the ring
            swing = math.sin(phase)
            radius = mid_ring + swing * (out_reach if swing > 0 else in_reach)
            x, y = polar(start_angle + (end_angle - start_angle) * i / steps, radius)
            controls.append((x, y, math.cos(phase)))
    for angle, radius in ((226, 113), (243, 106)):  # neck, outside the gear
        controls.append((*polar(angle, radius), 1.0))
    controls.append((*polar(256, 99), 1.0))  # head, resting on the top of the gear
    return catmull_rom(controls)


def draw_apple(layer, outline):
    center, radius = (214, 44), 19
    layer.stroke([(center[0] + 1, center[1] - radius + 3), (center[0] + 5, center[1] - radius - 9)], 2.2 + outline * 0.5,
                 APPLE_OUTLINE)
    for color, grow in ((APPLE_OUTLINE, outline), (APPLE, 0)):
        layer.disc((center[0] - 7, center[1]), radius - 5 + grow, color, squash=1.12)
        layer.disc((center[0] + 7, center[1]), radius - 5 + grow, color, squash=1.12)
    layer.disc((center[0] - 9, center[1] - 7), 4.5, APPLE_LIGHT, squash=1.3)
    layer.stroke([(center[0] + 1, center[1] - radius + 3), (center[0] + 5, center[1] - radius - 9)], 2.2, STEM)
    leaf = [(center[0] + 8, center[1] - radius - 6), (center[0] + 18, center[1] - radius - 9)]
    layer.stroke(leaf, 4.5 + outline * 0.5, OUTLINE)
    layer.stroke(leaf, 4.5, BODY)


def draw_icon(simplified):
    outline = 8 if simplified else 5
    body_radius = 16 if simplified else 13

    spine = snake_spine(simplified)
    points = [(x, y) for x, y, _ in spine]
    front = [depth > 0 for _, _, depth in spine]
    count = len(points)
    # Thin tail tip growing to the full thickness over the first third
    radii = [body_radius * min(1.0, 0.14 + 0.86 * i / (count * 0.3)) for i in range(count)]

    def draw_body(layer, first, last, with_outline):
        """Draws the spine samples first..last (inclusive)."""
        segment, segment_radii = points[first:last + 1], radii[first:last + 1]
        if with_outline:
            layer.stroke(segment, [r + outline for r in segment_radii], OUTLINE)
        layer.stroke(segment, segment_radii, BODY)
        if not simplified:
            layer.stroke([(x, y - r * 0.42) for (x, y), r in zip(segment, segment_radii)],
                         [r * 0.2 for r in segment_radii], BODY_LIGHT)
            for i in range(first - first % 13 + 13, last + 1, 13):
                if i > count * 0.08:
                    layer.disc(points[i], radii[i] * 0.3, SCALES)

    # Runs of consecutive samples passing in front of the ring
    runs, start = [], None
    for i, is_front in enumerate(front + [False]):
        if is_front and start is None:
            start = i
        elif not is_front and start is not None:
            runs.append((start, i - 1))
            start = None

    # Back to front: whole snake, gear over it, then the coils passing in front of the gear
    back, gear, top = Layer(), Layer(), Layer()
    draw_body(back, 0, count - 1, with_outline=True)
    draw_gear(gear, simplified, outline)
    for first, last in runs:
        draw_body(top, first, last, with_outline=True)
    # Where a front run starts or ends (outside the ring) its outline cap would cut the body: repaint
    # the body fill of the neighbouring samples over it
    reach = int(1.6 * (body_radius + outline) * count / sum(math.dist(a, b) for a, b in zip(points, points[1:])))
    for first, last in runs:
        if first > 0:
            draw_body(top, max(0, first - reach), first, with_outline=False)
        if last < count - 1:
            draw_body(top, last, min(count - 1, last + reach), with_outline=False)

    # Head (looking right, towards the apple), tongue behind it
    head = points[-1]
    head_radius = body_radius + (9 if simplified else 8.5)
    tongue_width = 4.2 if simplified else 2.8
    tip = (head[0] + head_radius + (17 if simplified else 21), head[1] + 1)
    for color, extra in ((OUTLINE, outline * 0.55), (TONGUE, 0)):
        top.stroke([(head[0] + head_radius - 6, head[1] + 5), tip], tongue_width + extra, color)
        top.stroke([tip, (tip[0] + 10, tip[1] - 8)], tongue_width * 0.8 + extra, color)
        top.stroke([tip, (tip[0] + 10, tip[1] + 8)], tongue_width * 0.8 + extra, color)
    top.disc(head, head_radius + outline, OUTLINE, squash=0.86)
    top.stroke(points[-12:], radii[-12:], BODY)  # joins the neck and the head without an outline between them
    top.disc(head, head_radius, BODY, squash=0.86)
    if not simplified:
        top.disc((head[0] - 2, head[1] - head_radius * 0.52), head_radius * 0.26, BODY_LIGHT, squash=0.55)
        top.disc((head[0] + head_radius - 5, head[1] - 3), 1.8, OUTLINE)
    eye = (head[0] + 6, head[1] - 5)
    eye_radius = 10.5 if simplified else 8
    top.disc(eye, eye_radius + outline * 0.45, OUTLINE)
    top.disc(eye, eye_radius, EYE_WHITE)
    top.disc((eye[0] + 2.2, eye[1] + 0.8), eye_radius * (0.62 if simplified else 0.56), EYE_PUPIL)
    if not simplified:
        top.disc((eye[0] + 4.4, eye[1] - 1.6), 1.5, EYE_WHITE)
        draw_apple(top, outline)

    image = back.image
    image.alpha_composite(gear.image)
    image.alpha_composite(top.image)
    return image


def downscale(image, size):
    # Premultiplied alpha avoids dark fringes around the transparent background
    return image.convert("RGBa").resize((size, size), Image.Resampling.LANCZOS).convert("RGBA")


def ico_bitmap(image):
    """32-bit BGRA DIB as stored in .ico files (double height, bottom-up, followed by an AND mask)."""
    size = image.width
    header = struct.pack("<IiiHHIIiiII", 40, size, size * 2, 1, 32, 0, 0, 0, 0, 0, 0)
    pixels = image.transpose(Image.Transpose.FLIP_TOP_BOTTOM).tobytes("raw", "BGRA")
    mask_row = bytes(((size + 31) // 32) * 4)  # all zeros: transparency comes from the alpha channel
    return header + pixels + mask_row * size


def ico_png(image):
    buffer = io.BytesIO()
    image.save(buffer, format="PNG", optimize=True)
    return buffer.getvalue()


def main():
    folder = Path(__file__).resolve().parent
    detailed, simple = draw_icon(simplified=False), draw_icon(simplified=True)
    images = {size: downscale(simple if size <= SIMPLIFIED_UP_TO else detailed, size) for size in ICO_SIZES}

    # Classic layout: uncompressed bitmaps for the small sizes, PNG for 256x256
    payloads = [ico_png(images[size]) if size >= 256 else ico_bitmap(images[size]) for size in ICO_SIZES]
    offset = 6 + 16 * len(ICO_SIZES)
    directory = struct.pack("<HHH", 0, 1, len(ICO_SIZES))
    for size, payload in zip(ICO_SIZES, payloads):
        directory += struct.pack("<BBBBHHII", size % 256, size % 256, 0, 0, 1, 32, len(payload), offset)
        offset += len(payload)
    (folder / "rsnake.ico").write_bytes(directory + b"".join(payloads))
    images[256].save(folder / "rsnake.png", optimize=True)
    print(f"Wrote rsnake.ico ({offset} bytes, sizes {ICO_SIZES}) and rsnake.png in {folder}")


if __name__ == "__main__":
    main()
