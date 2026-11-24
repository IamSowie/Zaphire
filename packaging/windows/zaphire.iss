; Zaphire installer (Inno Setup 6). Built by CI:
;   iscc /DAppVersion=0.1.0 packaging\windows\zaphire.iss
; Installs per user (no admin prompt) into %LOCALAPPDATA%\Programs\Zaphire.

#ifndef AppVersion
  #define AppVersion "0.0.0"
#endif

[Setup]
AppId={{6C1E5B0A-6A57-4C64-9B3D-5A3E2F0E7A11}
AppName=Zaphire
AppVersion={#AppVersion}
AppPublisher=Zaphire
AppPublisherURL=https://github.com/IamSowie/Zaphire
DefaultDirName={autopf}\Zaphire
DefaultGroupName=Zaphire
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
OutputDir=..\..\dist
OutputBaseFilename=Zaphire-{#AppVersion}-setup
SetupIconFile=..\..\assets\zaphire.ico
UninstallDisplayIcon={app}\zaphire.exe
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
LicenseFile=..\..\LICENSE

[Languages]
Name: "es"; MessagesFile: "compiler:Languages\Spanish.isl"
Name: "en"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"

[Files]
Source: "..\..\dist\zaphire-windows\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{group}\Zaphire"; Filename: "{app}\zaphire.exe"
Name: "{autodesktop}\Zaphire"; Filename: "{app}\zaphire.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\zaphire.exe"; Description: "{cm:LaunchProgram,Zaphire}"; Flags: nowait postinstall skipifsilent

[UninstallDelete]
; Settings and profile live in %APPDATA%\zaphire and are kept on purpose.
Type: filesandordirs; Name: "{app}"
