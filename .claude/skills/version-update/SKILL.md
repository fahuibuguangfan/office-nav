---
name: version-update
description: |
  自动更新项目版本号并同步所有配置文件（tauri.conf.json、Cargo.toml、package.json、app-version.json）。
  支持命令：/version-update（交互式选择：1=小版本 1.2.6→1.2.7、2=中版本 1.2.6→1.3.0、3=大版本 1.2.6→2.0.0、4=自定义）、/version-update 1.3.0（直接指定版本号）。
  自动生成更新日志，提交代码、创建并推送 v* Git 标签触发 CI/CD 自动打包（10-15分钟），CDN 同步（1-5分钟）。
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

## ⚠️ 核心机制（必须遵守）

本项目的 GitHub Actions Release 工作流（`.github/workflows/release.yml`）**只在推送 `v*` 开头的 Git 标签时触发**，普通 push 到 master **不会**触发打包。

因此发版**必须**完成两步推送：
1. 推送 master 分支（代码）
2. 推送 `v<版本号>` 标签（触发 CI）

**禁止**只手动改文件 + 推 master 就当作发版完成 —— 那样 GitHub 上不会有任何构建。

## 执行方式（强制使用脚本）

**必须**调用技能目录下的 `update-version.sh` 脚本来完成发版，**不要**手动 Edit 版本文件后自行 commit/push。脚本统一处理：4 个文件更新 → 提交 → 建标签 → 推分支 → 推标签 → 刷 CDN，并自带 JSON 合法性校验和推送重试。

```bash
bash .claude/skills/version-update/update-version.sh <新版本号> "<更新日志>"
```

只有当脚本因环境问题无法运行时，才按 `instructions.md` 的步骤手动执行，且**必须**手动 `git tag -a v<版本号>` 并 `git push origin v<版本号>`。

## 触发条件

用户明确要求更新版本号时，如：
- "更新版本到 1.3.0"
- "升级版本号"
- "发布新版本 v1.2.7"
- "/version-update 1.3.0"
- "/version-update"（不带参数，交互式选择）

## 功能

1. **同步更新 4 个版本号文件**：
   - `src-tauri/tauri.conf.json`
   - `src-tauri/Cargo.toml`（应用运行时通过 `env!("CARGO_PKG_VERSION")` 读取，不同步会导致版本检查异常）
   - `package.json`
   - `public/app-version.json`

2. **更新 app-version.json 内容**：
   - 版本号
   - 发布日期（当天）
   - 下载地址（GitHub Release）
   - 更新日志（数组格式，由用户提供或根据 git diff 自动生成）

3. **Git 提交、建标签、推送**：
   - 提交版本更新
   - 创建 `v<版本号>` 附注标签
   - 推送 master 分支 + 推送标签到 GitHub
   - **推送标签才会触发 CI/CD 自动打包**

4. **推送后校验**：
   - 用 `gh run list --limit 3` 确认 Release 工作流已被标签推送触发
   - 若未触发，排查标签是否推送成功

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

## 更新日志来源

按优先级获取：
1. 用户手动提供
2. 用户选择"根据代码变更自动生成" → 用 `git diff HEAD` 分析未提交改动，归纳为条目
3. 都没有 → 询问用户

## 注意事项

- 版本号格式：`x.y.z`（如 1.2.6）
- 会自动检查版本号是否递增
- **必须推送 `v*` 标签才会触发 GitHub Actions 打包**（约 10-15 分钟）
- CDN 同步时间：1-5 分钟（脚本会主动 purge jsDelivr，CI 打包后兜底再刷一次）

## 相关文件

- `src-tauri/tauri.conf.json` - Tauri 配置文件
- `src-tauri/Cargo.toml` - Rust 包配置（应用运行时版本来源）
- `package.json` - NPM 包配置
- `public/app-version.json` - 在线更新配置
- `.github/workflows/release.yml` - CI 打包工作流（由 v* 标签触发）
- `.claude/skills/version-update/update-version.sh` - 发版执行脚本
- `.claude/skills/version-update/instructions.md` - 详细分步指令（脚本不可用时的兜底）
