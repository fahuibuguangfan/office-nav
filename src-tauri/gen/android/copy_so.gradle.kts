// 处理 Windows 上的 .so 文件复制而不创建符号链接
afterEvaluate {
    tasks.withName(".*Compile.*Kotlin") {
        doLast {
            val targets = mapOf(
                "aarch64-linux-android" to "arm64-v8a",
                "armv7-linux-androideabi" to "armeabi-v7a",
                "i686-linux-android" to "x86",
                "x86_64-linux-android" to "x86_64"
            )

            val rootDir = File(project.rootDir, "../../")
            val cargoTargetDir = File(rootDir, "target")
            val jniDir = File(project.projectDir, "app/src/main/jniLibs")

            targets.forEach { (cargoTarget, abiDir) ->
                listOf("debug", "release").forEach { profile ->
                    val soSrc = File(cargoTargetDir, "$cargoTarget/$profile/libtauri_network_navigation_lib.so")
                    if (soSrc.exists()) {
                        val jniAbiDir = File(jniDir, abiDir)
                        jniAbiDir.mkdirs()
                        val soDest = File(jniAbiDir, "libtauri_network_navigation_lib.so")
                        try {
                            soSrc.copyTo(soDest, overwrite = true)
                            println("✓ 复制 $abiDir/$profile")
                        } catch (e: Exception) {
                            println("✗ 复制失败: ${e.message}")
                        }
                    }
                }
            }
        }
    }
}
