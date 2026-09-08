function readPackage(pkg) {
  // 批准所有包的构建脚本
  if (pkg.name === 'core-js' || pkg.name === 'vue-demi') {
    pkg.scripts = pkg.scripts || {}
  }
  return pkg
}

module.exports = {
  hooks: {
    readPackage
  }
}
