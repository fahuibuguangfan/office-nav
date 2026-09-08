# 图标生成指南

## 当前状态
- ✅ 已创建 SVG 图标：`public/icon.svg`
- ⚠️ 需要生成各种尺寸的图标文件

## 需要生成的图标

### 1. 使用 Tauri Icon 工具（推荐）

Tauri 提供了官方的图标生成工具。

#### 安装工具：
```bash
npm install -g @tauri-apps/cli
# 或者
cargo install tauri-cli
```

#### 生成图标：
```bash
# 从 512x512 的 PNG 图标生成所有尺寸
pnpm tauri icon path/to/icon.png
```

### 2. 手动生成（如果需要）

需要生成以下文件到 `src-tauri/icons/` 目录：

- `icon.ico` - Windows 图标（256x256）
- `icon.icns` - macOS 图标
- `32x32.png` - 小图标
- `128x128.png` - 中等图标
- `128x128@2x.png` - 高分辨率中等图标（256x256）
- `icon.png` - 托盘图标（512x512）

### 3. 在线工具

如果没有安装图标生成工具，可以使用在线服务：

1. **SVG 转 PNG**：
   - https://svgtopng.com/
   - https://cloudconvert.com/svg-to-png

2. **生成 ICO（Windows）**：
   - https://icoconvert.com/
   - https://convertio.co/zh/png-ico/

3. **生成 ICNS（macOS）**：
   - https://cloudconvert.com/png-to-icns
   - https://anyconv.com/png-to-icns-converter/

### 4. 使用 ImageMagick（命令行）

```bash
# 安装 ImageMagick
# Windows: choco install imagemagick
# macOS: brew install imagemagick
# Linux: sudo apt install imagemagick

# 从 SVG 生成 PNG
magick public/icon.svg -resize 512x512 src-tauri/icons/icon.png
magick public/icon.svg -resize 256x256 src-tauri/icons/128x128@2x.png
magick public/icon.svg -resize 128x128 src-tauri/icons/128x128.png
magick public/icon.svg -resize 32x32 src-tauri/icons/32x32.png

# 生成 ICO（Windows）
magick src-tauri/icons/icon.png -define icon:auto-resize=256,128,64,48,32,16 src-tauri/icons/icon.ico
```

## 临时方案

当前项目使用 Tauri 默认图标，可以正常运行。等生成自定义图标后，替换 `src-tauri/icons/` 目录下的文件即可。

## 替换步骤

1. 生成所有尺寸的图标文件
2. 将文件复制到 `src-tauri/icons/` 目录
3. 复制一份 `icon.png` 到 `public/` 目录（用于标题栏显示）
4. 重新编译应用
