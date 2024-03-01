$ErrorActionPreference = "Stop"

if (Test-Path $PSScriptRoot\build) {
	Remove-Item -Recurse -Force $PSScriptRoot\build
}

mkdir $PSScriptRoot\build

& "C:\Program Files (x86)\WiX Toolset v3.11\bin\candle.exe" `
	-arch x64 -ext WixUtilExtension `
	-dVersion="$Env:FEND_VERSION" `
	-dFendExePath="$PSScriptRoot\..\target\release\fend.exe" `
	-dLicenseMdPath="$PSScriptRoot\..\LICENSE.md" `
	-dIconPath="$PSScriptRoot\..\icon\fend-icon.ico" `
	-o "$PSScriptRoot\build\" "$PSScriptRoot\main.wxs"

& "C:\Program Files (x86)\WiX Toolset v3.11\bin\light.exe" `
	-spdb `
	-ext WixUIExtension `
	-ext WixUtilExtension `
	-cultures:en-US `
	-out "$PSScriptRoot\build\fend-windows-x64.msi" `
	"$PSScriptRoot\build\main.wixobj"
