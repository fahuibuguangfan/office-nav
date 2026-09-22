const { createCanvas } = require('canvas');
const fs = require('fs');
const path = require('path');

// 生成不同尺寸的图标
const sizes = [
  { name: '32x32.png', size: 32 },
  { name: '128x128.png', size: 128 },
  { name: '128x128@2x.png', size: 256 },
  { name: 'icon.png', size: 512 }
];

function drawNavigationIcon(canvas) {
  const ctx = canvas.getContext('2d');
  const size = canvas.width;
  const center = size / 2;
  const scale = size / 512; // 基于512标准缩放

  // 背景渐变
  const gradient = ctx.createLinearGradient(0, 0, size, size);
  gradient.addColorStop(0, '#1890ff');
  gradient.addColorStop(1, '#096dd9');
  ctx.fillStyle = gradient;
  ctx.fillRect(0, 0, size, size);

  // 绘制导航图标（指南针）
  ctx.save();
  ctx.translate(center, center);

  // 外圈
  ctx.strokeStyle = '#ffffff';
  ctx.lineWidth = 16 * scale;
  ctx.beginPath();
  ctx.arc(0, 0, 180 * scale, 0, Math.PI * 2);
  ctx.stroke();

  // 内圈
  ctx.strokeStyle = 'rgba(255, 255, 255, 0.4)';
  ctx.lineWidth = 2 * scale;
  ctx.beginPath();
  ctx.arc(0, 0, 140 * scale, 0, Math.PI * 2);
  ctx.stroke();

  // 指针 - 北（红色）
  ctx.fillStyle = '#ff4d4f';
  ctx.beginPath();
  ctx.moveTo(0, -120 * scale);
  ctx.lineTo(20 * scale, 0);
  ctx.lineTo(-20 * scale, 0);
  ctx.closePath();
  ctx.fill();

  // 指针 - 南（白色）
  ctx.fillStyle = '#ffffff';
  ctx.beginPath();
  ctx.moveTo(0, 120 * scale);
  ctx.lineTo(20 * scale, 0);
  ctx.lineTo(-20 * scale, 0);
  ctx.closePath();
  ctx.fill();

  // 中心圆
  ctx.fillStyle = '#ffffff';
  ctx.beginPath();
  ctx.arc(0, 0, 25 * scale, 0, Math.PI * 2);
  ctx.fill();

  ctx.fillStyle = '#1890ff';
  ctx.beginPath();
  ctx.arc(0, 0, 15 * scale, 0, Math.PI * 2);
  ctx.fill();

  // 方向标记
  ctx.fillStyle = '#ffffff';
  ctx.font = `bold ${40 * scale}px Arial`;
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.fillText('N', 0, -150 * scale);

  ctx.restore();
}

// 生成所有尺寸的图标
sizes.forEach(({ name, size }) => {
  const canvas = createCanvas(size, size);
  drawNavigationIcon(canvas);

  const buffer = canvas.toBuffer('image/png');
  const outputPath = path.join(__dirname, name);
  fs.writeFileSync(outputPath, buffer);
  console.log(`✅ 已生成: ${name} (${size}x${size})`);
});

console.log('\n🎉 所有图标生成完成！');
console.log('⚠️  请手动生成 .ico 和 .icns 文件：');
console.log('   - Windows: 使用在线工具将 icon.png 转换为 icon.ico');
console.log('   - macOS: 使用 iconutil 工具将 icon.png 转换为 icon.icns');
