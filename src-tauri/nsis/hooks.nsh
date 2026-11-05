; NSIS 安装/卸载钩子
; 在安装时将 hs.exe 和 huashun.exe 加入系统 PATH
; 在卸载时从 PATH 移除

!macro customInstall
  ; 从构建目录复制 CLI 二进制到安装目录
  ; CI 在 tauri build 前已经构建了 release 版的 hs.exe 和 huashun.exe
  ; 构建目录：{{target}}\release\
  ; 安装目录：$INSTDIR

  SetOutPath "$INSTDIR"
  File /nonfatal "${target}\release\hs.exe"
  File /nonfatal "${target}\release\huashun.exe"

  ; 将程序目录加入系统 PATH（当前用户）
  ; 读取当前 PATH
  ReadRegStr $0 HKCU "Environment" "Path"

  ; 检查是否已经在 PATH 中
  Push $0
  Push "$INSTDIR"
  Call StrStr
  Pop $1

  ; 如果不在 PATH 中，添加
  StrCmp $1 "" 0 +3
    ; 在 PATH 末尾添加安装目录
    StrCpy $0 "$0;$INSTDIR"
    WriteRegExpandStr HKCU "Environment" "Path" "$0"

  ; 通知系统环境变量已更新
  SendMessage ${HWND_BROADCAST} ${WM_WININICHANGE} 0 "STR:Environment" /TIMEOUT=5000
!macroend

!macro customUnInstall
  ; 卸载时从系统 PATH 移除程序目录

  ; 读取当前 PATH
  ReadRegStr $0 HKCU "Environment" "Path"

  ; 移除安装目录
  Push $0
  Push "$INSTDIR"
  Call un.StrStr
  Pop $1

  ; 如果找到，移除
  StrCmp $1 "" done
    ; 移除 PATH 中的安装目录（包括可能的前导 ;）
    ${StrRep} $0 $0 ";$INSTDIR" ""
    ${StrRep} $0 $0 "$INSTDIR;" ""
    ${StrRep} $0 $0 "$INSTDIR" ""
    WriteRegExpandStr HKCU "Environment" "Path" "$0"

  done:
  ; 通知系统环境变量已更新
  SendMessage ${HWND_BROADCAST} ${WM_WININICHANGE} 0 "STR:Environment" /TIMEOUT=5000
!macroend

; StrStr - 在字符串中查找子串
; 输入:
;   栈顶: 子串
;   栈顶-1: 源字符串
; 输出:
;   栈顶: 找到的位置（如果未找到则为空字符串）
Function StrStr
  Exch $R1 ; 子串
  Exch
  Exch $R2 ; 源字符串
  Push $R3
  Push $R4
  Push $R5
  StrLen $R3 $R1
  StrCpy $R4 0
  loop:
    StrCpy $R5 $R2 $R3 $R4
    StrCmp $R5 $R1 done
    StrCmp $R5 "" done
    IntOp $R4 $R4 + 1
    Goto loop
  done:
    StrCpy $R1 $R5
    Pop $R5
    Pop $R4
    Pop $R3
    Pop $R2
    Exch $R1
FunctionEnd

; 卸载版本的 StrStr
Function un.StrStr
  Exch $R1
  Exch
  Exch $R2
  Push $R3
  Push $R4
  Push $R5
  StrLen $R3 $R1
  StrCpy $R4 0
  loop:
    StrCpy $R5 $R2 $R3 $R4
    StrCmp $R5 $R1 done
    StrCmp $R5 "" done
    IntOp $R4 $R4 + 1
    Goto loop
  done:
    StrCpy $R1 $R5
    Pop $R5
    Pop $R4
    Pop $R3
    Pop $R2
    Exch $R1
FunctionEnd
