---
name: version-update
description: |
  自动更新项目版本号并同步所有配置文件（tauri.conf.json、package.json、app-version.json）。
  支持命令：/version-update（交互式选择：1=小版本 1.2.6→1.2.7、2=中版本 1.2.6→1.3.0、3=大版本 1.2.6→2.0.0、4=自定义）、/version-update 1.3.0（直接指定版本号）。
  自动生成更新日志，提交并推送代码，触发 CI/CD 自动打包（10-15分钟），CDN 同步（1-5分钟）。
  版本号规则：遵循语义化版本（主.次.修订），小版本=Bug修复，中版本=新功能，大版本=重大更新。
user-invocable: true
argument-hint: "[版本号] 或留空交互式选择升级方式"
allowed-tools:
  - Read
  - Edit
  - Bash
  - AskUserQuestion
---

# /version-update — 自动版本更新

## 触发条件

用户明确要求更新版本号时，如：
- "更新版本到 1.3.0"
- "升级版本号"
- "发布新版本 v1.2.7"
- "/version-update 1.3.0"
- "/version-update"（不带参数，交互式选择）

## 功能

1. **同步更新所有版本号文件**：
   - `src-tauri/tauri.conf.json`
   - `package.json`
   - `public/app-version.json`

2. **更新 app-version.json 内容**：
   - 版本号
   - 发布日期（当天）
   - 下载地址（GitHub Release）
   - 更新日志（需用户提供）

3. **Git 提交并推送**：
   - 自动提交版本更新
   - 推送到 GitHub
   - 触发 CI/CD 自动打包

## 使用方式

### 方式一：指定版本号
```bash
/version-update 1.3.0
```

### 方式二：快捷升级（推荐）
```bash
/version-update          # 交互式选择升级方式
```

**交互式选项**：
- 🔹 **升级一个小版本**（Bug 修复、小优化）：1.2.6 → 1.2.7
- 🔸 **升级一个中版本**（新功能、改进）：1.2.6 → 1.3.0
- 🔺 **升级一个大版本**（重大更新）：1.2.6 → 2.0.0
- 🎯 **自定义版本号**：手动输入版本号

## 版本号规则

遵循语义化版本 (Semantic Versioning)：`主版本.次版本.修订版本`

| 类型 | 说明 | 示例 |
|------|------|------|
| 修订版本（Patch） | Bug 修复、小优化、文档更新 | 1.2.6 → 1.2.7 |
| 次版本（Minor） | 新功能、向下兼容的改进 | 1.2.6 → 1.3.0 |
| 主版本（Major） | 重大更新、不兼容的 API 变更 | 1.2.6 → 2.0.0 |

## 注意事项

- 版本号格式：`x.y.z`（如 1.2.6）
- 必须提供更新日志内容
- 会自动检查版本号是否递增
- 推送后会触发 GitHub Actions 自动打包（约 10-15 分钟）
- CDN 同步时间：1-5 分钟

## 相关文件

- `src-tauri/tauri.conf.json` - Tauri 配置文件
- `package.json` - NPM 包配置
- `public/app-version.json` - 在线更新配置
