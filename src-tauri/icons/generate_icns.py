#!/usr/bin/env python3
"""
将 icon.png 转换为 icon.icns (macOS 图标)
需要 pillow-icns: pip install pillow-icns
"""

from PIL import Image
import os
import shutil

print("Converting icon.png to icon.icns...")

# 读取生成的 PNG 图标
img = Image.open('icon.png')

# macOS iconset 需要的尺寸
iconset_sizes = [
    (16, 'icon_16x16.png'),
    (32, 'icon_16x16@2x.png'),
    (32, 'icon_32x32.png'),
    (64, 'icon_32x32@2x.png'),
    (128, 'icon_128x128.png'),
    (256, 'icon_128x128@2x.png'),
    (256, 'icon_256x256.png'),
    (512, 'icon_256x256@2x.png'),
    (512, 'icon_512x512.png'),
    (1024, 'icon_512x512@2x.png'),
]

# 创建 iconset 目录
iconset_dir = 'icon.iconset'
if os.path.exists(iconset_dir):
    shutil.rmtree(iconset_dir)
os.makedirs(iconset_dir)

# 生成各个尺寸的图标
for size, filename in iconset_sizes:
    resized = img.resize((size, size), Image.Resampling.LANCZOS)
    resized.save(os.path.join(iconset_dir, filename))
    print(f"  Generated: {filename} ({size}x{size})")

print(f"\nIconset directory created: {iconset_dir}")
print("\nTo generate ICNS file on macOS, run:")
print(f"  iconutil -c icns {iconset_dir}")
print("\nNote: iconutil is only available on macOS.")
print("For Windows/Linux, you can use online tools or png2icns package.")

# 尝试使用 pillow-icns（如果安装了的话）
try:
    # 简单方法：保存为 ICNS（需要 macOS 或 pillow-icns）
    import subprocess

    # 检查是否在 macOS 上
    result = subprocess.run(['iconutil', '-c', 'icns', iconset_dir],
                          capture_output=True, text=True)
    if result.returncode == 0:
        print("\nSuccessfully generated: icon.icns")
    else:
        print(f"\niconutil not available: {result.stderr}")
        print("ICNS generation skipped (requires macOS)")
except Exception as e:
    print(f"\nCould not run iconutil: {e}")
    print("To generate ICNS, please run iconutil manually on macOS")

print("\nDone!")
