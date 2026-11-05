import java.io.File
import org.apache.tools.ant.taskdefs.condition.Os
import org.gradle.api.DefaultTask
import org.gradle.api.GradleException
import org.gradle.api.logging.LogLevel
import org.gradle.api.tasks.Input
import org.gradle.api.tasks.TaskAction

open class BuildTask : DefaultTask() {
    @Input
    var rootDirRel: String? = null
    @Input
    var target: String? = null
    @Input
    var release: Boolean? = null

    @TaskAction
    fun assemble() {
        val environment = System.getenv().toMutableMap()

        // 确保 PATH 包含 pnpm
        val nodePath = "C:\\nvm4w\\nodejs"
        val currentPath = environment["PATH"] ?: ""
        if (!currentPath.contains(nodePath)) {
            environment["PATH"] = "$nodePath;$currentPath"
        }

        // 禁用符号链接（Windows 不需要开发者模式）
        environment["TAURI_ANDROID_NO_SYMLINK"] = "1"

        val executable = "pnpm.cmd"
        runTauriCli(executable, environment)
    }

    fun runTauriCli(executable: String, environment: Map<String, String>? = null) {
        val rootDirRel = rootDirRel ?: throw GradleException("rootDirRel cannot be null")
        val target = target ?: throw GradleException("target cannot be null")
        val release = release ?: throw GradleException("release cannot be null")
        val args = listOf("tauri", "android", "android-studio-script");

        project.exec {
            workingDir(File(project.projectDir, rootDirRel))
            if (environment != null) {
                environment(environment)
            }
            executable(executable)
            args(args)
            if (project.logger.isEnabled(LogLevel.DEBUG)) {
                args("-vv")
            } else if (project.logger.isEnabled(LogLevel.INFO)) {
                args("-v")
            }
            if (release) {
                args("--release")
            }
            args(listOf("--target", target))
        }.assertNormalExitValue()
    }
}