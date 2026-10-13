# version-update

自动更新项目版本号，确保所有版本相关文件同步。

## 触发条件

用户明确要求更新版本号时，如：
- "更新版本到 1.3.0"
- "升级版本号"
- "发布新版本 v1.2.7"
- "/version-update 1.3.0"

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

```bash
/version-update <新版本号>
```

## 注意事项

- 版本号格式：`x.y.z`（如 1.2.6）
- 必须提供更新日志内容
- 会自动检查版本号是否递增
- 推送后会触发 GitHub Actions 自动打包

## 相关文件

- `src-tauri/tauri.conf.json` - Tauri 配置文件
- `package.json` - NPM 包配置
- `public/app-version.json` - 在线更新配置
