//! 子进程控制台窗口控制
//!
//! Windows 上，当宿主进程为 GUI 子系统（无控制台）时，`std::process::Command`
//! / `tokio::process::Command` 启动的子进程会各自弹出一个黑色控制台窗口。
//! 通过 `CREATE_NO_WINDOW` 创建标志抑制该行为。
//!
//! debug 构建下宿主自身带控制台（`windows_subsystem = "windows"` 未生效），
//! 子进程继承父控制台故不弹窗；release 构建下宿主无控制台，故需此标志。
//!
//! 提供 [`HideWindowExt::hide_console_window`] 跨平台扩展：Windows 上设置
//! `CREATE_NO_WINDOW`，其他平台为空操作，调用处无需条件编译。

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// 跨平台扩展 trait：隐藏子进程控制台窗口
///
/// 同时为 `std::process::Command` 与 `tokio::process::Command` 实现，
/// 调用处统一写成 `cmd.hide_console_window()` 即可。
pub trait HideWindowExt: Sized {
    /// 设置创建标志以隐藏子进程控制台窗口（Windows）；其他平台为空操作
    fn hide_console_window(&mut self) -> &mut Self;
}

#[cfg(target_os = "windows")]
mod imp {
    use super::{HideWindowExt, CREATE_NO_WINDOW};
    use std::os::windows::process::CommandExt;

    impl HideWindowExt for std::process::Command {
        fn hide_console_window(&mut self) -> &mut Self {
            self.creation_flags(CREATE_NO_WINDOW)
        }
    }

    impl HideWindowExt for tokio::process::Command {
        fn hide_console_window(&mut self) -> &mut Self {
            // tokio::process::Command 在 Windows 上同样实现了 CommandExt
            self.creation_flags(CREATE_NO_WINDOW)
        }
    }
}

#[cfg(not(target_os = "windows"))]
mod imp {
    use super::HideWindowExt;

    impl HideWindowExt for std::process::Command {
        fn hide_console_window(&mut self) -> &mut Self {
            self
        }
    }

    impl HideWindowExt for tokio::process::Command {
        fn hide_console_window(&mut self) -> &mut Self {
            self
        }
    }
}
