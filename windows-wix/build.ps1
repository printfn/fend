$ErrorActionPreference = "Stop"

if (Test-Path $PSScriptRoot\build) {
	Remove-Item -Recurse -Force $PSScriptRoot\build
}

mkdir $PSScriptRoot\build

Set-Location "$PSScriptRoot"

dotnet tool restore
dotnet tool run wix extension add WixToolset.UI.wixext/4.0.5
dotnet tool run wix extension add WixToolset.Util.wixext/4.0.5

dotnet tool run wix build `
	-arch x64 -ext WixToolset.UI.wixext -ext WixToolset.Util.wixext `
	-d Version="$Env:FEND_VERSION" `
	-d FendExePath="$PSScriptRoot\..\target\release\fend.exe" `
	-d LicenseMdPath="$PSScriptRoot\..\LICENSE.md" `
	-d IconPath="$PSScriptRoot\..\icon\fend-icon.ico" `
	-o "$PSScriptRoot\build\fend-windows-x64.msi" "$PSScriptRoot\main.wxs"
