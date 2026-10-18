#!/usr/bin/env python3
"""生成 Lumos 品牌图标。

设计意象：深色圆角底座 + 一本打开的书 + 一束从书页升起的光，
对应 "Lumos"（点亮）与 "知识库" 两个核心意象。全部图形用代码绘制，
不依赖任何外部素材或字体，方便任何人复现。

用法:
    python scripts/generate-icon.py

输出:
    docs/assets/lumos-icon.png   1024x1024 源图（Tauri 图标源）
    public/favicon.png           256x256 网页图标
"""

from __future__ import annotations

from pathlib import Path

from PIL import Image, ImageChops, ImageDraw, ImageFilter

SIZE = 1024
PLATE_MARGIN = 64
PLATE_RADIUS = 224

ROOT = Path(__file__).resolve().parent.parent
ASSETS = ROOT / "docs" / "assets"
PUBLIC = ROOT / "public"

# 品牌色
INK_TOP = (27, 33, 64)
INK_BOTTOM = (9, 12, 26)
EDGE = (44, 53, 99)
BRAND_VIOLET = (139, 124, 255)
BEAM_CYAN = (51, 214, 195)
PAPER_LIGHT = (247, 245, 255)
PAPER_SHADE = (223, 217, 250)
PAPER_STROKE = (185, 176, 234)


def rounded_mask(size: int, margin: int, radius: int) -> Image.Image:
    """生成圆角矩形遮罩。"""
    mask = Image.new("L", (size, size), 0)
    ImageDraw.Draw(mask).rounded_rectangle(
        (margin, margin, size - margin, size - margin), radius=radius, fill=255
    )
    return mask


def vertical_gradient(
    size: int, top: tuple[int, int, int], bottom: tuple[int, int, int]
) -> Image.Image:
    """生成自上而下的线性渐变。"""
    gradient = Image.new("RGBA", (size, size))
    draw = ImageDraw.Draw(gradient)
    for y in range(size):
        t = y / (size - 1)
        color = tuple(round(top[i] + (bottom[i] - top[i]) * t) for i in range(3))
        draw.line([(0, y), (size, y)], fill=(*color, 255))
    return gradient


def radial_glow(size: int, color: tuple[int, int, int], max_alpha: int) -> Image.Image:
    """生成柔和的径向光晕。"""
    glow = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    pixels = glow.load()
    center = size / 2
    for y in range(size):
        for x in range(size):
            distance = ((x - center) ** 2 + (y - center) ** 2) ** 0.5 / center
            falloff = max(0.0, 1.0 - distance) ** 2.4
            alpha = int(falloff * max_alpha)
            if alpha:
                pixels[x, y] = (*color, alpha)
    return glow


def build_icon() -> Image.Image:
    """绘制 1024x1024 图标。"""
    canvas = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))

    # 1. 圆角底座
    plate = vertical_gradient(SIZE, INK_TOP, INK_BOTTOM)
    plate.putalpha(rounded_mask(SIZE, PLATE_MARGIN, PLATE_RADIUS))
    canvas.alpha_composite(plate)

    # 2. 底座内的品牌光晕
    glow = radial_glow(760, BRAND_VIOLET, 96)
    canvas.alpha_composite(glow, (SIZE // 2 - 380, 300))

    # 3. 向上的光束（底部窄、顶部宽，越靠上越淡）
    beam_mask = Image.new("L", (SIZE, SIZE), 0)
    ImageDraw.Draw(beam_mask).polygon(
        [(486, 612), (538, 612), (628, 262), (396, 262)], fill=255
    )
    fade = Image.new("L", (SIZE, SIZE))
    fade_draw = ImageDraw.Draw(fade)
    for y in range(262, 613):
        t = (y - 262) / (613 - 262)
        fade_draw.line([(0, y), (SIZE, y)], fill=int(60 + 130 * t))
    beam_mask = ImageChops.multiply(beam_mask, fade)
    beam_color = vertical_gradient(SIZE, BEAM_CYAN, BRAND_VIOLET)
    beam = beam_color.copy()
    beam.putalpha(beam_mask.filter(ImageFilter.GaussianBlur(6)))
    canvas.alpha_composite(beam)

    # 4. 书页（先画左页，再画右页，中间留出书脊）
    draw = ImageDraw.Draw(canvas)
    left_page = [(236, 618), (496, 660), (496, 764), (236, 722)]
    right_page = [(788, 618), (528, 660), (528, 764), (788, 722)]
    draw.polygon(left_page, fill=(*PAPER_LIGHT, 255), outline=PAPER_STROKE, width=3)
    draw.polygon(right_page, fill=(*PAPER_SHADE, 255), outline=PAPER_STROKE, width=3)

    # 5. 内页线，增加“书”的可读性
    for offset in (26, 52):
        draw.line(
            [(262, 630 + offset - 14), (470, 664 + offset - 14)],
            fill=(*PAPER_STROKE, 210),
            width=6,
        )
        draw.line(
            [(554, 664 + offset - 14), (762, 630 + offset - 14)],
            fill=(*PAPER_STROKE, 210),
            width=6,
        )

    # 6. 书脊
    draw.line([(512, 660), (512, 764)], fill=(*PAPER_STROKE, 255), width=8)

    # 7. 顶部四角星光
    draw.polygon(
        [
            (512, 118),
            (526, 176),
            (584, 190),
            (526, 204),
            (512, 262),
            (498, 204),
            (440, 190),
            (498, 176),
        ],
        fill=(*PAPER_LIGHT, 235),
    )
    draw.polygon(
        [
            (512, 158),
            (520, 182),
            (544, 190),
            (520, 198),
            (512, 222),
            (504, 198),
            (480, 190),
            (504, 182),
        ],
        fill=(*BRAND_VIOLET, 220),
    )

    # 8. 底座描边
    draw.rounded_rectangle(
        (PLATE_MARGIN, PLATE_MARGIN, SIZE - PLATE_MARGIN, SIZE - PLATE_MARGIN),
        radius=PLATE_RADIUS,
        outline=(*EDGE, 255),
        width=4,
    )

    return canvas


def main() -> None:
    """写出源图与网页图标。"""
    ASSETS.mkdir(parents=True, exist_ok=True)
    PUBLIC.mkdir(parents=True, exist_ok=True)

    icon = build_icon()
    icon.save(ASSETS / "lumos-icon.png")
    icon.resize((256, 256), Image.LANCZOS).save(PUBLIC / "favicon.png")

    print(f"icon source -> {ASSETS / 'lumos-icon.png'}")
    print(f"favicon     -> {PUBLIC / 'favicon.png'}")


if __name__ == "__main__":
    main()
