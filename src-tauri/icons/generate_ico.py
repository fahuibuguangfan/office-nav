#!/usr/bin/env python3
"""
将 icon.png 转换为 icon.ico (Windows 图标)
"""

from PIL import Image

print("Converting icon.png to icon.ico...")

# 读取生成的 PNG 图标
img = Image.open('icon.png')

# ICO 文件需要多个尺寸
sizes = [(16, 16), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)]

# 生成不同尺寸的图像列表
images = []
for size in sizes:
    resized = img.resize(size, Image.Resampling.LANCZOS)
    images.append(resized)

# 保存为 ICO 文件
img.save('icon.ico', format='ICO', sizes=[(image.width, image.height) for image in images])

print("Successfully generated: icon.ico")
print("Sizes included: 16x16, 32x32, 48x48, 64x64, 128x128, 256x256")
