; Define installer name and attributes
Name "CloudCafe Installer"
OutFile "CloudCafeInstaller.exe"
InstallDir "$PROGRAMFILES\CloudCafe"
RequestExecutionLevel admin
; Define the installer sections
Section "CloudCafe" SecMyApplication
    ; Set the installer to run in elevated mode
    ;RequestExecutionLevel admin
    SetShellVarContext all
    ; Copy the application files to the installation directory
    SetOutPath "$INSTDIR"
    File "CloudCafe.exe"
    File "CloudCafeDriverInstaller.exe"
    ; Create a start menu shortcut for the application
    CreateDirectory "$SMPROGRAMS\CloudCafe"
    CreateShortCut "$SMPROGRAMS\CloudCafe\CloudCafe.lnk" "$INSTDIR\CloudCafe.exe"
    WriteUninstaller "$INSTDIR\uninstall.exe"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\CloudCafe" "DisplayName" "CloudCafe"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\CloudCafe" "UninstallString" "$INSTDIR\uninstall.exe"
    ; Launch the packaged EXE with specific command line arguments
    ExecWait '"$INSTDIR\CloudCafeDriverInstaller.exe" install'
    ;CreateShortCut "$SMSTARTUP\Uninstall this app.lnk" "$INSTDIR\uninstall.exe"

SectionEnd
RequestExecutionLevel admin
Section "Uninstall"
    ; Set the uninstaller to run in elevated mode
    ;RequestExecutionLevel admin
    SetShellVarContext all
    ; Remove the start menu shortcut
    Delete "$SMPROGRAMS\CloudCafe\CloudCafe.lnk"
    ; Launch the packaged EXE with different command line arguments
    ExecWait '"$INSTDIR\CloudCafeDriverInstaller.exe" uninstall'
    ; Remove the installation directory and its contents
    RMDir /r "$INSTDIR"
    DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\CloudCafe"

SectionEnd