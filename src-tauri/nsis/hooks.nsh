; Tauri 2 NSIS 安装钩子
; 安装时：把 hs.exe / huashun.exe 放进安装目录，并把安装目录加入系统 PATH
; 卸载时：删除这两个文件，并从系统 PATH 移除安装目录
;
; CLI 二进制在 CI 中先于 tauri build 构建，位于 src-tauri/target/release/。
; ${__FILEDIR__} 是本文件所在目录（src-tauri/nsis），据此定位构建产物。
; 本地如果没先构建 CLI，/nonfatal 只告警不中断打包。
; installMode 为 perMachine，安装器以管理员权限运行，因此写 Machine 级 PATH。

!macro NSIS_HOOK_POSTINSTALL
  SetOutPath "$INSTDIR"
  File /nonfatal "${__FILEDIR__}\..\target\release\hs.exe"
  File /nonfatal "${__FILEDIR__}\..\target\release\huashun.exe"

  nsExec::ExecToLog `powershell -NoProfile -ExecutionPolicy Bypass -Command "$$p=[Environment]::GetEnvironmentVariable('Path','Machine'); if (($$p -split ';') -notcontains '$INSTDIR') { [Environment]::SetEnvironmentVariable('Path', $$p.TrimEnd(';') + ';$INSTDIR', 'Machine') }"`
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  Delete "$INSTDIR\hs.exe"
  Delete "$INSTDIR\huashun.exe"

  nsExec::ExecToLog `powershell -NoProfile -ExecutionPolicy Bypass -Command "$$p=[Environment]::GetEnvironmentVariable('Path','Machine'); $$n=(($$p -split ';') | Where-Object { $$_ -and $$_ -ne '$INSTDIR' }) -join ';'; [Environment]::SetEnvironmentVariable('Path', $$n, 'Machine')"`
!macroend
