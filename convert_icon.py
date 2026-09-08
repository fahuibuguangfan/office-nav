#!/usr/bin/env python3
"""
转换图标为 RGBA 格式
"""
from PIL import Image
import os

# 图标文件路径
icon_path = "src-tauri/icons/icon.png"

print(f"正在转换图标: {icon_path}")

# 打开图标
img = Image.open(icon_path)
print(f"原始格式: {img.mode}, 尺寸: {img.size}")

# 转换为 RGBA 模式
if img.mode != 'RGBA':
    img = img.convert('RGBA')
    print(f"已转换为: {img.mode}")

    # 保存为 RGBA 格式
    img.save(icon_path, 'PNG')
    print(f"✓ 图标已保存为 RGBA 格式")
else:
    print("✓ 图标已经是 RGBA 格式")

# 同时转换其他尺寸的图标
icon_dir = "src-tauri/icons"
for filename in os.listdir(icon_dir):
    if filename.endswith('.png') and filename != 'icon.png':
        filepath = os.path.join(icon_dir, filename)
        try:
            img = Image.open(filepath)
            if img.mode != 'RGBA':
                img = img.convert('RGBA')
                img.save(filepath, 'PNG')
                print(f"✓ {filename} 已转换为 RGBA 格式")
        except Exception as e:
            print(f"✗ {filename} 转换失败: {e}")

print("\n所有图标转换完成！")
