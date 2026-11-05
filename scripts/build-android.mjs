#!/usr/bin/env node

import { execSync } from 'child_process';
import path from 'path';
import fs from 'fs';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const args = process.argv.slice(2);
const buildType = args.includes('debug') ? 'debug' : 'release';

const projectRoot = path.resolve(__dirname, '..');
const gradleDir = path.join(projectRoot, 'src-tauri', 'gen', 'android');

console.log(`🔨 开始构建 Android ${buildType} APK...`);

try {
  console.log('\n📦 步骤 1/2: 构建前端...');
  execSync('pnpm build', {
    cwd: projectRoot,
    stdio: 'inherit'
  });

  console.log('\n📦 步骤 2/2: 构建 Android APK...');

  const env = { ...process.env };
  env.ANDROID_SDK_ROOT = 'C:/Users/74710/AppData/Local/Android/Sdk';
  env.ANDROID_NDK_ROOT = 'C:/Users/74710/AppData/Local/Android/Sdk/ndk/27.0.12077973';
  env.ANDROID_HOME = env.ANDROID_SDK_ROOT;

  const gradleCmd = buildType === 'debug' ? 'assembleDebug' : 'assembleRelease';
  const gradleWrapper = process.platform === 'win32'
    ? path.join(gradleDir, 'gradlew.bat')
    : path.join(gradleDir, 'gradlew');

  const cmd = `"${gradleWrapper}" ${gradleCmd} -x rustBuildArm64Release -x rustBuildArmRelease -x rustBuildX86Release -x rustBuildX86_64Release`;

  execSync(cmd, {
    cwd: gradleDir,
    stdio: 'inherit',
    shell: true,
    env
  });

  const outputDir = path.join(gradleDir, 'app', 'build', 'outputs', 'apk', 'arm64', buildType);
  const apkFile = path.join(outputDir, `app-arm64-${buildType}.apk`);

  if (fs.existsSync(apkFile)) {
    const stats = fs.statSync(apkFile);
    const sizeMB = (stats.size / (1024 * 1024)).toFixed(2);
    console.log(`\n✅ 构建成功！`);
    console.log(`📁 输出: ${apkFile}`);
    console.log(`📊 大小: ${sizeMB} MB`);
  }
} catch (error) {
  console.error(`\n❌ 构建失败`);
  process.exit(1);
}
