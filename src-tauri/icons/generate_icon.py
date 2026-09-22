#!/usr/bin/env python3
"""
生成导航图标 - 指南针样式
需要安装 Pillow: pip install Pillow
"""

from PIL import Image, ImageDraw, ImageFont
import math

def draw_navigation_icon(size):
    """绘制导航图标（指南针）"""
    # 创建图像
    img = Image.new('RGB', (size, size), color='white')
    draw = ImageDraw.Draw(img)

    center = size // 2
    scale = size / 512

    # 背景渐变（简化为纯色）
    for y in range(size):
        # 从 #1890ff 到 #096dd9 的渐变
        r = int(24 + (9 - 24) * y / size)
        g = int(144 + (109 - 144) * y / size)
        b = int(255 + (217 - 255) * y / size)
        draw.line([(0, y), (size, y)], fill=(r, g, b))

    # 外圈（白色圆环）
    ring_radius = int(180 * scale)
    ring_width = int(16 * scale)
    draw.ellipse(
        [center - ring_radius, center - ring_radius,
         center + ring_radius, center + ring_radius],
        outline='white',
        width=ring_width
    )

    # 内圈（淡白色）
    inner_radius = int(140 * scale)
    draw.ellipse(
        [center - inner_radius, center - inner_radius,
         center + inner_radius, center + inner_radius],
        outline=(255, 255, 255, 100),
        width=int(2 * scale)
    )

    # 指针 - 北（红色三角形）
    pointer_height = int(120 * scale)
    pointer_width = int(20 * scale)
    north_points = [
        (center, center - pointer_height),  # 顶点
        (center + pointer_width, center),    # 右下
        (center - pointer_width, center)     # 左下
    ]
    draw.polygon(north_points, fill='#ff4d4f')

    # 指针 - 南（白色三角形）
    south_points = [
        (center, center + pointer_height),  # 底点
        (center + pointer_width, center),    # 右上
        (center - pointer_width, center)     # 左上
    ]
    draw.polygon(south_points, fill='white')

    # 中心圆（白色）
    center_radius = int(25 * scale)
    draw.ellipse(
        [center - center_radius, center - center_radius,
         center + center_radius, center + center_radius],
        fill='white'
    )

    # 中心小圆（蓝色）
    inner_center_radius = int(15 * scale)
    draw.ellipse(
        [center - inner_center_radius, center - inner_center_radius,
         center + inner_center_radius, center + inner_center_radius],
        fill='#1890ff'
    )

    # 方向标记 N
    try:
        # 尝试使用系统字体
        font_size = int(40 * scale)
        font = ImageFont.truetype("arial.ttf", font_size)
    except:
        # 如果没有找到字体，使用默认字体
        font = ImageFont.load_default()

    text = "N"
    # 使用 textbbox 获取文字边界
    bbox = draw.textbbox((0, 0), text, font=font)
    text_width = bbox[2] - bbox[0]
    text_height = bbox[3] - bbox[1]

    text_x = center - text_width // 2
    text_y = center - int(150 * scale) - text_height // 2
    draw.text((text_x, text_y), text, fill='white', font=font)

    return img

# 生成不同尺寸的图标
sizes = {
    '32x32.png': 32,
    '128x128.png': 128,
    '128x128@2x.png': 256,
    'icon.png': 512
}

print("Start generating navigation icons...")
for filename, size in sizes.items():
    img = draw_navigation_icon(size)
    img.save(filename)
    print(f"Generated: {filename} ({size}x{size})")

print("\nPNG icons generated successfully!")
print("\nNext steps:")
print("1. Generate ICO (Windows):")
print("   Online tool: https://www.icoconverter.com/")
print("   Or use command: magick icon.png -define icon:auto-resize=256,128,64,48,32,16 icon.ico")
print("\n2. Generate ICNS (macOS):")
print("   mkdir icon.iconset")
print("   for size in 16 32 128 256 512; do")
print("     magick icon.png -resize ${size}x${size} icon.iconset/icon_${size}x${size}.png")
print("   done")
print("   iconutil -c icns icon.iconset")
