#!/bin/bash
# version-update 技能专用脚本
# 功能：自动更新项目版本号、提交代码、创建标签并推送
# 用法：bash update-version.sh <new_version> <changelog>

set -e  # 遇到错误立即退出

# 参数校验
if [ -z "$1" ]; then
  echo "❌ 错误：缺少版本号参数"
  echo "用法：bash update-version.sh <版本号> <更新日志>"
  exit 1
fi

if [ -z "$2" ]; then
  echo "❌ 错误：缺少更新日志参数"
  echo "用法：bash update-version.sh <版本号> <更新日志>"
  exit 1
fi

NEW_VERSION="$1"
CHANGELOG="$2"
TODAY=$(date +%Y-%m-%d)

# 版本号格式校验
if ! [[ "$NEW_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "❌ 错误：版本号格式不正确（应为 x.y.z）"
  exit 1
fi

echo "🚀 开始版本升级：v$NEW_VERSION"
echo ""

# 1. 更新 tauri.conf.json
echo "📝 [1/3] 更新 src-tauri/tauri.conf.json..."
TAURI_CONF="src-tauri/tauri.conf.json"
if [ -f "$TAURI_CONF" ]; then
  # 使用 sed 替换版本号（兼容 macOS 和 Linux）
  if [[ "$OSTYPE" == "darwin"* ]]; then
    sed -i '' "s/\"version\": \"[^\"]*\"/\"version\": \"$NEW_VERSION\"/" "$TAURI_CONF"
  else
    sed -i "s/\"version\": \"[^\"]*\"/\"version\": \"$NEW_VERSION\"/" "$TAURI_CONF"
  fi
  echo "   ✅ tauri.conf.json 更新完成"
else
  echo "   ⚠️  未找到 $TAURI_CONF，跳过"
fi

# 2. 更新 package.json
echo "📝 [2/3] 更新 package.json..."
PACKAGE_JSON="package.json"
if [ -f "$PACKAGE_JSON" ]; then
  if [[ "$OSTYPE" == "darwin"* ]]; then
    sed -i '' "s/\"version\": \"[^\"]*\"/\"version\": \"$NEW_VERSION\"/" "$PACKAGE_JSON"
  else
    sed -i "s/\"version\": \"[^\"]*\"/\"version\": \"$NEW_VERSION\"/" "$PACKAGE_JSON"
  fi
  echo "   ✅ package.json 更新完成"
else
  echo "   ⚠️  未找到 $PACKAGE_JSON，跳过"
fi

# 3. 更新 app-version.json
# 重要：app-version.json 会被应用拉取并由 serde_json/reqwest 反序列化为 VersionInfo 结构体，
# 必须是严格合法的 JSON。早期版本用 bash 字符串拼接生成 JSON，存在两个致命缺陷：
#   (a) 多行更新日志在数组元素之间插入字面 "\n"（反斜杠+n），导致 ",\n" 这种非法转义；
#   (b) bash 的字符串替换对反斜杠转义不可靠，含反斜杠的日志（如 Windows 路径）会被吞字符。
# 现改为把更新日志通过临时文件传给 node，用 JSON.stringify 保证转义正确，彻底根治校验失败。
echo "📝 [3/3] 更新 public/app-version.json..."
APP_VERSION="public/app-version.json"
if [ -f "$APP_VERSION" ]; then
  # 将多行更新日志写入临时文件（保留原始字节，不做任何 shell 转义）
  LOG_TMP=$(mktemp)
  printf '%s\n' "$CHANGELOG" > "$LOG_TMP"

  # 用 node 读取日志文件并生成严格合法的 JSON，写入 app-version.json
  # node 在 Tauri/Vite 项目环境中一定可用；node 会原样读取文件字节，
  # 再交给 JSON.stringify 做正确转义，杜绝 bash 拼接 JSON 的隐患。
  if command -v node >/dev/null 2>&1; then
    node -e "
      const fs = require('fs');
      const ver = process.argv[1];
      const date = process.argv[2];
      const logFile = process.argv[3];
      const outFile = process.argv[4];
      // 按行切分，去首尾空白，丢弃空行
      const lines = fs.readFileSync(logFile, 'utf8')
        .split(/\r?\n/)
        .map(s => s.trim())
        .filter(s => s.length > 0);
      const data = {
        version: ver,
        release_date: date,
        download_url: {
          windows: 'https://github.com/fahuibuguangfan/office-nav/releases/download/v' + ver + '/_' + ver + '_x64-setup.exe',
          macos: 'https://github.com/fahuibuguangfan/office-nav/releases/download/v' + ver + '/_' + ver + '_aarch64.dmg',
          linux: 'https://github.com/fahuibuguangfan/office-nav/releases/download/v' + ver + '/_' + ver + '_amd64.AppImage'
        },
        update_log: lines,
        force_update: false
      };
      fs.writeFileSync(outFile, JSON.stringify(data, null, 2) + '\n', 'utf8');
    " "$NEW_VERSION" "$TODAY" "$LOG_TMP" "$APP_VERSION"
    rm -f "$LOG_TMP"
    echo "   ✅ app-version.json 更新完成"
  else
    # 兜底：无 node 时用 python3（同样能保证 JSON 合法）
    python3 -c "
import sys, json
ver, date, log_file, out_file = sys.argv[1:5]
with open(log_file, 'r', encoding='utf-8') as f:
    lines = [ln.strip() for ln in f.read().splitlines() if ln.strip()]
data = {
    'version': ver,
    'release_date': date,
    'download_url': {
        'windows': 'https://github.com/fahuibuguangfan/office-nav/releases/download/v' + ver + '/_' + ver + '_x64-setup.exe',
        'macos': 'https://github.com/fahuibuguangfan/office-nav/releases/download/v' + ver + '/_' + ver + '_aarch64.dmg',
        'linux': 'https://github.com/fahuibuguangfan/office-nav/releases/download/v' + ver + '/_' + ver + '_amd64.AppImage'
    },
    'update_log': lines,
    'force_update': False
}
with open(out_file, 'w', encoding='utf-8') as f:
    json.dump(data, f, ensure_ascii=False, indent=2)
    f.write('\n')
" "$NEW_VERSION" "$TODAY" "$LOG_TMP" "$APP_VERSION"
    rm -f "$LOG_TMP"
    echo "   ✅ app-version.json 更新完成（python3 兜底）"
  fi

  # 写入后立即自校验：如果生成的 JSON 不合法，立即报错中止，避免提交坏文件
  if command -v node >/dev/null 2>&1; then
    node -e "JSON.parse(require('fs').readFileSync('$APP_VERSION','utf8'))" \
      || { echo "❌ 生成的 app-version.json 校验失败，已中止"; exit 1; }
  fi
else
  echo "   ⚠️  未找到 $APP_VERSION，跳过"
fi

echo ""
echo "📦 提交更改..."

# 4. Git 提交
git add src-tauri/tauri.conf.json package.json public/app-version.json 2>/dev/null || true
git commit -m "chore: 升级版本至 v$NEW_VERSION

$CHANGELOG"

echo "   ✅ 代码已提交"

# 5. 创建 Git 标签
echo "🏷️  创建标签 v$NEW_VERSION..."
git tag -a "v$NEW_VERSION" -m "Release v$NEW_VERSION

$CHANGELOG"
echo "   ✅ 标签已创建"

# 6. 推送代码和标签
echo "🚀 推送到 GitHub..."
git push origin master || git push origin main || echo "⚠️  推送分支失败，请检查远程分支名"
git push origin "v$NEW_VERSION"
echo "   ✅ 代码和标签已推送"

echo ""
echo "🎉 版本升级完成！"
echo ""
echo "📋 版本信息："
echo "   版本号：v$NEW_VERSION"
echo "   发布日期：$TODAY"
echo "   标签：v$NEW_VERSION"
echo ""
echo "⏳ 后续流程："
echo "   1. GitHub Actions 自动打包（约 10-15 分钟）"
echo "   2. CDN 同步 app-version.json（约 1-5 分钟）"
echo "   3. 用户启动应用时自动检测更新"
echo ""
