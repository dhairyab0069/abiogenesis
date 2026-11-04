; Inno Setup 6 script for Abiogenesis.
; Build: iscc /DAppVersion=0.3.0 /DSourceExe=..\..\target\x86_64-pc-windows-msvc\release\abiogenesis.exe installer.iss
; Output: out\Abiogenesis-<version>-windows-x86_64-setup.exe (relative to this file)

#ifndef AppVersion
  #define AppVersion "0.3.0"
#endif
#ifndef SourceExe
  #define SourceExe "..\..\target\x86_64-pc-windows-msvc\release\abiogenesis.exe"
#endif

#define AppName "Abiogenesis"
#define AppExeName "abiogenesis.exe"

[Setup]
; Fixed GUID: never change it, Windows uses it to recognise upgrades.
AppId={{B3C7F2A4-5D1E-4E8B-9A62-7F0C1D4E8A35}
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher=dhairyab0069
AppPublisherURL=https://github.com/dhairyab0069/abiogenesis
AppSupportURL=https://github.com/dhairyab0069/abiogenesis/issues
DefaultDirName={autopf}\{#AppName}
DefaultGroupName={#AppName}
DisableProgramGroupPage=yes
UninstallDisplayIcon={app}\{#AppExeName}
SetupIconFile=app.ico
OutputDir=out
OutputBaseFilename=Abiogenesis-{#AppVersion}-windows-x86_64-setup
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "{#SourceExe}"; DestDir: "{app}"; DestName: "{#AppExeName}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#AppName}"; Filename: "{app}\{#AppExeName}"; IconFilename: "{app}\{#AppExeName}"
Name: "{group}\Uninstall {#AppName}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\{#AppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#AppExeName}"; Description: "{cm:LaunchProgram,{#AppName}}"; Flags: nowait postinstall skipifsilent
