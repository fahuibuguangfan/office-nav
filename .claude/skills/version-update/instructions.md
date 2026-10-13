当用户要求更新项目版本时，执行以下步骤：

## 1. 解析版本号

### 如果用户提供了明确版本号
直接使用该版本号（格式：x.y.z）。

### 如果用户未提供版本号或输入不明确
读取当前版本并提供快捷选项：

```
当前版本：1.2.6

请选择升级方式：
1. 🔹 升级一个小版本（Bug 修复、小优化）→ 1.2.7
2. 🔸 升级一个中版本（新功能、改进）→ 1.3.0
3. 🔺 升级一个大版本（重大更新）→ 2.0.0
4. 🎯 自定义版本号

请输入选项编号或直接输入版本号：
```

根据用户选择自动计算新版本号：
- 选项 1：修订版本 +1（1.2.6 → 1.2.7）
- 选项 2：次版本 +1，修订版本归 0（1.2.6 → 1.3.0）
- 选项 3：主版本 +1，次版本和修订版本归 0（1.2.6 → 2.0.0）
- 选项 4：询问用户输入自定义版本号

## 2. 读取当前版本

读取 `src-tauri/tauri.conf.json` 获取当前版本号，检查新版本是否比当前版本更高。

## 3. 询问更新日志

询问用户本次更新的主要内容，用于填写 `public/app-version.json` 的 `update_log` 字段（数组格式，每项一条更新说明）。

**提示用户**：请提供本次更新的主要内容（每行一条，回车分隔）：

## 4. 更新文件

### 4.1 更新 src-tauri/tauri.conf.json

```json
{
  "version": "新版本号"
}
```

### 4.2 更新 package.json

```json
{
  "version": "新版本号"
}
```

### 4.3 更新 public/app-version.json

```json
{
  "version": "新版本号",
  "release_date": "今天日期（YYYY-MM-DD）",
  "download_url": {
    "windows": "https://github.com/fahuibuguangfan/office-nav/releases/download/v新版本号/_新版本号_x64-setup.exe",
    "macos": "https://github.com/fahuibuguangfan/office-nav/releases/download/v新版本号/_新版本号_aarch64.dmg",
    "linux": "https://github.com/fahuibuguangfan/office-nav/releases/download/v新版本号/_新版本号_amd64.AppImage"
  },
  "update_log": [
    用户提供的更新说明
  ],
  "force_update": false
}
```

## 5. Git 提交并推送

```bash
git add src-tauri/tauri.conf.json package.json public/app-version.json
git commit -m "chore: 升级版本至 v新版本号"
git push
```

## 6. 提示用户

告知用户：
- ✅ 版本号已更新到 x.y.z
- ✅ 已推送到 GitHub
- ⏳ GitHub Actions 将自动打包（约 10-15 分钟）
- 📦 打包完成后，用户可通过应用内"检查更新"功能获取新版本
- 🕐 CDN 同步时间约 1-5 分钟

## 错误处理

- 如果新版本号不合法（非 x.y.z 格式），提示用户重新输入
- 如果新版本号不高于当前版本，询问用户是否确认继续
- 如果 git push 失败，提示用户手动推送

## 示例交互

### 示例 1：快捷升级

```
用户：/version-update

助手：当前版本：1.2.6

     请选择升级方式：
     1. 🔹 升级一个小版本（Bug 修复、小优化）→ 1.2.7
     2. 🔸 升级一个中版本（新功能、改进）→ 1.3.0
     3. 🔺 升级一个大版本（重大更新）→ 2.0.0
     4. 🎯 自定义版本号

     请输入选项编号或直接输入版本号：

用户：1

助手：准备升级到 v1.2.7
     请提供本次更新的主要内容（每行一条）：

用户：
- 修复启动时检查更新失败的问题
- 优化设置菜单布局

助手：正在更新版本号...
     ✅ src-tauri/tauri.conf.json - 1.2.6 → 1.2.7
     ✅ package.json - 1.2.6 → 1.2.7
     ✅ public/app-version.json - 更新完成
     
     正在提交并推送...
     ✅ 已提交：chore: 升级版本至 v1.2.7
     ✅ 已推送到 GitHub
     
     🎉 版本更新完成！
     ⏳ GitHub Actions 将自动打包（约 10-15 分钟）
     🕐 CDN 同步时间约 1-5 分钟
     📦 打包完成后用户可通过"设置 → 检查更新"获取新版本
```

### 示例 2：指定版本号

```
用户：/version-update 1.3.0

助手：当前版本：1.2.6
     新版本：1.3.0 ✅

     请提供本次更新的主要内容（每行一条）：

用户：
- 新增黑暗模式支持
- 优化启动速度
- 修复已知 Bug

助手：正在更新版本号...
     （后续流程同示例 1）
```
