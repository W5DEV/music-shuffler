[Setup]
AppName=Music Shuffler
AppVersion=1.0.0
AppPublisher=Your Company
AppPublisherURL=https://github.com/W5DEV/music-shuffler
AppSupportURL=https://github.com/W5DEV/music-shuffler/issues
AppUpdatesURL=https://github.com/W5DEV/music-shuffler/releases
DefaultDirName={autopf}\Music Shuffler
DisableProgramGroupPage=yes
LicenseFile=..\LICENSE
OutputDir=..\dist
OutputBaseFilename=music-shuffler-setup
SetupIconFile=..\assets\icon.ico
Compression=lzma
SolidCompression=yes
WizardStyle=modern
PrivilegesRequired=lowest
ArchitecturesAllowed=x64
ArchitecturesInstallIn64BitMode=x64

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "quicklaunchicon"; Description: "{cm:CreateQuickLaunchIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked; OnlyBelowVersion: 6.1

[Files]
Source: "..\release-windows\music-shuffler.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\release-windows\README.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion; DestName: "LICENSE.txt"

[Icons]
Name: "{autoprograms}\Music Shuffler"; Filename: "{app}\music-shuffler.exe"
Name: "{autodesktop}\Music Shuffler"; Filename: "{app}\music-shuffler.exe"; Tasks: desktopicon
Name: "{userappdata}\Microsoft\Internet Explorer\Quick Launch\Music Shuffler"; Filename: "{app}\music-shuffler.exe"; Tasks: quicklaunchicon

[Run]
Filename: "{app}\music-shuffler.exe"; Description: "{cm:LaunchProgram,Music Shuffler}"; Flags: nowait postinstall skipifsilent

[Code]
function InitializeSetup(): Boolean;
var
  ResultCode: Integer;
begin
  // Check if Visual C++ Redistributables are needed
  if not RegKeyExists(HKLM, 'SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\X64') then
  begin
    if MsgBox('This application requires Visual C++ Redistributables. Would you like to download and install them?', 
              mbConfirmation, MB_YESNO) = IDYES then
    begin
      ShellExec('open', 'https://aka.ms/vs/17/release/vc_redist.x64.exe', '', '', SW_SHOWNORMAL, ewNoWait, ResultCode);
    end;
  end;
  Result := True;
end;

[Registry]
Root: HKCU; Subkey: "Software\Music Shuffler"; ValueType: string; ValueName: "InstallPath"; ValueData: "{app}"

[UninstallDelete]
Type: filesandordirs; Name: "{userappdata}\music-shuffler" 