import java.util.Properties

plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
    id("rust")
}

val tauriProperties = Properties().apply {
    val propFile = file("tauri.properties")
    if (propFile.exists()) {
        propFile.inputStream().use { load(it) }
    }
}

// 签名配置：优先读取 keystore.properties（不入库），缺失时回落到 release.keystore
val keystoreProperties = Properties().apply {
    val propFile = file("keystore.properties")
    if (propFile.exists()) {
        propFile.inputStream().use { load(it) }
    }
}

android {
    compileSdk = 36
    namespace = "com.office.network_navigation"
    defaultConfig {
        manifestPlaceholders["usesCleartextTraffic"] = "false"
        applicationId = "com.office.network_navigation"
        minSdk = 24
        targetSdk = 36
        versionCode = tauriProperties.getProperty("tauri.android.versionCode", "1").toInt()
        versionName = tauriProperties.getProperty("tauri.android.versionName", "1.0")
    }
    signingConfigs {
        create("release") {
            storeFile = file("release.keystore")
            storePassword = keystoreProperties.getProperty("storePassword", "networknav2026")
            keyAlias = keystoreProperties.getProperty("keyAlias", "network_navigation")
            keyPassword = keystoreProperties.getProperty("keyPassword", "networknav2026")
        }
    }
    buildTypes {
        getByName("debug") {
            manifestPlaceholders["usesCleartextTraffic"] = "true"
            isDebuggable = true
            isJniDebuggable = true
            isMinifyEnabled = false
            packaging {                jniLibs.keepDebugSymbols.add("*/arm64-v8a/*.so")
                jniLibs.keepDebugSymbols.add("*/armeabi-v7a/*.so")
                jniLibs.keepDebugSymbols.add("*/x86/*.so")
                jniLibs.keepDebugSymbols.add("*/x86_64/*.so")
            }
        }
        getByName("release") {
            isMinifyEnabled = true
            signingConfig = signingConfigs.getByName("release")
            proguardFiles(
                *fileTree(".") { include("**/*.pro") }
                    .plus(getDefaultProguardFile("proguard-android-optimize.txt"))
                    .toList().toTypedArray()
            )
        }
    }
    kotlinOptions {
        jvmTarget = "1.8"
    }
    buildFeatures {
        buildConfig = true
    }
}

rust {
    rootDirRel = "../../../"
}

dependencies {
    implementation("androidx.webkit:webkit:1.14.0")
    implementation("androidx.appcompat:appcompat:1.7.1")
    implementation("androidx.activity:activity-ktx:1.10.1")
    implementation("com.google.android.material:material:1.12.0")
    implementation("androidx.lifecycle:lifecycle-process:2.10.0")
    testImplementation("junit:junit:4.13.2")
    androidTestImplementation("androidx.test.ext:junit:1.1.4")
    androidTestImplementation("androidx.test.espresso:espresso-core:3.5.0")
}

apply(from = "tauri.build.gradle.kts")

// Windows 上复制 .so 文件而不创建符号链接
tasks.register("copySoFiles") {
    doLast {
        val targets = listOf(
            "aarch64-linux-android" to "arm64-v8a",
            "armv7-linux-androideabi" to "armeabi-v7a",
            "i686-linux-android" to "x86",
            "x86_64-linux-android" to "x86_64"
        )

        val rootDir = rootProject.projectDir.parentFile.parentFile
        val cargoTargetDir = File(rootDir, "target")

        targets.forEach { (cargoTarget, abiDir) ->
            listOf("debug", "release").forEach { profile ->
                val soSrc = File(cargoTargetDir, "$cargoTarget/$profile/libtauri_network_navigation_lib.so")
                if (soSrc.exists()) {
                    val jniDir = File(projectDir, "src/main/jniLibs/$abiDir")
                    jniDir.mkdirs()
                    val soDest = File(jniDir, "libtauri_network_navigation_lib.so")
                    soSrc.copyTo(soDest, overwrite = true)
                    logger.info("复制 .so 文件: $soSrc -> $soDest")
                }
            }
        }
    }
}

// 在编译前运行复制任务
tasks.withName("compile.*ReleaseKotlin") {
    finalizedBy("copySoFiles")
}