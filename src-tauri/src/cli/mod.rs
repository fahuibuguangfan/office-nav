use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::Command;

/// 办公室服务导航 CLI 工具
#[derive(Parser)]
#[command(version, about = "办公室服务导航命令行工具", long_about = None)]
#[command(name = get_program_name())]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// 显示版本信息
    #[command(visible_alias = "v")]
    Version,

    /// 启动或唤起 GUI 主程序
    Open,
}

/// 从 argv[0] 获取程序名（hs 或 huashun）
fn get_program_name() -> &'static str {
    let args: Vec<String> = std::env::args().collect();
    if args.is_empty() {
        return "hs";
    }

    let exe_path = PathBuf::from(&args[0]);
    let file_stem = exe_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("hs");

    // 返回静态字符串（仅支持 hs 和 huashun）
    if file_stem == "huashun" {
        "huashun"
    } else {
        "hs"
    }
}

/// CLI 入口
pub fn run() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Version) => {
            show_version();
        }
        Some(Commands::Open) => {
            if let Err(e) = open_gui() {
                eprintln!("启动 GUI 失败: {}", e);
                std::process::exit(1);
            }
        }
        None => {
            // 没有子命令时显示帮助（--version 会被 clap 自动处理）
            Cli::parse_from(&["", "--help"]);
        }
    }
}

/// 显示版本信息
fn show_version() {
    let version = env!("CARGO_PKG_VERSION");
    let platform = get_platform();
    let commit = option_env!("GIT_COMMIT_HASH").unwrap_or("unknown");

    println!("办公室服务导航 CLI v{}", version);
    println!("平台: {}", platform);
    println!("Commit: {}", commit);
}

/// 获取平台信息
fn get_platform() -> &'static str {
    if cfg!(target_os = "windows") {
        if cfg!(target_arch = "x86_64") {
            "windows-x86_64"
        } else if cfg!(target_arch = "aarch64") {
            "windows-aarch64"
        } else {
            "windows"
        }
    } else if cfg!(target_os = "macos") {
        if cfg!(target_arch = "aarch64") {
            "darwin-aarch64"
        } else {
            "darwin-x86_64"
        }
    } else if cfg!(target_os = "linux") {
        if cfg!(target_arch = "x86_64") {
            "linux-x86_64"
        } else if cfg!(target_arch = "aarch64") {
            "linux-aarch64"
        } else {
            "linux"
        }
    } else {
        "unknown"
    }
}

/// 启动或唤起 GUI 主程序
fn open_gui() -> Result<(), String> {
    // CLI 程序和 GUI 主程序在同一个目录
    let cli_exe = std::env::current_exe()
        .map_err(|e| format!("无法获取当前程序路径: {}", e))?;

    let cli_dir = cli_exe.parent()
        .ok_or("无法获取程序目录")?;

    // 根据平台查找主程序
    let gui_exe = if cfg!(target_os = "windows") {
        // Windows 下可能是 "办公室服务导航.exe" 或 "tauri-network-navigation.exe"
        let chinese_name = cli_dir.join("办公室服务导航.exe");
        let english_name = cli_dir.join("tauri-network-navigation.exe");

        if chinese_name.exists() {
            chinese_name
        } else if english_name.exists() {
            english_name
        } else {
            return Err(format!(
                "找不到 GUI 主程序:\n  {}\n  {}\n\n提示：请确保 CLI 和 GUI 程序在同一目录",
                chinese_name.display(),
                english_name.display()
            ));
        }
    } else if cfg!(target_os = "macos") {
        // macOS 下可能在 .app/Contents/MacOS/ 目录
        cli_dir.join("办公室服务导航")
    } else {
        cli_dir.join("办公室服务导航")
    };

    if !gui_exe.exists() {
        return Err(format!(
            "找不到 GUI 主程序: {}\n提示：请确保 CLI 和 GUI 程序在同一目录",
            gui_exe.display()
        ));
    }

    // 启动 GUI（如果已运行，single-instance 插件会唤起已有窗口）
    Command::new(&gui_exe)
        .spawn()
        .map_err(|e| format!("启动 GUI 失败: {}", e))?;

    println!("GUI 已启动");
    Ok(())
}
