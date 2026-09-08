# 构建与发布指南

## 一、开发环境准备

### 1. 必备工具
- Node.js >= 18.0
- pnpm >= 8.0
- Rust >= 1.70
- Tauri CLI 2.x

### 2. 安装依赖
```bash
# 安装前端依赖
pnpm install

# 安装 Tauri CLI
cargo install tauri-cli --version "^2.0.0"
```

## 二、开发模式

### 启动开发服务器
```bash
# 方式一：使用 pnpm
pnpm tauri dev

# 方式二：使用 cargo
cargo tauri dev
```

开发模式特性：
- 热重载（前端）
- 自动重新编译（Rust 后端）
- DevTools 可用
- 详细的错误日志

## 三、构建生产版本

### 1. 构建前检查

**检查清单**：
- [ ] 更新版本号（`src-tauri/Cargo.toml` 和 `src-tauri/tauri.conf.json`）
- [ ] 更新应用图标（`src-tauri/icons/`）
- [ ] 检查配置文件（数据源 URL、API 地址）
- [ ] 运行测试确保功能正常
- [ ] 清理开发用的 console.log

### 2. 执行构建

```bash
# 构建生产版本（优化模式）
pnpm tauri build

# 构建调试版本（包含调试符号）
pnpm tauri build --debug
```

### 3. 构建产物

构建完成后，安装包位于：

**Windows**:
- NSIS 安装程序: `src-tauri/target/release/bundle/nsis/*.exe`
- MSI 安装程序: `src-tauri/target/release/bundle/msi/*.msi`
- 可执行文件: `src-tauri/target/release/tauri-network-navigation.exe`

**macOS**:
- DMG: `src-tauri/target/release/bundle/dmg/*.dmg`
- APP: `src-tauri/target/release/bundle/macos/*.app`

**Linux**:
- DEB: `src-tauri/target/release/bundle/deb/*.deb`
- AppImage: `src-tauri/target/release/bundle/appimage/*.AppImage`

## 四、优化配置

### 1. Cargo 优化（已配置）

`src-tauri/Cargo.toml`:
```toml
[profile.release]
codegen-units = 1    # 更好的优化，但编译更慢
lto = true           # 链接时优化
opt-level = 3        # 最高优化级别
panic = "abort"      # 减小二进制文件大小
strip = true         # 移除调试符号
```

### 2. 前端优化

- 使用 Vite 的代码分割
- 按需加载 Ant Design Vue 组件
- 图片资源优化
- Tree-shaking 未使用的代码

## 五、签名与公证

### Windows 代码签名

```bash
# 设置证书指纹
$env:TAURI_SIGNING_PRIVATE_KEY = "证书指纹"

# 构建并签名
pnpm tauri build
```

### macOS 签名和公证

```bash
# 设置开发者证书
export APPLE_CERTIFICATE = "Developer ID Application: ..."
export APPLE_CERTIFICATE_PASSWORD = "..."
export APPLE_ID = "your@email.com"
export APPLE_PASSWORD = "app-specific-password"

# 构建、签名、公证
pnpm tauri build
```

## 六、版本管理

### 版本号规范

遵循语义化版本：`MAJOR.MINOR.PATCH`

- **MAJOR**: 不兼容的 API 变更
- **MINOR**: 向后兼容的功能新增
- **PATCH**: 向后兼容的问题修复

### 更新版本

1. 更新 `src-tauri/Cargo.toml`:
```toml
[package]
version = "1.1.0"
```

2. 更新 `src-tauri/tauri.conf.json`:
```json
{
  "version": "1.1.0"
}
```

3. 创建 Git 标签:
```bash
git tag -a v1.1.0 -m "Release version 1.1.0"
git push origin v1.1.0
```

## 七、发布流程

### 1. GitHub Releases

```bash
# 1. 推送代码和标签
git push origin main
git push origin v1.1.0

# 2. 在 GitHub 上创建 Release
# 3. 上传构建产物（.exe, .dmg, .deb 等）
# 4. 编写 Release Notes
```

### 2. 自动更新配置

Tauri 支持内置的自动更新功能。需要：

1. 配置更新服务器
2. 生成更新清单
3. 在应用中启用更新检查

详见：https://tauri.app/v1/guides/distribution/updater

## 八、CI/CD 自动化

### GitHub Actions 示例

创建 `.github/workflows/build.yml`:

```yaml
name: Build and Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    strategy:
      matrix:
        platform: [windows-latest, macos-latest, ubuntu-latest]
    
    runs-on: ${{ matrix.platform }}
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Node
        uses: actions/setup-node@v3
        with:
          node-version: 18
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Install pnpm
        run: npm install -g pnpm
      
      - name: Install dependencies
        run: pnpm install
      
      - name: Build
        run: pnpm tauri build
      
      - name: Upload artifacts
        uses: actions/upload-artifact@v3
        with:
          name: ${{ matrix.platform }}-builds
          path: src-tauri/target/release/bundle/
```

## 九、常见问题

### 1. 构建失败

**问题**: Rust 编译错误  
**解决**: 
```bash
# 清理并重新构建
cargo clean
pnpm tauri build
```

### 2. 安装包过大

**问题**: 生成的安装包体积超过 50MB  
**解决**:
- 检查是否包含了不必要的依赖
- 确保 `strip = true` 已启用
- 使用 `cargo bloat` 分析二进制文件大小

### 3. 权限问题

**问题**: macOS 提示"应用已损坏"  
**解决**: 必须进行代码签名和公证

**问题**: Windows SmartScreen 警告  
**解决**: 使用 EV 代码签名证书

## 十、测试清单

发布前测试：

- [ ] 安装包能否正常安装
- [ ] 应用能否正常启动
- [ ] 所有功能是否正常工作
- [ ] 自动启动是否生效
- [ ] 系统托盘是否正常
- [ ] 窗口大小和位置是否正确
- [ ] 能否正常卸载

## 十一、文档更新

发布时需要更新的文档：

- [ ] README.md（功能、截图、下载链接）
- [ ] CHANGELOG.md（版本变更记录）
- [ ] 用户手册（如有）
- [ ] API 文档（如有）

## 十二、性能优化建议

1. **减小包体积**
   - 使用 upx 压缩可执行文件（可选）
   - 移除未使用的依赖
   - 优化图片资源

2. **提升启动速度**
   - 延迟加载非关键模块
   - 使用缓存减少初始化时间
   - 优化 Rust 代码性能

3. **减少内存占用**
   - 避免内存泄漏
   - 及时清理不需要的资源
   - 使用对象池复用对象

---

**最后更新**: 2026-09-07  
**适用版本**: v1.0.0+
