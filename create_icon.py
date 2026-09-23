from PIL import Image, ImageDraw, ImageFont
import os

# 创建一个 1024x1024 的新图标（蓝色渐变背景 + 白色 N 字母）
size = 1024
img = Image.new('RGBA', (size, size), (0, 0, 0, 0))
draw = ImageDraw.Draw(img)

# 绘制圆角矩形背景（蓝色渐变效果）
for i in range(size):
    color = (int(41 + (100-41)*i/size), int(128 + (180-128)*i/size), int(185 + (220-185)*i/size), 255)
    draw.rectangle([0, i, size, i+1], fill=color)

# 绘制圆角效果
corner_radius = 180
draw.ellipse([0, 0, corner_radius*2, corner_radius*2], fill=(41, 128, 185, 0))
draw.ellipse([size-corner_radius*2, 0, size, corner_radius*2], fill=(41, 128, 185, 0))
draw.ellipse([0, size-corner_radius*2, corner_radius*2, size], fill=(100, 180, 220, 0))
draw.ellipse([size-corner_radius*2, size-corner_radius*2, size, size], fill=(100, 180, 220, 0))

# 绘制白色的 "N" 字母
try:
    font = ImageFont.truetype("arial.ttf", 600)
except:
    font = ImageFont.load_default()

text = "N"
bbox = draw.textbbox((0, 0), text, font=font)
text_width = bbox[2] - bbox[0]
text_height = bbox[3] - bbox[1]
position = ((size - text_width) // 2, (size - text_height) // 2 - 50)
draw.text(position, text, fill=(255, 255, 255, 255), font=font)

img.save('app-icon.png')
print("图标已创建: app-icon.png")
