Unicode true
!include "LogicLib.nsh"
!include "FileFunc.nsh"
Name "BibCiTeX"
OutFile "${OUTPUT}"
RequestExecutionLevel user
ShowInstDetails show
AutoCloseWindow true
Page instfiles
Var PowerShell
Function .onInit
  ; Legacy Tauri updater invokes /P /R /UPDATE /ARGS (or /S for quiet).
  ; No elevation: MSIX registration must belong to the current desktop user.
  StrCpy $PowerShell "$SYSDIR\WindowsPowerShell\v1.0\powershell.exe"
  IfFileExists "$WINDIR\Sysnative\WindowsPowerShell\v1.0\powershell.exe" 0 +2
    StrCpy $PowerShell "$WINDIR\Sysnative\WindowsPowerShell\v1.0\powershell.exe"
FunctionEnd
Section
  InitPluginsDir
  SetOutPath "$PLUGINSDIR"
  File /oname=migrate.ps1 "${SCRIPT}"
  File /oname=configuration.json "${CONFIGURATION}"
  File /oname=BibCiTeX.appinstaller "${APPINSTALLER}"
  nsExec::ExecToLog '"$PowerShell" -NoLogo -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$PLUGINSDIR\migrate.ps1" -Configuration "$PLUGINSDIR\configuration.json"'
  Pop $0
  ${If} $0 != 0
    SetErrorLevel 1
    MessageBox MB_OK|MB_ICONSTOP "BibCiTeX 更新未完成，请查看安装详情。" /SD IDOK
    Abort
  ${EndIf}
  SetErrorLevel 0
SectionEnd
