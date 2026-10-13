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
echo "📝 [3/3] 更新 public/app-version.json..."
APP_VERSION="public/app-version.json"
if [ -f "$APP_VERSION" ]; then
  # 读取现有内容
  OLD_CONTENT=$(cat "$APP_VERSION")

  # 转换更新日志为 JSON 数组格式
  IFS=$'\n' read -rd '' -a LINES <<<"$CHANGELOG" || true
  JSON_ARRAY=""
  for line in "${LINES[@]}"; do
    # 去除首尾空格
    line=$(echo "$line" | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')
    [ -z "$line" ] && continue

    if [ -z "$JSON_ARRAY" ]; then
      JSON_ARRAY="\"$line\""
    else
      JSON_ARRAY="$JSON_ARRAY,\n    \"$line\""
    fi
  done

  # 生成新的 JSON 内容
  cat > "$APP_VERSION" << EOF
{
  "version": "$NEW_VERSION",
  "release_date": "$TODAY",
  "download_url": {
    "windows": "https://github.com/fahuibuguangfan/office-nav/releases/download/v$NEW_VERSION/_${NEW_VERSION}_x64-setup.exe",
    "macos": "https://github.com/fahuibuguangfan/office-nav/releases/download/v$NEW_VERSION/_${NEW_VERSION}_aarch64.dmg",
    "linux": "https://github.com/fahuibuguangfan/office-nav/releases/download/v$NEW_VERSION/_${NEW_VERSION}_amd64.AppImage"
  },
  "update_log": [
    $JSON_ARRAY
  ],
  "force_update": false
}
EOF
  echo "   ✅ app-version.json 更新完成"
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
